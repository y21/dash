use std::error::Error;
use std::fmt::Display;
use std::mem::MaybeUninit;
use std::ops::{Bound, Index, RangeBounds};
use std::rc::Rc;

use crate::Vm;
use crate::dispatch::{DispatchContext, HandleResult, INSTRUCTION_LUT};
use crate::frame::Ip;
use crate::jit::codegen::CompileResult;
use crate::jit::ir::{ConditionalGotoCondition, ConditionalGotoPopPolicy};
use crate::jit::mmap::MmapFn;
use crate::localscope::LocalScope;
use crate::value::{Unpack, Unrooted, ValueKind};

mod codegen;
mod dataflow;
mod ir;
mod mmap;
mod passes;
mod state;

use dash_log::debug;
use dash_middle::compiler::instruction::Instruction;
use dash_middle::compiler::operands::InstructionOperands;
pub use state::State;

#[derive(Debug)]
pub enum CompileError {
    UnimplementedInstruction(Instruction, Ip),
}

impl Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::UnimplementedInstruction(instr, ip) => {
                write!(f, "Unimplemented instruction {:?} at {:?}", instr, ip)
            }
        }
    }
}

impl Error for CompileError {}

#[derive(Debug)]
struct InnerJitFnHandle {
    func: MmapFn,
    max_stack_depth: usize,
}

#[derive(Debug, Clone)]
pub struct JitFnHandle(Rc<InnerJitFnHandle>);

impl JitFnHandle {
    fn reserve_max_stack_depth(&self, vm: &mut Vm) {
        let len = vm.stack.len();
        vm.stack.reserve(len + self.0.max_stack_depth);
    }

    pub fn call(&self, vm: &mut Vm) -> JitReturn {
        self.reserve_max_stack_depth(vm);

        let mut out = MaybeUninit::<JitOutData>::zeroed();

        let previously_in_jit = vm.frames.replace_in_jit(true);
        let ret = self
            .0
            .func
            .call3::<&mut Vm, &JitVtable, &mut MaybeUninit<JitOutData>, InternalJitReturn>(vm, &JIT_VTABLE, &mut out);
        vm.frames.replace_in_jit(previously_in_jit);

        // SAFETY: ip is always in bounds
        let ip = unsafe { &raw const (*out.as_ptr()).ip };

        match ret.status {
            // SAFETY: jit initializes out->ip for normal returns
            HandlerStubStatus::Normal => JitReturn::Normal { ip: Ip(unsafe { *ip }) },
            // SAFETY: jit initializes out->ip for bailout returns
            HandlerStubStatus::PreludeTypeMismatchBailout => {
                JitReturn::PreludeTypeMismatchBailout { ip: Ip(unsafe { *ip }) }
            }
            HandlerStubStatus::Return => JitReturn::Return {
                value: unsafe { ret.payload.value },
            },
            HandlerStubStatus::FlatCallReturn => JitReturn::FlatCallReturn,
            HandlerStubStatus::Exception => {
                let exception = unsafe { ret.payload.value };
                JitReturn::Exception { value: exception }
            }
        }
    }
}

type InternalJitReturn = HandlerStubReturn;

