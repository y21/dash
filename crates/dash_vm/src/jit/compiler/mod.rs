use std::mem::offset_of;

use dash_middle::compiler::constant::ConstantPool;
use dash_middle::compiler::extract::{ExtractSource, extract_back_infallible};
use dash_middle::compiler::instruction::Instruction;
use dash_middle::compiler::operands::*;
use dash_middle::exhaust;

use crate::frame::Ip;
use crate::jit::compiler::backend::{
    PrologueData, REG_ARG1, REG_ARG2, REG_ARG3, REG_ARG4, REG_RETURN32, REG_RETURN64, REG_STACK_RELATIVE, Register,
    call_reg, cmp_reg_al_imm8, epilogue, je_bytecode_ip, jmp_bytecode_ip, jmp_internal_label, jne_bytecode_ip,
    jne_internal_label, mark_bytecode_ip, mark_internal_label, mov_mem_imm32, mov_reg_imm32, mov_reg_mem_u8,
    mov_reg_reg, prologue, test_reg_reg, usable_callee_saved_regs, usable_caller_saved_regs,
};
use crate::jit::compiler::codebuffer::CodeBuffer;
use crate::jit::jumpresolver::InternalLabel;
use crate::jit::mmap::MmapFn;
use crate::jit::{CheckCondition, JitOutData, JitVtable};
use crate::localscope::LocalScope;

mod backend;
mod codebuffer;

