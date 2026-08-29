use std::mem::{MaybeUninit, offset_of};
use std::rc::Rc;

use dash_middle::compiler::constant::ConstantPool;
use dash_middle::compiler::extract::{ExtractSource, extract_back_infallible};
use dash_middle::compiler::instruction::Instruction;
use dash_middle::compiler::operands::*;
use dash_middle::exhaust;

use crate::Vm;
use crate::dispatch::{DispatchContext, INSTRUCTION_LUT};
use crate::frame::Ip;
use crate::jit::jumpresolver::InternalLabel;
use crate::jit::mmap::MmapFn;
use crate::localscope::LocalScope;
use crate::value::{Unpack, Unrooted, ValueKind};

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

#[derive(Debug)]
pub enum CompileError {
    UnhandledInstruction(Instruction),
}

fn compile_uncached(scope: &mut LocalScope<'_>, start: Ip, end: Ip) -> Result<MmapFn, CompileError> {
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

        fn emit_conditional_jump(
            cx: &mut JitExtractContext,
            x86: &mut x86::Emitter,
            offset: i16,
            check: CheckCondition,
            jump_if_true: bool,
            pop: bool,
        ) {
            let target_bc_ip = target_from_relative(cx.ip as u32, offset);
            x86.mov_reg_mem_u8(
                x86::Register::Rax,
                x86::Register::R14,
                offset_of!(JitVtable, check_last_value).try_into().unwrap(),
            );
            x86.mov_reg_reg(x86::Register::Rdi, x86::Register::R12);
            x86.mov_reg_imm32(x86::Register::Rsi, pop.into());
            x86.mov_reg_imm32(x86::Register::Rdx, check as u32);
            x86.call_reg(x86::Register::Rax);
            x86.cmp_reg_al_imm8(1);
            match jump_if_true {
                true => x86.je_bytecode_ip(target_bc_ip),
                false => x86.jne_bytecode_ip(target_bc_ip),
            }
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
                    emit_conditional_jump(&mut cx, &mut x86, offset, CheckCondition::Truthy, false, true);
                }
                Instruction::JmpFalseNP => {
                    let JmpFalseNoPopOperands(ConditionalJumpNoPopOperands { offset, value: _ }) =
                        extract_back_infallible(&mut cx);
                    emit_conditional_jump(&mut cx, &mut x86, offset, CheckCondition::Truthy, false, false);
                }
                Instruction::JmpTrueP => {
                    let JmpTruePopOperands(ConditionalJumpPopOperands { offset, value: _ }) =
                        extract_back_infallible(&mut cx);
                    emit_conditional_jump(&mut cx, &mut x86, offset, CheckCondition::Truthy, true, true);
                }
                Instruction::JmpTrueNP => {
                    let JmpTrueNoPopOperands(ConditionalJumpNoPopOperands { offset, value: _ }) =
                        extract_back_infallible(&mut cx);
                    emit_conditional_jump(&mut cx, &mut x86, offset, CheckCondition::Truthy, true, false);
                }
                Instruction::JmpNullishNP => {
                    let JmpNullishNoPopOperands(ConditionalJumpNoPopOperands { offset, value: _ }) =
                        extract_back_infallible(&mut cx);
                    emit_conditional_jump(&mut cx, &mut x86, offset, CheckCondition::Nullish, true, false);
                }
                Instruction::JmpNullishP => {
                    let JmpNullishPopOperands(ConditionalJumpPopOperands { offset, value: _ }) =
                        extract_back_infallible(&mut cx);
                    emit_conditional_jump(&mut cx, &mut x86, offset, CheckCondition::Nullish, true, true);
                }
                Instruction::JmpUndefinedNP => {
                    let JmpUndefinedNoPopOperands(ConditionalJumpNoPopOperands { offset, value: _ }) =
                        extract_back_infallible(&mut cx);
                    emit_conditional_jump(&mut cx, &mut x86, offset, CheckCondition::Undefined, true, false);
                }
                Instruction::JmpUndefinedP => {
                    let JmpUndefinedPopOperands(ConditionalJumpPopOperands { offset, value: _ }) =
                        extract_back_infallible(&mut cx);
                    emit_conditional_jump(&mut cx, &mut x86, offset, CheckCondition::Undefined, true, true);
                }
                Instruction::Jmp => {
                    let JmpOperands(offset) = extract_back_infallible(&mut cx);
                    let target_bc_ip = target_from_relative(cx.ip as u32, offset);
                    x86.jmp_bytecode_ip(target_bc_ip);
                }
                Instruction::LoopBackJmp => {
                    let LoopBackjumpOperands { offset, hotness: _ } = extract_back_infallible(&mut cx);
                    let target_bc_ip = target_from_relative(cx.ip as u32, offset);
                    x86.jmp_bytecode_ip(target_bc_ip);
                }
                Instruction::LdLocal => {
                    let LdLocalOperands(_) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::IntrinsicOp => {
                    let IntrinsicOperands(_) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::Pop => {
                    let PopOperands(_) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::Add => {
                    let AddOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::Sub => {
                    let SubOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::Mul => {
                    let MulOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::Div => {
                    let DivOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::Rem => {
                    let RemOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::Pow => {
                    let PowOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::Gt => {
                    let GtOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::Ge => {
                    let GeOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::Lt => {
                    let LtOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::Le => {
                    let LeOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::Eq => {
                    let EqOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::Ne => {
                    let NeOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::LdGlobal => {
                    let LdGlobalOperands(_) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::String => {
                    let StringConstantOperands(_) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::Boolean => {
                    let BooleanConstantOperands(_) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::Number => {
                    let NumberConstantOperands(_) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::Regex => {
                    let RegexConstantOperands(_) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::Null => emit_stub!(),
                Instruction::Undefined => emit_stub!(),
                Instruction::Function => {
                    let FunctionConstantOperands(_) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::Pos => {
                    let PosOperands(_) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::Neg => {
                    let NegOperands(_) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::TypeOf => {
                    let TypeofOperands { value: _ } = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::TypeOfGlobalIdent => {
                    let TypeofIdentOperands(_) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::BitNot => {
                    let BitnotOperands(_) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::Not => {
                    let NotOperands(_) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::StoreLocal => {
                    let StoreLocalOperands { local: _, kind: _ } = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::StoreGlobal => {
                    let StoreGlobalOperands { name: _, kind: _ } = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::Ret => {
                    let RetOperands { tc_depth: _, value: _ } = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::Call => {
                    let CallOperands {
                        argc: _,
                        has_this: _,
                        function_call_kind: _,
                        spread_indices,
                    } = extract_back_infallible(&mut cx);
                    exhaust!(spread_indices, &mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::StaticPropAccess => {
                    let StaticPropertyAccessOperands { ident: _, target: _ } = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::DynamicPropAccess => {
                    let DynamicPropertyAccessOperands { key: _, target: _ } = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::ArrayLit => {
                    let ArrayLiteralOperands { members, dense: _ } = extract_back_infallible(&mut cx);
                    exhaust!(members, &mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::ObjLit => {
                    let ObjectLiteralOperands { members } = extract_back_infallible(&mut cx);
                    exhaust!(members, &mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::BindThis => {
                    let BindThisOperands { value: _ } = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::This => emit_stub!(),
                Instruction::StaticPropAssign => {
                    let StaticPropertyAssignOperands {
                        kind: _,
                        target: _,
                        key: _,
                    } = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::DynamicPropAssign => {
                    let DynamicPropertyAssignOperands {
                        kind: _,
                        key: _,
                        target: _,
                    } = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::LdLocalExt => {
                    let LdLocalExtOperands(_) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::StoreLocalExt => {
                    let StoreLocalExtOperands { local: _, kind: _ } = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::StrictEq => {
                    let StrictEqOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::StrictNe => {
                    let StrictNeOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::PopTry => emit_stub!(),
                Instruction::FinallyEnd => {
                    let FinallyEndOperands(_) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::Throw => {
                    let ThrowOperands { value: _ } = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::Yield => {
                    let YieldOperands { value: _ } = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::BitOr => {
                    let BitorOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::BitXor => {
                    let BitxorOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::BitAnd => {
                    let BitandOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::BitShl => {
                    let BitshlOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::BitShr => {
                    let BitshrOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::BitUshr => {
                    let BitushrOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::ObjIn => {
                    let ObjectInOperands { target: _, key: _ } = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::InstanceOf => {
                    let InstanceofOperands {
                        constructor: _,
                        value: _,
                    } = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::ImportDyn => {
                    let ImportDynOperands { value: _ } = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::ImportStatic => {
                    let ImportStaticOperands {
                        kind: _,
                        local: _,
                        path: _,
                    } = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::ExportDefault => {
                    let ExportDefaultOperands { value: _ } = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::ExportNamed => {
                    let ExportNamedOperands { members: _ } = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::Debugger => emit_stub!(),
                Instruction::Global => emit_stub!(),
                Instruction::Super => emit_stub!(),
                Instruction::Undef => emit_stub!(),
                Instruction::Await => {
                    let AwaitOperands { value: _ } = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::Nan => emit_stub!(),
                Instruction::Infinity => emit_stub!(),
                Instruction::CallSymbolIterator => {
                    let CallSymbolIteratorOperands { value: _ } = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::CallForInIterator => {
                    let ForInIteratorOperands { value: _ } = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::DeletePropertyStatic => {
                    let DeletePropertyStaticOperands { target: _, key: _ } = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::DeletePropertyDynamic => {
                    let DeletePropertyDynamicOperands { target: _, key: _ } = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::ObjDestruct => {
                    let ObjectDestructuringOperands {
                        rest_local_id: _,
                        target: _,
                        members,
                    } = extract_back_infallible(&mut cx);
                    exhaust!(members, &mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::ArrayDestruct => {
                    let ArrayDestructuringOperands { array: _, members } = extract_back_infallible(&mut cx);
                    exhaust!(members, &mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::AssignProperties => {
                    let AssignPropertiesOperands { members, target: _ } = extract_back_infallible(&mut cx);
                    exhaust!(members, &mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::DelayedReturn => {
                    let DelayedRetOperands(_) = extract_back_infallible(&mut cx);
                    emit_stub_for_instr(&mut x86, instr, operands_absolute_ip);
                }
                Instruction::NewTarget => emit_stub!(),
                Instruction::Nop => emit_stub!(),
                Instruction::Try => return Err(CompileError::UnhandledInstruction(instr)),
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

        Ok(MmapFn::alloc(x86.buffer()))
    })
}

pub fn compile_loop_region(scope: &mut LocalScope<'_>, start: Ip, end: Ip) -> Result<JitFnHandle, CompileError> {
    let current_fn = Rc::as_ptr(scope.frames.current_fn());
    let key = (current_fn, start);

    if let Some(func) = scope.jit.compiled_fn_cache.get(&key) {
        Ok(JitFnHandle(Rc::clone(func)))
    } else {
        let func = Rc::new(compile_uncached(scope, start, end)?);
        scope.jit.compiled_fn_cache.insert(key, Rc::clone(&func));
        Ok(JitFnHandle(func))
    }
}
