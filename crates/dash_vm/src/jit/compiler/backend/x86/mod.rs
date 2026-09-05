mod modrm {
    pub const MOD_R: u8 = 0b11_000_000;
    pub const MOD_M8: u8 = 0b01_000_000;
}

#[expect(dead_code)]
mod rex {
    pub const W: u8 = 0x08;
    pub const R: u8 = 0x04;
    pub const X: u8 = 0x02;
    pub const B: u8 = 0x01;

    pub const BASE: u8 = 0x40;
}

mod opcodes {
    pub const MOV_REG_IMM32: u8 = 0xB8;
    pub const MOV_RM_R: u8 = 0x89;
    pub const MOV_R_RM: u8 = 0x8B;
    pub const RET: u8 = 0xC3;
    pub const PUSH_REG: u8 = 0x50;
    pub const POP_REG: u8 = 0x58;
    pub const CMP_AL_IMM8: u8 = 0x3C;
    pub const TEST_R_R: u8 = 0x85;
    pub const JE_REL32: u8 = 0x84;
    pub const JNE_REL32: u8 = 0x85;
    pub const JMP_REL32: u8 = 0xE9;
    pub const ADD_RM_IMM8: u8 = 0x83;
    pub const SUB_RM_IMM8: u8 = 0x83;
    pub const LEA: u8 = 0x8D;
    pub const MOV_MEM_IMM32: u8 = 0xC7;
}

use std::mem::offset_of;

use crate::frame::Ip;
use crate::jit::JitVtable;
use crate::jit::compiler::codebuffer::CodeBuffer;
use crate::jit::jumpresolver::{CpuIp, InternalLabel, PatchSite};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[expect(dead_code)]
pub enum Register {
    Al,
    Rax,
    Eax,
    R8,
    Rbp,
    Rsp,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
    Rdi,
    Rsi,
    Rdx,
    Rcx,
    Rbx,
}

impl Register {
    pub fn size_bits(self) -> u32 {
        match self {
            Register::Rax
            | Register::R8
            | Register::Rbp
            | Register::R12
            | Register::R13
            | Register::Rsp
            | Register::Rdi
            | Register::Rsi
            | Register::R14
            | Register::R15
            | Register::Rdx
            | Register::Rcx
            | Register::Rbx
            | Register::R9
            | Register::R10
            | Register::R11 => 64,
            Register::Eax => 32,
            Register::Al => 8,
        }
    }

    /// Require that the size of the register is the same as the size of the other register, and return it.
    pub fn join_register_size(self, other: Register) -> u32 {
        assert_eq!(
            self.size_bits(),
            other.size_bits(),
            "Register {self:?} of size {} does not match register {other:?} of size {}",
            self.size_bits(),
            other.size_bits()
        );
        self.size_bits()
    }

    pub fn reg_field(self) -> u8 {
        match self {
            Register::Rax | Register::Eax | Register::Al | Register::R8 => 0,
            Register::Rcx | Register::R9 => 1,
            Register::Rdx | Register::R10 => 2,
            Register::Rbx | Register::R11 => 3,
            Register::Rsp | Register::R12 => 4,
            Register::Rbp | Register::R13 => 5,
            Register::Rsi | Register::R14 => 6,
            Register::Rdi | Register::R15 => 7,
        }
    }

    pub fn needs_rex_prefix(self) -> bool {
        match self {
            Register::R8
            | Register::R9
            | Register::R10
            | Register::R11
            | Register::R12
            | Register::R13
            | Register::R14
            | Register::R15 => true,
            Register::Rax
            | Register::Eax
            | Register::Rbp
            | Register::Rsp
            | Register::Rdi
            | Register::Rsi
            | Register::Rdx
            | Register::Al
            | Register::Rcx
            | Register::Rbx => false,
        }
    }
}