#[derive(Debug)]
#[expect(dead_code)]
pub enum CompileError {
    UnhandledInstruction(Instruction),
    UnsupportedArchitecture,
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

fn emit_stub_for_instr(buf: &mut CodeBuffer, instr: Instruction, ip: u32, prologue_data: &PrologueData) {
    mov_reg_reg(buf, REG_ARG1, prologue_data.vm_csreg);
    mov_reg_mem_u8(buf, REG_ARG2, REG_STACK_RELATIVE, -prologue_data.out_data_rbp_offset);
    mov_reg_imm32(buf, REG_ARG3, instr as u32);
    mov_reg_imm32(buf, REG_ARG4, ip);
    call_reg(buf, prologue_data.stub_csreg);
    test_reg_reg(buf, REG_RETURN32, REG_RETURN32);
    jne_internal_label(buf, InternalLabel::StubStatusHandler);
}

fn target_from_relative(next_bc_ip: u32, rel: i16) -> Ip {
    let target = next_bc_ip as i64 + rel as i64;
    assert!(target >= 0, "computed negative bytecode ip target: {target}");
    Ip(target as u32)
}

fn emit_conditional_jump(
    cx: &mut JitExtractContext,
    buf: &mut CodeBuffer,
    prologue_data: &PrologueData,
    caller_regs: &mut Vec<Register>,
    offset: i16,
    check: CheckCondition,
    jump_if_true: bool,
    pop: bool,
) {
    let fn_reg = caller_regs.pop().unwrap();

    let target_bc_ip = target_from_relative(cx.ip as u32, offset);
    mov_reg_mem_u8(
        buf,
        fn_reg,
        prologue_data.vtable_csreg,
        offset_of!(JitVtable, check_last_value).try_into().unwrap(),
    );
    mov_reg_reg(buf, REG_ARG1, prologue_data.vm_csreg);
    mov_reg_imm32(buf, REG_ARG2, pop.into());
    mov_reg_imm32(buf, REG_ARG3, check as u32);
    call_reg(buf, fn_reg);
    cmp_reg_al_imm8(buf, 1);
    match jump_if_true {
        true => je_bytecode_ip(buf, target_bc_ip),
        false => jne_bytecode_ip(buf, target_bc_ip),
    }

    caller_regs.push(fn_reg);
}

pub fn compile(bytecode: &[u8], scope: &LocalScope<'_>, start: Ip, end: Ip) -> Result<MmapFn, CompileError> {
    let mut buffer = CodeBuffer::new(bytecode.len());

    let mut callee_regs = usable_callee_saved_regs();
    let mut caller_regs = usable_caller_saved_regs();

    let prologue_data = prologue(&mut buffer, &mut callee_regs);

    let mut cx = JitExtractContext {
        bytes: bytecode,
        ip: 0,
        scope,
    };

    while let Some((instr_ip, instr)) = cx.next() {
        mark_bytecode_ip(&mut buffer, instr_ip);

        // IP for the operands *in the full bytecode* of the function (not the sliced loop bytecode).
        let operands_absolute_ip = start.0 + cx.ip as u32;

        macro_rules! emit_stub {
            () => {{
                emit_stub_for_instr(&mut buffer, instr, operands_absolute_ip, &prologue_data);
            }};
        }

        match instr {
            Instruction::JmpFalseP => {
                let JmpFalsePopOperands(ConditionalJumpPopOperands { offset, value: _ }) =
                    extract_back_infallible(&mut cx);
                emit_conditional_jump(
                    &mut cx,
                    &mut buffer,
                    &prologue_data,
                    &mut caller_regs,
                    offset,
                    CheckCondition::Truthy,
                    false,
                    true,
                );
            }
            Instruction::JmpFalseNP => {
                let JmpFalseNoPopOperands(ConditionalJumpNoPopOperands { offset, value: _ }) =
                    extract_back_infallible(&mut cx);
                emit_conditional_jump(
                    &mut cx,
                    &mut buffer,
                    &prologue_data,
                    &mut caller_regs,
                    offset,
                    CheckCondition::Truthy,
                    false,
                    false,
                );
            }
            Instruction::JmpTrueP => {
                let JmpTruePopOperands(ConditionalJumpPopOperands { offset, value: _ }) =
                    extract_back_infallible(&mut cx);
                emit_conditional_jump(
                    &mut cx,
                    &mut buffer,
                    &prologue_data,
                    &mut caller_regs,
                    offset,
                    CheckCondition::Truthy,
                    true,
                    true,
                );
            }
            Instruction::JmpTrueNP => {
                let JmpTrueNoPopOperands(ConditionalJumpNoPopOperands { offset, value: _ }) =
                    extract_back_infallible(&mut cx);
                emit_conditional_jump(
                    &mut cx,
                    &mut buffer,
                    &prologue_data,
                    &mut caller_regs,
                    offset,
                    CheckCondition::Truthy,
                    true,
                    false,
                );
            }
            Instruction::JmpNullishNP => {
                let JmpNullishNoPopOperands(ConditionalJumpNoPopOperands { offset, value: _ }) =
                    extract_back_infallible(&mut cx);
                emit_conditional_jump(
                    &mut cx,
                    &mut buffer,
                    &prologue_data,
                    &mut caller_regs,
                    offset,
                    CheckCondition::Nullish,
                    true,
                    false,
                );
            }
            Instruction::JmpNullishP => {
                let JmpNullishPopOperands(ConditionalJumpPopOperands { offset, value: _ }) =
                    extract_back_infallible(&mut cx);
                emit_conditional_jump(
                    &mut cx,
                    &mut buffer,
                    &prologue_data,
                    &mut caller_regs,
                    offset,
                    CheckCondition::Nullish,
                    true,
                    true,
                );
            }
            Instruction::JmpUndefinedNP => {
                let JmpUndefinedNoPopOperands(ConditionalJumpNoPopOperands { offset, value: _ }) =
                    extract_back_infallible(&mut cx);
                emit_conditional_jump(
                    &mut cx,
                    &mut buffer,
                    &prologue_data,
                    &mut caller_regs,
                    offset,
                    CheckCondition::Undefined,
                    true,
                    false,
                );
            }
            Instruction::JmpUndefinedP => {
                let JmpUndefinedPopOperands(ConditionalJumpPopOperands { offset, value: _ }) =
                    extract_back_infallible(&mut cx);
                emit_conditional_jump(
                    &mut cx,
                    &mut buffer,
                    &prologue_data,
                    &mut caller_regs,
                    offset,
                    CheckCondition::Undefined,
                    true,
                    true,
                );
            }
            Instruction::Jmp => {
                let JmpOperands(offset) = extract_back_infallible(&mut cx);
                let target_bc_ip = target_from_relative(cx.ip as u32, offset);
                jmp_bytecode_ip(&mut buffer, target_bc_ip);
            }
            Instruction::LoopBackJmp => {
                let LoopBackjumpOperands { offset, hotness: _ } = extract_back_infallible(&mut cx);
                let target_bc_ip = target_from_relative(cx.ip as u32, offset);
                jmp_bytecode_ip(&mut buffer, target_bc_ip);
            }
            Instruction::LdLocal => {
                let LdLocalOperands(_) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::IntrinsicOp => {
                let IntrinsicOperands(_) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::Pop => {
                let PopOperands(_) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::Add => {
                let AddOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::Sub => {
                let SubOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::Mul => {
                let MulOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::Div => {
                let DivOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::Rem => {
                let RemOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::Pow => {
                let PowOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::Gt => {
                let GtOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::Ge => {
                let GeOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::Lt => {
                let LtOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::Le => {
                let LeOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::Eq => {
                let EqOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::Ne => {
                let NeOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::LdGlobal => {
                let LdGlobalOperands(_) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::String => {
                let StringConstantOperands(_) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::Boolean => {
                let BooleanConstantOperands(_) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::Number => {
                let NumberConstantOperands(_) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::Regex => {
                let RegexConstantOperands(_) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::Null => emit_stub!(),
            Instruction::Undefined => emit_stub!(),
            Instruction::Function => {
                let FunctionConstantOperands(_) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::Pos => {
                let PosOperands(_) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::Neg => {
                let NegOperands(_) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::TypeOf => {
                let TypeofOperands { value: _ } = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::TypeOfGlobalIdent => {
                let TypeofIdentOperands(_) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::BitNot => {
                let BitnotOperands(_) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::Not => {
                let NotOperands(_) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::StoreLocal => {
                let StoreLocalOperands { local: _, kind: _ } = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::StoreGlobal => {
                let StoreGlobalOperands { name: _, kind: _ } = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::Ret => {
                let RetOperands { tc_depth: _, value: _ } = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::Call => {
                let CallOperands {
                    argc: _,
                    has_this: _,
                    function_call_kind: _,
                    spread_indices,
                } = extract_back_infallible(&mut cx);
                exhaust!(spread_indices, &mut cx);
                emit_stub!();
            }
            Instruction::StaticPropAccess => {
                let StaticPropertyAccessOperands { ident: _, target: _ } = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::DynamicPropAccess => {
                let DynamicPropertyAccessOperands { key: _, target: _ } = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::ArrayLit => {
                let ArrayLiteralOperands { members, dense: _ } = extract_back_infallible(&mut cx);
                exhaust!(members, &mut cx);
                emit_stub!();
            }
            Instruction::ObjLit => {
                let ObjectLiteralOperands { members } = extract_back_infallible(&mut cx);
                exhaust!(members, &mut cx);
                emit_stub!();
            }
            Instruction::BindThis => {
                let BindThisOperands { value: _ } = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::This => emit_stub!(),
            Instruction::StaticPropAssign => {
                let StaticPropertyAssignOperands {
                    kind: _,
                    target: _,
                    key: _,
                } = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::DynamicPropAssign => {
                let DynamicPropertyAssignOperands {
                    kind: _,
                    key: _,
                    target: _,
                } = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::LdLocalExt => {
                let LdLocalExtOperands(_) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::StoreLocalExt => {
                let StoreLocalExtOperands { local: _, kind: _ } = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::StrictEq => {
                let StrictEqOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::StrictNe => {
                let StrictNeOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::PopTry => emit_stub!(),
            Instruction::FinallyEnd => {
                let FinallyEndOperands(_) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::Throw => {
                let ThrowOperands { value: _ } = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::Yield => {
                let YieldOperands { value: _ } = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::BitOr => {
                let BitorOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::BitXor => {
                let BitxorOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::BitAnd => {
                let BitandOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::BitShl => {
                let BitshlOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::BitShr => {
                let BitshrOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::BitUshr => {
                let BitushrOperands(BinaryOperator { left: _, right: _ }) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::ObjIn => {
                let ObjectInOperands { target: _, key: _ } = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::InstanceOf => {
                let InstanceofOperands {
                    constructor: _,
                    value: _,
                } = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::ImportDyn => {
                let ImportDynOperands { value: _ } = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::ImportStatic => {
                let ImportStaticOperands {
                    kind: _,
                    local: _,
                    path: _,
                } = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::ExportDefault => {
                let ExportDefaultOperands { value: _ } = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::ExportNamed => {
                let ExportNamedOperands { members: _ } = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::Debugger => emit_stub!(),
            Instruction::Global => emit_stub!(),
            Instruction::Super => emit_stub!(),
            Instruction::Undef => emit_stub!(),
            Instruction::Await => {
                let AwaitOperands { value: _ } = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::Nan => emit_stub!(),
            Instruction::Infinity => emit_stub!(),
            Instruction::CallSymbolIterator => {
                let CallSymbolIteratorOperands { value: _ } = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::CallForInIterator => {
                let ForInIteratorOperands { value: _ } = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::DeletePropertyStatic => {
                let DeletePropertyStaticOperands { target: _, key: _ } = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::DeletePropertyDynamic => {
                let DeletePropertyDynamicOperands { target: _, key: _ } = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::ObjDestruct => {
                let ObjectDestructuringOperands {
                    rest_local_id: _,
                    target: _,
                    members,
                } = extract_back_infallible(&mut cx);
                exhaust!(members, &mut cx);
                emit_stub!();
            }
            Instruction::ArrayDestruct => {
                let ArrayDestructuringOperands { array: _, members } = extract_back_infallible(&mut cx);
                exhaust!(members, &mut cx);
                emit_stub!();
            }
            Instruction::AssignProperties => {
                let AssignPropertiesOperands { members, target: _ } = extract_back_infallible(&mut cx);
                exhaust!(members, &mut cx);
                emit_stub!();
            }
            Instruction::DelayedReturn => {
                let DelayedRetOperands(_) = extract_back_infallible(&mut cx);
                emit_stub!();
            }
            Instruction::NewTarget => emit_stub!(),
            Instruction::Nop => emit_stub!(),
            Instruction::Try => return Err(CompileError::UnhandledInstruction(instr)),
        }
    }

    // Exit branch (end-of-loop/end-of-bytecode)
    assert!(cx.ip == bytecode.len());
    assert!(start.0 + cx.ip as u32 == end.0);
    mark_bytecode_ip(&mut buffer, Ip(cx.ip as u32));
    mov_reg_mem_u8(
        &mut buffer,
        REG_RETURN64,
        REG_STACK_RELATIVE,
        -prologue_data.out_data_rbp_offset,
    );
    mov_mem_imm32(
        &mut buffer,
        REG_RETURN64,
        offset_of!(JitOutData, ip).try_into().unwrap(),
        end.0.cast_signed(),
    );
    mov_reg_imm32(&mut buffer, REG_RETURN32, 0);

    // Epilogue
    mark_internal_label(&mut buffer, InternalLabel::Epilogue);
    epilogue(&mut buffer, &prologue_data);

    mark_internal_label(&mut buffer, InternalLabel::StubStatusHandler);
    jmp_internal_label(&mut buffer, InternalLabel::Epilogue);

    Ok(MmapFn::alloc(buffer.buffer()))
}
