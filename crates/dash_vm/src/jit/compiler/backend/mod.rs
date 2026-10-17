#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod x86;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use x86::{
    PrologueData, REG_ARG1, REG_ARG2, REG_ARG3, REG_ARG4, REG_RETURN32, REG_RETURN64, REG_STACK_RELATIVE, Register,
    call_reg, cmp_reg_al_imm8, codegen_epilogue, codegen_prologue, je_block, je_synthetic_label, jmp_block,
    jmp_synthetic_label, jne_block, jne_synthetic_label, mov_mem_imm32, mov_reg_imm32, mov_reg_mem_u8, mov_reg_reg,
    register_synthetic_block, register_user_block, test_reg_reg, usable_callee_saved_regs, usable_caller_saved_regs,
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