pub fn usable_callee_saved_regs() -> Vec<Register> {
    vec![
        Register::Rbx,
        Register::R12,
        Register::R13,
        Register::R14,
        Register::R15,
    ]
}
pub fn usable_caller_saved_regs() -> Vec<Register> {
    vec![
        Register::Rax,
        Register::Rcx,
        Register::Rdx,
        Register::Rsi,
        Register::Rdi,
        Register::R8,
        Register::R9,
        Register::R10,
        Register::R11,
    ]
}

pub const REG_ARG1: Register = Register::Rdi;
pub const REG_ARG2: Register = Register::Rsi;
pub const REG_ARG3: Register = Register::Rdx;
pub const REG_ARG4: Register = Register::Rcx;
pub const REG_STACK_RELATIVE: Register = Register::Rbp;
pub const REG_RETURN64: Register = Register::Rax;
pub const REG_RETURN32: Register = Register::Eax;

pub struct PrologueData {
    pub stub_csreg: Register,
    pub vtable_csreg: Register,
    pub vm_csreg: Register,
    pub out_data_rbp_offset: i8,
}

pub fn prologue(buf: &mut CodeBuffer, callee_regs: &mut Vec<Register>) -> PrologueData {
    let stub_csreg = callee_regs.pop().unwrap();
    let vtable_csreg = callee_regs.pop().unwrap();
    let vm_csreg = callee_regs.pop().unwrap();

    // START OF STACK
    push(buf, Register::Rbp); // rsp aligned
    mov_reg_reg(buf, Register::Rbp, Register::Rsp);
    push(buf, vm_csreg); // rbp-8, rsp misaligned by 8
    mov_reg_reg(buf, vm_csreg, Register::Rdi);
    push(buf, stub_csreg); // rbp-16, rsp aligned
    push(buf, vtable_csreg); // rbp-24, rsp misaligned by 8
    mov_reg_reg(buf, vtable_csreg, Register::Rsi);
    push(buf, Register::Rdx); // Out data - rbp-32, rsp aligned (exact reg being pushed doesn't matter - only need to make space)
    // END OF STACK

    // The stub fn is very hot, so put it in a callee-saved register
    mov_reg_mem_u8(
        buf,
        stub_csreg,
        Register::Rsi,
        offset_of!(JitVtable, stub_fn).try_into().unwrap(),
    );

    PrologueData {
        stub_csreg,
        vtable_csreg,
        vm_csreg,
        out_data_rbp_offset: 32,
    }
}

pub fn epilogue(buf: &mut CodeBuffer, prologue_data: &PrologueData) {
    add_rsp_imm8(buf, 8); // rdx - not a callee saved register and we have the return payload in that reg already, so just discard directly
    pop(buf, prologue_data.vtable_csreg);
    pop(buf, prologue_data.stub_csreg);
    pop(buf, prologue_data.vm_csreg);
    pop(buf, Register::Rbp);
    ret(buf);
}

pub fn mov_reg_reg(buf: &mut CodeBuffer, dest: Register, src: Register) {
    let mut rex = 0;

    if dest.needs_rex_prefix() {
        rex |= rex::B;
    }
    if src.needs_rex_prefix() {
        rex |= rex::R;
    }
    if dest.join_register_size(src) == 64 {
        rex |= rex::W;
    }

    if rex != 0 {
        buf.push(rex::BASE | rex);
    }

    buf.push(opcodes::MOV_RM_R);

    let modrm = modrm::MOD_R | (src.reg_field() << 3) | dest.reg_field();
    buf.push(modrm);
}

pub fn push(buf: &mut CodeBuffer, reg: Register) {
    if reg.needs_rex_prefix() {
        buf.push(rex::BASE | rex::B);
    }
    buf.push(opcodes::PUSH_REG + reg.reg_field());
}

pub fn mov_reg_mem_u8(buf: &mut CodeBuffer, dest: Register, base: Register, offset: i8) {
    let mut rex = 0;
    if base.needs_rex_prefix() {
        rex |= rex::B;
    }
    if dest.needs_rex_prefix() {
        rex |= rex::R;
    }
    if dest.join_register_size(base) == 64 {
        rex |= rex::W;
    }

    if rex != 0 {
        buf.push(rex::BASE | rex);
    }

    buf.push(opcodes::MOV_R_RM);

    let modrm = modrm::MOD_M8 | (dest.reg_field() << 3) | base.reg_field();
    buf.push(modrm);

    buf.push(offset as u8);
}

