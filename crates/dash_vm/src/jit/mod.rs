use std::mem::MaybeUninit;
use std::rc::Rc;

use crate::Vm;
use crate::dispatch::{DispatchContext, INSTRUCTION_LUT};
use crate::frame::Ip;
use crate::jit::mmap::MmapFn;
use crate::localscope::LocalScope;
use crate::value::{Unpack, Unrooted, ValueKind};

mod basic_blocks;
mod compiler;
mod jumpresolver;
mod mmap;
mod state;

pub use compiler::CompileError;
use dash_middle::compiler::constant::ConstantPool;
use dash_middle::compiler::extract::{ExtractSource, extract_back_infallible};
use dash_middle::compiler::instruction::Instruction;
use dash_middle::compiler::operands::InstructionOperands;
pub use state::State;

#[derive(Debug)]
pub struct JitFnHandle(Rc<MmapFn>);

impl JitFnHandle {
    pub fn call(&self, vm: &mut Vm) -> JitReturn {
        let mut out = MaybeUninit::<JitOutData>::zeroed();

        let previously_in_jit = vm.frames.replace_in_jit(true);
        let ret = self
            .0
            .call3::<&mut Vm, &JitVtable, &mut MaybeUninit<JitOutData>, InternalJitReturn>(vm, &JIT_VTABLE, &mut out);
        vm.frames.replace_in_jit(previously_in_jit);

        // SAFETY: ip is always in bounds
        let ip = unsafe { &raw const (*out.as_ptr()).ip };

        match ret.status {
            // SAFETY: jit initializes out->ip for normal returns
            HandlerStubStatus::Normal => JitReturn::Normal { ip: Ip(unsafe { *ip }) },
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
    Normal {
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
        Ok(Some(_)) => todo!(),
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

#[repr(u8)]
enum CheckCondition {
    Truthy,
    Nullish,
    Undefined,
}

#[repr(C)]
struct JitVtable {
    stub_fn: extern "C" fn(&mut Vm, *mut JitOutData, u8, u32) -> HandlerStubReturn,
    check_last_value: extern "C" fn(&mut Vm, bool, CheckCondition) -> bool,
}

extern "C" fn check_last_value(vm: &mut Vm, pop: bool, check: CheckCondition) -> bool {
    let value = vm.stack.last().unwrap().clone();

    let result = match check {
        CheckCondition::Truthy => value.is_truthy(vm),
        CheckCondition::Nullish => value.is_nullish(),
        CheckCondition::Undefined => matches!(value.unpack(), ValueKind::Undefined(_)),
    };
    if pop {
        vm.stack.pop();
    }
    result
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
        Ok(JitFnHandle(Rc::clone(func)))
    } else {
        let func = Rc::new(scope.frames.with_current_bytecode(|bytecode| {
            let bytecode = &bytecode[start.0 as usize..end.0 as usize];

            compiler::compile(bytecode, scope, start, end)
        })?);

        scope.jit.compiled_fn_cache.insert(key, Rc::clone(&func));
        Ok(JitFnHandle(func))
    }
}

#[derive(Debug)]
struct NoValueExtractContext<'a, 'vm> {
    bytes: &'a [u8],
    ip: usize,
    scope: &'a LocalScope<'vm>,
}

impl<'a, 'vm> NoValueExtractContext<'a, 'vm> {
    fn new(bytes: &'a [u8], scope: &'a LocalScope<'vm>) -> Self {
        Self { bytes, ip: 0, scope }
    }

    pub fn ip(&self) -> Ip {
        Ip(self.ip as u32)
    }

    pub fn has_next(&self) -> bool {
        self.ip < self.bytes.len()
    }

    pub fn next_instruction_with_operands(&mut self) -> Option<(Ip, InstructionOperands<Self>)> {
        if self.has_next() {
            let ip = self.ip();
            Some((ip, extract_back_infallible(self)))
        } else {
            None
        }
    }
}

impl<'a, 'vm> ExtractSource for NoValueExtractContext<'a, 'vm> {
    type Value = ();

    type Unrooted = ();

    fn fetch_bytes<const N: usize>(&mut self) -> [u8; N] {
        let bytes = self.bytes[self.ip..self.ip + N].try_into().unwrap();
        self.ip += N;
        bytes
    }

    fn constants(&self) -> &ConstantPool {
        &self.scope.frames.current_fn().constants
    }

    fn stack_len(&self) -> usize {
        // Bit of a hack, but usize::MAX (instead of 0) prevents overflows in ForwardSequence stack value count calculation.
        // In any case, the value here should not matter since we never actually use material stack values in the JIT.
        usize::MAX
    }

    fn pop_stack_rooted(&mut self) -> Self::Value {}

    fn pop_stack(&mut self) -> Self::Unrooted {}

    fn peek_stack_rooted(&mut self) -> Self::Value {}

    fn peek_stack(&self) -> Self::Unrooted {}

    fn truncate_stack(&mut self, _: usize) {}
}

impl Iterator for NoValueExtractContext<'_, '_> {
    type Item = (Ip, Instruction);

    fn next(&mut self) -> Option<Self::Item> {
        if self.ip < self.bytes.len() {
            let byte = self.bytes[self.ip];
            self.ip += 1;
            Some((Ip((self.ip - 1) as u32), Instruction::from_repr(byte).unwrap()))
        } else {
            None
        }
    }
}
