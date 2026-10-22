#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod x86;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use x86::{
    PrologueData, REG_ARG1, REG_ARG2, REG_ARG3, REG_ARG4, REG_RETURN32, REG_RETURN64, REG_STACK_RELATIVE, Scale,
    addsd_reg_mem_sib, call_reg, cmp_reg_al_imm8, codegen_epilogue, codegen_prologue, decq_mem, je_block,
    je_synthetic_label, jmp_block, jmp_synthetic_label, jne_block, jne_synthetic_label, mov_mem_imm32,
    mov_mem_reg_scaled, mov_reg_imm32, mov_reg_mem, mov_reg_reg, movsd_mem_sib_reg, movsd_reg_mem_sib,
    register::Register, register::usable_callee_saved_regs, register::usable_caller_saved_gprs, register::usable_fprs,
    register_synthetic_block, register_user_block, seta_reg, test_reg_reg, ucomisd_reg_mem_sib, xor_reg_reg,
};

use crate::jit::compiler::codebuffer::CodeBuffer;
use crate::jit::compiler::jumpresolver::SyntheticLabel;
use crate::jit::ir::BranchTarget;

pub fn je_branch_target(buf: &mut CodeBuffer, target: BranchTarget) {
    match target {
        BranchTarget::Block(block) => je_block(buf, block),
        BranchTarget::Break => je_synthetic_label(buf, SyntheticLabel::Exit),
    }
}

pub fn jmp_branch_target(buf: &mut CodeBuffer, target: BranchTarget) {
    match target {
        BranchTarget::Block(block) => jmp_block(buf, block),
        BranchTarget::Break => jmp_synthetic_label(buf, SyntheticLabel::Exit),
    }
}
