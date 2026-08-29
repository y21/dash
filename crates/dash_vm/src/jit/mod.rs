use std::mem::{MaybeUninit, offset_of};
use std::rc::Rc;

use dash_middle::compiler::constant::ConstantPool;
use dash_middle::compiler::extract::{ExtractSource, extract_back_infallible};
use dash_middle::compiler::instruction::Instruction;
use dash_middle::compiler::operands::*;

use crate::Vm;
use crate::dispatch::{DispatchContext, INSTRUCTION_LUT};
use crate::frame::Ip;
use crate::jit::jumpresolver::InternalLabel;
use crate::jit::mmap::MmapFn;
use crate::localscope::LocalScope;
use crate::value::Unrooted;

mod jumpresolver;
mod mmap;
mod state;
mod x86;

pub use state::State;

#[derive(Debug)]
pub struct JitFnHandle(Rc<MmapFn>);

impl JitFnHandle {
    pub fn call(&self, vm: &mut Vm) -> JitReturn {
        let mut out = MaybeUninit::<JitOutData>::zeroed();

        let ret = self
            .0
            .call3::<&mut Vm, &JitVtable, &mut MaybeUninit<JitOutData>, InternalJitReturn>(vm, &JIT_VTABLE, &mut out);

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

#[repr(C)]
struct JitVtable {
    stub_fn: extern "C" fn(&mut Vm, *mut JitOutData, u8, u32) -> HandlerStubReturn,
    last_value_is_truthy: extern "C" fn(&mut Vm, bool) -> bool,
}

extern "C" fn last_value_is_truthy(vm: &mut Vm, pop: bool) -> bool {
    let result = vm.stack.last().unwrap().clone().is_truthy(&mut vm.scope());
    if pop {
        vm.stack.pop();
    }
    result
}

static JIT_VTABLE: JitVtable = JitVtable {
    stub_fn: handler_stub,
    last_value_is_truthy,
};

#[repr(C)]
struct JitOutData {
    ip: u32,
}

struct JitExtractContext<'a, 'vm> {
    bytes: &'a [u8],
    ip: usize,
    scope: &'a LocalScope<'vm>,
}

impl<'a, 'vm> ExtractSource for JitExtractContext<'a, 'vm> {
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
        0
    }

    fn pop_stack_rooted(&mut self) -> Self::Value {}

    fn pop_stack(&mut self) -> Self::Unrooted {}

    fn peek_stack_rooted(&mut self) -> Self::Value {}

    fn peek_stack(&self) -> Self::Unrooted {}

    fn truncate_stack(&mut self, _: usize) {}
}

impl Iterator for JitExtractContext<'_, '_> {
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

fn compile_uncached(scope: &mut LocalScope<'_>, start: Ip, end: Ip) -> MmapFn {
    scope.frames.with_current_bytecode(|bytecode| {
        fn target_from_relative(next_bc_ip: u32, rel: i16) -> Ip {
            let target = next_bc_ip as i64 + rel as i64;
            assert!(target >= 0, "computed negative bytecode ip target: {target}");
            Ip(target as u32)
        }

        let bytecode = &bytecode[start.0 as usize..end.0 as usize];

        let mut x86 = x86::Emitter::new(bytecode.len());

        // Prologue
        // START OF STACK
        x86.push(x86::Register::Rbp); // rsp aligned
        x86.mov_reg_reg(x86::Register::Rbp, x86::Register::Rsp);
        x86.push(x86::Register::R12); // Vm pointer - rbp-8, rsp misaligned by 8
        x86.mov_reg_reg(x86::Register::R12, x86::Register::Rdi);
        x86.push(x86::Register::R13); // Stub fn - rbp-16, rsp aligned
        x86.push(x86::Register::R14); // Vtable - rbp-24, rsp misaligned by 8
        x86.mov_reg_reg(x86::Register::R14, x86::Register::Rsi);
        const OUT_DATA_RBP_OFFSET: i8 = 32;
        x86.push(x86::Register::Rdx); // Out data - rbp-32, rsp aligned
        // END OF STACK

        // The stub fn is very hot, so put it in a callee-saved register
        // TODO: use a call variant that calls [r13+offset] directly
        x86.mov_reg_mem_u8(
            x86::Register::R13,
            x86::Register::Rsi,
            offset_of!(JitVtable, stub_fn).try_into().unwrap(),
        );

        // ... Body ...
        fn emit_stub_for_instr(x86: &mut x86::Emitter, instr: Instruction, ip: u32) {
            x86.mov_reg_reg(x86::Register::Rdi, x86::Register::R12);
            x86.mov_reg_mem_u8(x86::Register::Rsi, x86::Register::Rbp, -OUT_DATA_RBP_OFFSET);
            x86.mov_reg_imm32(x86::Register::Rdx, instr as u32);
            x86.mov_reg_imm32(x86::Register::Rcx, ip);
            x86.call_reg(x86::Register::R13);
            x86.test_reg_reg(x86::Register::Eax, x86::Register::Eax);
            x86.jne_internal_label(InternalLabel::StubStatusHandler);
        }

        let mut cx = JitExtractContext {
            bytes: bytecode,
            ip: 0,
            scope,
        };
        while let Some((instr_ip, instr)) = cx.next() {
            x86.mark_bytecode_ip(instr_ip);

            // IP for the operands *in the full bytecode* of the function (not the sliced loop bytecode).
            let operands_absolute_ip = start.0 + cx.ip as u32;

            macro_rules! emit_stub {
                ($operands:ty) => {{
                    let _: $operands = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }};
                () => {{
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }};
            }

            match instr {
                Instruction::JmpFalseP => {
                    let JmpFalsePopOperands(ConditionalJumpPopOperands { offset, value: _ }) =
                        extract_back_infallible(&mut cx);
                    let target_bc_ip = target_from_relative(cx.ip as u32, offset);

                    x86.mov_reg_mem_u8(
                        x86::Register::Rax,
                        x86::Register::R14,
                        offset_of!(JitVtable, last_value_is_truthy).try_into().unwrap(),
                    );
                    x86.mov_reg_reg(x86::Register::Rdi, x86::Register::R12);
                    x86.mov_reg_imm32(x86::Register::Rsi, 1);
                    x86.call_reg(x86::Register::Rax);
                    x86.cmp_reg_al_imm8(1);
                    x86.jne_bytecode_ip(target_bc_ip);
                }
                Instruction::LoopBackJmp => {
                    let LoopBackjumpOperands { offset, hotness: _ } = extract_back_infallible(&mut cx);
                    let target_bc_ip = target_from_relative(cx.ip as u32, offset);

                    x86.jmp_bytecode_ip(target_bc_ip);
                }
                Instruction::IntrinsicOp => emit_stub!(IntrinsicOperands<JitExtractContext<'_, '_>>),
                Instruction::LdLocal => emit_stub!(LdLocalOperands),
                Instruction::Pop => emit_stub!(PopOperands<JitExtractContext<'_, '_>>),
                Instruction::Add => emit_stub!(AddOperands<JitExtractContext<'_, '_>>),
                Instruction::Sub => emit_stub!(SubOperands<JitExtractContext<'_, '_>>),
                Instruction::Mul => emit_stub!(MulOperands<JitExtractContext<'_, '_>>),
                Instruction::Div => emit_stub!(DivOperands<JitExtractContext<'_, '_>>),
                Instruction::Rem => emit_stub!(RemOperands<JitExtractContext<'_, '_>>),
                Instruction::Pow => emit_stub!(PowOperands<JitExtractContext<'_, '_>>),
                Instruction::Gt => emit_stub!(GtOperands<JitExtractContext<'_, '_>>),
                Instruction::Ge => emit_stub!(GeOperands<JitExtractContext<'_, '_>>),
                Instruction::Lt => emit_stub!(LtOperands<JitExtractContext<'_, '_>>),
                Instruction::Le => emit_stub!(LeOperands<JitExtractContext<'_, '_>>),
                Instruction::Eq => emit_stub!(EqOperands<JitExtractContext<'_, '_>>),
                Instruction::Ne => emit_stub!(NeOperands<JitExtractContext<'_, '_>>),
                Instruction::LdGlobal => emit_stub!(LdGlobalOperands),
                Instruction::String => emit_stub!(StringConstantOperands),
                Instruction::Boolean => emit_stub!(BooleanConstantOperands),
                Instruction::Number => emit_stub!(NumberConstantOperands),
                Instruction::Regex => emit_stub!(RegexConstantOperands),
                Instruction::Null => emit_stub!(),
                Instruction::Undefined => emit_stub!(),
                Instruction::Function => emit_stub!(FunctionConstantOperands),
                Instruction::Pos => emit_stub!(PosOperands<JitExtractContext<'_, '_>>),
                Instruction::Neg => emit_stub!(NegOperands<JitExtractContext<'_, '_>>),
                Instruction::TypeOf => emit_stub!(TypeofOperands<JitExtractContext<'_, '_>>),
                Instruction::TypeOfGlobalIdent => emit_stub!(TypeofIdentOperands),
                Instruction::BitNot => emit_stub!(BitnotOperands<JitExtractContext<'_, '_>>),
                Instruction::Not => emit_stub!(NotOperands<JitExtractContext<'_, '_>>),
                Instruction::StoreLocal => emit_stub!(StoreLocalOperands<JitExtractContext<'_, '_>>),
                Instruction::StoreGlobal => emit_stub!(StoreGlobalOperands<JitExtractContext<'_, '_>>),
                Instruction::Ret => emit_stub!(RetOperands<JitExtractContext<'_, '_>>),
                Instruction::Call => emit_stub!(CallOperands),
                Instruction::Jmp => emit_stub!(JmpOperands),
                Instruction::StaticPropAccess => {
                    emit_stub!(StaticPropertyAccessOperands<JitExtractContext<'_, '_>>)
                }
                Instruction::DynamicPropAccess => {
                    emit_stub!(DynamicPropertyAccessOperands<JitExtractContext<'_, '_>>)
                }
                Instruction::ArrayLit => emit_stub!(ArrayLiteralOperands),
                Instruction::ObjLit => emit_stub!(ObjectLiteralOperands<JitExtractContext<'_, '_>>),
                Instruction::BindThis => emit_stub!(BindThisOperands<JitExtractContext<'_, '_>>),
                Instruction::This => emit_stub!(),
                Instruction::StaticPropAssign => {
                    emit_stub!(StaticPropertyAssignOperands<JitExtractContext<'_, '_>>)
                }
                Instruction::DynamicPropAssign => {
                    emit_stub!(DynamicPropertyAssignOperands<JitExtractContext<'_, '_>>)
                }
                Instruction::LdLocalExt => emit_stub!(LdLocalExtOperands),
                Instruction::StoreLocalExt => emit_stub!(StoreLocalExtOperands<JitExtractContext<'_, '_>>),
                Instruction::StrictEq => emit_stub!(StrictEqOperands<JitExtractContext<'_, '_>>),
                Instruction::StrictNe => emit_stub!(StrictNeOperands<JitExtractContext<'_, '_>>),
                Instruction::PopTry => emit_stub!(),
                Instruction::FinallyEnd => emit_stub!(FinallyEndOperands),
                Instruction::Throw => emit_stub!(ThrowOperands<JitExtractContext<'_, '_>>),
                Instruction::Yield => emit_stub!(YieldOperands<JitExtractContext<'_, '_>>),
                Instruction::JmpFalseNP => emit_stub!(JmpFalseNoPopOperands<JitExtractContext<'_, '_>>),
                Instruction::JmpTrueP => emit_stub!(JmpTruePopOperands<JitExtractContext<'_, '_>>),
                Instruction::JmpTrueNP => emit_stub!(JmpTrueNoPopOperands<JitExtractContext<'_, '_>>),
                Instruction::JmpNullishP => emit_stub!(JmpNullishPopOperands<JitExtractContext<'_, '_>>),
                Instruction::JmpNullishNP => emit_stub!(JmpNullishNoPopOperands<JitExtractContext<'_, '_>>),
                Instruction::JmpUndefinedNP => emit_stub!(JmpUndefinedNoPopOperands<JitExtractContext<'_, '_>>),
                Instruction::JmpUndefinedP => emit_stub!(JmpUndefinedPopOperands<JitExtractContext<'_, '_>>),
                Instruction::BitOr => emit_stub!(BitorOperands<JitExtractContext<'_, '_>>),
                Instruction::BitXor => emit_stub!(BitxorOperands<JitExtractContext<'_, '_>>),
                Instruction::BitAnd => emit_stub!(BitandOperands<JitExtractContext<'_, '_>>),
                Instruction::BitShl => emit_stub!(BitshlOperands<JitExtractContext<'_, '_>>),
                Instruction::BitShr => emit_stub!(BitshrOperands<JitExtractContext<'_, '_>>),
                Instruction::BitUshr => emit_stub!(BitushrOperands<JitExtractContext<'_, '_>>),
                Instruction::ObjIn => emit_stub!(ObjectInOperands<JitExtractContext<'_, '_>>),
                Instruction::InstanceOf => emit_stub!(InstanceofOperands<JitExtractContext<'_, '_>>),
                Instruction::ImportDyn => emit_stub!(ImportDynOperands<JitExtractContext<'_, '_>>),
                Instruction::ImportStatic => emit_stub!(ImportStaticOperands),
                Instruction::ExportDefault => emit_stub!(ExportDefaultOperands<JitExtractContext<'_, '_>>),
                Instruction::ExportNamed => emit_stub!(ExportNamedOperands),
                Instruction::Debugger => emit_stub!(),
                Instruction::Global => emit_stub!(),
                Instruction::Super => emit_stub!(),
                Instruction::Undef => emit_stub!(),
                Instruction::Await => emit_stub!(AwaitOperands<JitExtractContext<'_, '_>>),
                Instruction::Nan => emit_stub!(),
                Instruction::Infinity => emit_stub!(),
                Instruction::CallSymbolIterator => {
                    emit_stub!(CallSymbolIteratorOperands<JitExtractContext<'_, '_>>)
                }
                Instruction::CallForInIterator => {
                    emit_stub!(ForInIteratorOperands<JitExtractContext<'_, '_>>)
                }
                Instruction::DeletePropertyStatic => {
                    emit_stub!(DeletePropertyStaticOperands<JitExtractContext<'_, '_>>)
                }
                Instruction::DeletePropertyDynamic => {
                    emit_stub!(DeletePropertyDynamicOperands<JitExtractContext<'_, '_>>)
                }
                Instruction::ObjDestruct => {
                    emit_stub!(ObjectDestructuringOperands<JitExtractContext<'_, '_>>)
                }
                Instruction::ArrayDestruct => {
                    emit_stub!(ArrayDestructuringOperands<JitExtractContext<'_, '_>>)
                }
                Instruction::AssignProperties => {
                    emit_stub!(AssignPropertiesOperands<JitExtractContext<'_, '_>>)
                }
                Instruction::DelayedReturn => emit_stub!(DelayedRetOperands<JitExtractContext<'_, '_>>),
                Instruction::NewTarget => emit_stub!(),
                Instruction::Nop => emit_stub!(),
                Instruction::Try => todo!("try instructions are not yet supported by the JIT"),
            }
        }

        // Exit branch (end-of-loop/end-of-bytecode)
        assert!(cx.ip == bytecode.len());
        assert!(start.0 + cx.ip as u32 == end.0);
        x86.mark_bytecode_ip(Ip(cx.ip as u32));
        x86.mov_reg_mem_u8(x86::Register::Rax, x86::Register::Rbp, -OUT_DATA_RBP_OFFSET);
        x86.move_mem_imm32(
            x86::Register::Rax,
            offset_of!(JitOutData, ip).try_into().unwrap(),
            end.0.cast_signed(),
        );
        x86.mov_reg_imm32(x86::Register::Eax, 0);

        // Epilogue
        x86.mark_internal_label(InternalLabel::Epilogue);
        x86.add_rsp_imm8(8); // rdx - not a callee saved register and we have the return payload in that reg already, so just discard directly
        x86.pop(x86::Register::R14);
        x86.pop(x86::Register::R13);
        x86.pop(x86::Register::R12);
        x86.pop(x86::Register::Rbp);
        x86.ret();

        x86.mark_internal_label(InternalLabel::StubStatusHandler);
        x86.jmp_internal_label(InternalLabel::Epilogue);

        MmapFn::alloc(x86.buffer())
    })
}

pub fn compile_loop_region(scope: &mut LocalScope<'_>, start: Ip, end: Ip) -> JitFnHandle {
    let current_fn = Rc::as_ptr(scope.frames.current_fn());
    let key = (current_fn, start);

    if let Some(func) = scope.jit.compiled_fn_cache.get(&key) {
        JitFnHandle(Rc::clone(func))
    } else {
        let func = Rc::new(compile_uncached(scope, start, end));
        scope.jit.compiled_fn_cache.insert(key, Rc::clone(&func));
        JitFnHandle(func)
    }
}