pub fn mov_reg_imm32(buf: &mut CodeBuffer, reg: Register, imm: u32) {
    if reg.needs_rex_prefix() {
        buf.push(rex::BASE | rex::B);
    }
    buf.push(opcodes::MOV_REG_IMM32 + reg.reg_field());
    buf.extend_from_slice(&imm.to_le_bytes());
}

pub fn call_reg(buf: &mut CodeBuffer, register: Register) {
    let mut rex = 0;
    if register.needs_rex_prefix() {
        rex |= rex::B;
    }

    if rex != 0 {
        buf.push(rex::BASE | rex);
    }

    buf.push(0xFF);

    let modrm = modrm::MOD_R | (2 << 3) | register.reg_field();
    buf.push(modrm);
}

pub fn test_reg_reg(buf: &mut CodeBuffer, reg1: Register, reg2: Register) {
    let mut rex = 0;
    if reg1.needs_rex_prefix() {
        rex |= rex::R;
    }
    if reg2.needs_rex_prefix() {
        rex |= rex::B;
    }
    if reg1.join_register_size(reg2) == 64 {
        rex |= rex::W;
    }

    if rex != 0 {
        buf.push(rex::BASE | rex);
    }

    buf.push(opcodes::TEST_R_R);

    let modrm = modrm::MOD_R | (reg1.reg_field() << 3) | reg2.reg_field();
    buf.push(modrm);
}

pub fn mov_mem_imm32(buf: &mut CodeBuffer, base: Register, offset: u8, imm: i32) {
    buf.push(opcodes::MOV_MEM_IMM32);
    let modrm = modrm::MOD_M8 | (0 << 3) | base.reg_field();
    buf.push(modrm);
    buf.push(offset);
    buf.extend_from_slice(&imm.to_le_bytes());
}

pub fn ret(buf: &mut CodeBuffer) {
    buf.push(opcodes::RET);
}

pub fn pop(buf: &mut CodeBuffer, reg: Register) {
    if reg.needs_rex_prefix() {
        buf.push(rex::BASE | rex::B);
    }
    buf.push(opcodes::POP_REG + reg.reg_field());
}

pub fn add_rsp_imm8(buf: &mut CodeBuffer, imm: u8) {
    buf.push(rex::BASE | rex::W);
    buf.push(opcodes::ADD_RM_IMM8);
    let modrm = modrm::MOD_R | (0 << 3) | Register::Rsp.reg_field();
    buf.push(modrm);
    buf.push(imm as u8);
}

#[expect(dead_code)]
pub fn sub_rsp_imm8(buf: &mut CodeBuffer, imm: u8) {
    buf.push(rex::BASE | rex::W);
    buf.push(opcodes::SUB_RM_IMM8);
    let modrm = modrm::MOD_R | (5 << 3) | Register::Rsp.reg_field();
    buf.push(modrm);
    buf.push(imm as u8);
}

pub fn cmp_reg_al_imm8(buf: &mut CodeBuffer, imm: u8) {
    buf.push(opcodes::CMP_AL_IMM8);
    buf.push(imm);
}

pub fn jne_imm32(buf: &mut CodeBuffer, offset: i32) {
    buf.push(0x0f);
    buf.push(opcodes::JNE_REL32);
    buf.extend_from_slice(&offset.to_le_bytes());
}

pub fn je_imm32(buf: &mut CodeBuffer, offset: i32) {
    buf.push(0x0f);
    buf.push(opcodes::JE_REL32);
    buf.extend_from_slice(&offset.to_le_bytes());
}

