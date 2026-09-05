#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod x86;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use x86::{
    PrologueData, REG_ARG1, REG_ARG2, REG_ARG3, REG_ARG4, REG_RETURN32, REG_RETURN64, REG_STACK_RELATIVE, Register,
    call_reg, cmp_reg_al_imm8, epilogue, je_bytecode_ip, jmp_bytecode_ip, jmp_internal_label, jne_bytecode_ip,
    jne_internal_label, mark_bytecode_ip, mark_internal_label, mov_mem_imm32, mov_reg_imm32, mov_reg_mem_u8,
    mov_reg_reg, prologue, test_reg_reg, usable_callee_saved_regs, usable_caller_saved_regs,
};