#[derive(Debug)]
pub enum JitReturn {
    /// JIT code returns normally at the given bytecode ip.
    /// IP needs to be synchronized with the interpreter state.
    Normal {
        ip: Ip,
    },
    /// A return from the currently executing function.
    Return {
        value: Unrooted,
    },
    /// A return from a flat call. The return value is on the stack.
    /// The frame has already been popped and execution is expected to continue in the interpreter as usual.
    /// Interpreter state such as IP is already synchronized.
    FlatCallReturn,
    PreludeTypeMismatchBailout {
        ip: Ip,
    },
    Exception {
        value: Unrooted,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
#[repr(C)]
pub enum HandlerStubStatus {
    Normal,
    Return,
    FlatCallReturn,
    PreludeTypeMismatchBailout,
    Exception,
}

#[repr(C)]
pub union HandlerStubPayload {
    value: Unrooted,
    uninit: (),
}

#[repr(C)]
struct HandlerStubReturn {
    status: HandlerStubStatus,
    payload: HandlerStubPayload,
}

/// Called by JIT code to call a handler and synchronize any vm state.
extern "C" fn handler_stub(vm: &mut Vm, _: *mut JitOutData, handler: u8, ip: u32) -> HandlerStubReturn {
    vm.frames.set_ip(Ip(ip));
    let cx = DispatchContext::new(vm.scope());

    let result = INSTRUCTION_LUT[handler as usize](cx);

    match result {
        Ok(Some(HandleResult::Return(value))) => HandlerStubReturn {
            status: HandlerStubStatus::Return,
            payload: HandlerStubPayload { value },
        },
        Ok(Some(HandleResult::Await(_) | HandleResult::Yield(_))) => todo!("await/yield unhandled"),
        Ok(None) => HandlerStubReturn {
            status: HandlerStubStatus::Normal,
            payload: HandlerStubPayload { uninit: () },
        },
        Err(exception) => HandlerStubReturn {
            status: HandlerStubStatus::Exception,
            payload: HandlerStubPayload { value: exception },
        },
    }
}

#[repr(C)]
struct JitVtable {
    stub_fn: extern "C" fn(&mut Vm, *mut JitOutData, u8, u32) -> HandlerStubReturn,
    check_last_value: extern "C" fn(&mut Vm, ConditionalGotoPopPolicy, ConditionalGotoCondition) -> bool,
}

extern "C" fn check_last_value(
    vm: &mut Vm,
    pop: ConditionalGotoPopPolicy,
    condition: ConditionalGotoCondition,
) -> bool {
    let value = match pop {
        ConditionalGotoPopPolicy::Pop => vm.stack.pop().unwrap(),
        ConditionalGotoPopPolicy::Peek => *vm.stack.last().unwrap(),
    };

    match condition {
        ConditionalGotoCondition::IsFalsy => !value.is_truthy(vm),
        ConditionalGotoCondition::IsFalse => matches!(value.unpack(), ValueKind::Boolean(false)),
        ConditionalGotoCondition::IsTrue => matches!(value.unpack(), ValueKind::Boolean(true)),
        ConditionalGotoCondition::IsTruthy => value.is_truthy(vm),
        ConditionalGotoCondition::IsNullish => value.is_nullish(),
        ConditionalGotoCondition::IsUndefined => matches!(value.unpack(), ValueKind::Undefined(_)),
    }
}

static JIT_VTABLE: JitVtable = JitVtable {
    stub_fn: handler_stub,
    check_last_value,
};

#[repr(C)]
struct JitOutData {
    ip: u32,
}

pub fn compile_loop_region(scope: &mut LocalScope<'_>, start: Ip, end: Ip) -> Result<JitFnHandle, CompileError> {
    let current_fn = Rc::as_ptr(scope.frames.current_fn());
    let key = (current_fn, start);

    if let Some(func) = scope.jit.compiled_fn_cache.get(&key) {
        Ok(func.clone())
    } else {
        let handle = scope.frames.with_current_bytecode(|bytecode| {
            let bytecode = &bytecode[start.0 as usize..end.0 as usize];
            let bytecode = BufferRef::from(bytecode);

            let mut body = ir::lower_body(bytecode, scope)?;
            let mut dataflow = dataflow::DataflowEngine::compute(&body);
            passes::run(&mut body, &mut dataflow);

            debug!("{body:#?}");

            let CompileResult { func, max_stack_depth } = codegen::compile(&body, start, end, &dataflow, scope);

            Ok(JitFnHandle(Rc::new(InnerJitFnHandle { func, max_stack_depth })))
        })?;

        // TODO(idea): cache compiled fns (and other metadata like dataflow analysis?) on disk?
        scope.jit.compiled_fn_cache.insert(key, handle.clone());
        Ok(handle)
    }
}

#[derive(Debug)]
struct NoValueExtractContext<'a>(dash_middle::compiler::extract::NoValueExtractContext<'a>);

impl<'a> NoValueExtractContext<'a> {
    fn new(bytes: &'a [u8], ip: Ip, vm: &'a Vm) -> Self {
        Self(dash_middle::compiler::extract::NoValueExtractContext {
            bytes,
            ip: ip.0 as usize,
            constants: &vm.frames.current_fn().constants,
        })
    }

    pub fn ip(&self) -> Ip {
        Ip(self.0.ip as u32)
    }

    pub fn next_instruction_with_operands(
        &mut self,
    ) -> Option<(
        Ip,
        InstructionOperands<dash_middle::compiler::extract::NoValueExtractContext<'a>>,
    )> {
        self.0
            .next_instruction_with_operands()
            .map(|(ip, operands)| (Ip(ip as u32), operands))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct BufferRef<'a> {
    pub buf: &'a [u8],
}

impl Index<Ip> for BufferRef<'_> {
    type Output = u8;

    fn index(&self, index: Ip) -> &Self::Output {
        &self.buf[index.0 as usize]
    }
}

impl<'a> BufferRef<'a> {
    pub fn len(&self) -> u32 {
        self.buf.len() as u32
    }

    pub fn as_slice(&self) -> &'a [u8] {
        self.buf
    }

    #[expect(dead_code)]
    pub fn slice(&self, r: impl RangeBounds<Ip>) -> BufferRef<'a> {
        let start = map_bound_into(r.start_bound(), |&Ip(ip)| ip as usize);
        let end = map_bound_into(r.end_bound(), |&Ip(ip)| ip as usize);
        Self {
            buf: &self.buf[(start, end)],
        }
    }
}

fn map_bound_into<T, U>(t: Bound<T>, mapper: impl Fn(T) -> U) -> Bound<U> {
    match t {
        Bound::Included(t) => Bound::Included(mapper(t)),
        Bound::Excluded(t) => Bound::Excluded(mapper(t)),
        Bound::Unbounded => Bound::Unbounded,
    }
}

impl<'a> From<&'a [u8]> for BufferRef<'a> {
    fn from(buf: &'a [u8]) -> Self {
        BufferRef { buf }
    }
}