pub fn jne_bytecode_ip(buf: &mut CodeBuffer, target_bc_ip: Ip) {
    let patch_site = buf.offset() + 2;
    if let Some(target_x86_ip) = buf.jumps.add_user_reference(target_bc_ip, patch_site) {
        jne_imm32(buf, 0);
        patch_rel32(buf, patch_site, target_x86_ip);
    } else {
        jne_imm32(buf, 0);
    }
}

pub fn je_bytecode_ip(buf: &mut CodeBuffer, target_bc_ip: Ip) {
    let patch_site = buf.offset() + 2;
    if let Some(target_x86_ip) = buf.jumps.add_user_reference(target_bc_ip, patch_site) {
        je_imm32(buf, 0);
        patch_rel32(buf, patch_site, target_x86_ip);
    } else {
        je_imm32(buf, 0);
    }
}

pub fn jne_internal_label(buf: &mut CodeBuffer, label: InternalLabel) {
    let patch_site = buf.offset() + 2;
    jne_imm32(buf, 0);
    if let Some(target_x86_ip) = buf.jumps.add_internal_reference(label, patch_site) {
        patch_rel32(buf, patch_site, target_x86_ip);
    }
}

pub fn jmp_internal_label(buf: &mut CodeBuffer, label: InternalLabel) {
    let patch_site = buf.offset() + 1;
    jmp_imm32(buf, 0);
    if let Some(target_x86_ip) = buf.jumps.add_internal_reference(label, patch_site) {
        patch_rel32(buf, patch_site, target_x86_ip);
    }
}

pub fn jmp_imm32(buf: &mut CodeBuffer, offset: i32) {
    buf.push(opcodes::JMP_REL32);
    buf.extend_from_slice(&offset.to_le_bytes());
}

pub fn jmp_bytecode_ip(buf: &mut CodeBuffer, target_bc_ip: Ip) {
    let patch_site = buf.offset() + 1;
    if let Some(target_x86_ip) = buf.jumps.add_user_reference(target_bc_ip, patch_site) {
        jmp_imm32(buf, 0);
        patch_rel32(buf, patch_site, target_x86_ip);
    } else {
        jmp_imm32(buf, 0);
    }
}

#[expect(dead_code)]
pub fn lea_reg_mem(buf: &mut CodeBuffer, dest: Register, base: Register, offset: i8) {
    let mut rex = 0;
    if dest.needs_rex_prefix() {
        rex |= rex::B;
    }
    if base.needs_rex_prefix() {
        rex |= rex::R;
    }
    if dest.join_register_size(base) == 64 {
        rex |= rex::W;
    }
    if rex != 0 {
        buf.push(rex::BASE | rex);
    }
    buf.push(opcodes::LEA);
    let modrm = modrm::MOD_M8 | (dest.reg_field() << 3) | base.reg_field();
    buf.push(modrm);
    buf.push(offset as u8);
}

fn patch_rel32(buf: &mut CodeBuffer, patch_site: PatchSite, target: CpuIp) {
    let patch_site = patch_site as usize;
    let disp = target as i64 - (patch_site as i64 + 4);
    let disp = i32::try_from(disp).expect("rel32 displacement out of range");
    buf.buffer[patch_site..patch_site + 4].copy_from_slice(&disp.to_le_bytes());
}

fn patch_sites_to(buf: &mut CodeBuffer, patch_sites: Vec<PatchSite>, target: CpuIp) {
    for patch_site in patch_sites {
        patch_rel32(buf, patch_site, target);
    }
}

pub fn mark_bytecode_ip(buf: &mut CodeBuffer, bc_ip: Ip) {
    let x86_ip = buf.offset();
    let patch_sites = buf.jumps.resolve_user_label(bc_ip, x86_ip);
    patch_sites_to(buf, patch_sites, x86_ip);
}

pub fn mark_internal_label(buf: &mut CodeBuffer, label: InternalLabel) {
    let x86_ip = buf.offset();
    let patch_sites = buf.jumps.resolve_internal_label(label, x86_ip);
    patch_sites_to(buf, patch_sites, x86_ip);
}
