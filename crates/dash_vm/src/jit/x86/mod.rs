mod modrm {
    pub const MOD_R: u8 = 0b11_000_000;
    pub const MOD_M8: u8 = 0b01_000_000;
}

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
    pub const JNE_REL32: u8 = 0x85;
    pub const JMP_REL32: u8 = 0xE9;
    pub const ADD_RM_IMM8: u8 = 0x83;
    pub const SUB_RM_IMM8: u8 = 0x83;
}

use crate::frame::Ip;
use crate::jit::jumpresolver::{InternalLabel, JumpResolver, PatchSite, X86Ip};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register {
    Al,
    Rax,
    Eax,
    R8,
    Rbp,
    Rsp,
    R12,
    R13,
    R14,
    Rdi,
    Rsi,
    Rdx,
}

impl Register {
    pub fn is_64_bit(&self) -> bool {
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
            | Register::Rdx => true,
            Register::Eax | Register::Al => false,
        }
    }

    pub fn reg_field(&self) -> u8 {
        match self {
            Register::Rax | Register::Eax | Register::Al | Register::R8 => 0,
            Register::Rbp | Register::R13 => 5,
            Register::Rsp | Register::R12 => 4,
            Register::Rdi => 7,
            Register::Rsi | Register::R14 => 6,
            Register::Rdx => 2,
        }
    }

    pub fn needs_rex_prefix(&self) -> bool {
        match self {
            Register::R8 | Register::R12 | Register::R13 | Register::R14 => true,
            Register::Rax
            | Register::Eax
            | Register::Rbp
            | Register::Rsp
            | Register::Rdi
            | Register::Rsi
            | Register::Rdx
            | Register::Al => false,
        }
    }
}

pub struct Emitter {
    buffer: Vec<u8>,
    jumps: JumpResolver,
}

impl Emitter {
    pub fn new(bytecode_len: usize) -> Self {
        Self {
            buffer: Vec::new(),
            jumps: JumpResolver::new(bytecode_len),
        }
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    pub fn offset(&self) -> X86Ip {
        self.buffer.len().try_into().unwrap()
    }

    fn patch_rel32(&mut self, patch_site: PatchSite, target: X86Ip) {
        let patch_site = patch_site as usize;
        let disp = target as i64 - (patch_site as i64 + 4);
        let disp = i32::try_from(disp).expect("rel32 displacement out of range");
        self.buffer[patch_site..patch_site + 4].copy_from_slice(&disp.to_le_bytes());
    }

    fn patch_sites_to(&mut self, patch_sites: Vec<PatchSite>, target: X86Ip) {
        for patch_site in patch_sites {
            self.patch_rel32(patch_site, target);
        }
    }

    pub fn mark_bytecode_ip(&mut self, bc_ip: Ip) {
        let x86_ip = self.offset();
        let patch_sites = self.jumps.resolve_user_label(bc_ip, x86_ip);
        self.patch_sites_to(patch_sites, x86_ip);
    }

    pub fn mark_internal_label(&mut self, label: InternalLabel) {
        let x86_ip = self.offset();
        let patch_sites = self.jumps.resolve_internal_label(label, x86_ip);
        self.patch_sites_to(patch_sites, x86_ip);
    }

    pub fn mov_reg_imm32(&mut self, reg: Register, imm: u32) {
        if reg.needs_rex_prefix() {
            self.buffer.push(rex::BASE | rex::B);
        }
        self.buffer.push(opcodes::MOV_REG_IMM32 + reg.reg_field());
        self.buffer.extend_from_slice(&imm.to_le_bytes());
    }

    pub fn ret(&mut self) {
        self.buffer.push(opcodes::RET);
    }

    pub fn push(&mut self, reg: Register) {
        if reg.needs_rex_prefix() {
            self.buffer.push(rex::BASE | rex::B);
        }
        self.buffer.push(opcodes::PUSH_REG + reg.reg_field());
    }

    pub fn pop(&mut self, reg: Register) {
        if reg.needs_rex_prefix() {
            self.buffer.push(rex::BASE | rex::B);
        }
        self.buffer.push(opcodes::POP_REG + reg.reg_field());
    }

    pub fn add_rsp_imm8(&mut self, imm: u8) {
        self.buffer.push(rex::BASE | rex::W);
        self.buffer.push(opcodes::ADD_RM_IMM8);
        let modrm = modrm::MOD_R | (0 << 3) | Register::Rsp.reg_field();
        self.buffer.push(modrm);
        self.buffer.push(imm as u8);
    }

    pub fn sub_rsp_imm8(&mut self, imm: u8) {
        self.buffer.push(rex::BASE | rex::W);
        self.buffer.push(opcodes::SUB_RM_IMM8);
        let modrm = modrm::MOD_R | (5 << 3) | Register::Rsp.reg_field();
        self.buffer.push(modrm);
        self.buffer.push(imm as u8);
    }

    pub fn mov_reg_reg(&mut self, dest: Register, src: Register) {
        let mut rex = 0;

        if dest.needs_rex_prefix() {
            rex |= rex::B;
        }
        if src.needs_rex_prefix() {
            rex |= rex::R;
        }

        let is_64bit = dest.is_64_bit();
        assert!(
            src.is_64_bit() == is_64bit,
            "Source and destination registers must be of the same size"
        );

        if is_64bit {
            rex |= rex::W;
        }

        if rex != 0 {
            self.buffer.push(rex::BASE | rex);
        }

        self.buffer.push(opcodes::MOV_RM_R);

        let modrm = modrm::MOD_R | (src.reg_field() << 3) | dest.reg_field();
        self.buffer.push(modrm);
    }

    pub fn mov_reg_mem_u8(&mut self, dest: Register, base: Register, offset: u8) {
        let mut rex = 0;
        if base.needs_rex_prefix() {
            rex |= rex::B;
        }
        if dest.needs_rex_prefix() {
            rex |= rex::R;
        }
        let is_64bit = dest.is_64_bit();
        assert!(
            base.is_64_bit() == is_64bit,
            "Source and destination registers must be of the same size"
        );

        if is_64bit {
            rex |= rex::W;
        }

        if rex != 0 {
            self.buffer.push(rex::BASE | rex);
        }

        self.buffer.push(opcodes::MOV_R_RM);

        let modrm = modrm::MOD_M8 | (dest.reg_field() << 3) | base.reg_field();
        self.buffer.push(modrm);

        self.buffer.extend_from_slice(&offset.to_le_bytes());
    }

    pub fn call_reg(&mut self, register: Register) {
        let mut rex = 0;
        if register.needs_rex_prefix() {
            rex |= rex::B;
        }

        if rex != 0 {
            self.buffer.push(rex::BASE | rex);
        }

        self.buffer.push(0xFF);

        let modrm = modrm::MOD_R | (2 << 3) | register.reg_field();
        self.buffer.push(modrm);
    }

    pub fn cmp_reg_al_imm8(&mut self, imm: u8) {
        self.buffer.push(opcodes::CMP_AL_IMM8);
        self.buffer.push(imm);
    }

    pub fn jne_imm32(&mut self, offset: i32) {
        self.buffer.push(0x0f);
        self.buffer.push(opcodes::JNE_REL32);
        self.buffer.extend_from_slice(&offset.to_le_bytes());
    }

    pub fn test_reg_reg(&mut self, reg1: Register, reg2: Register) {
        let mut rex = 0;
        if reg1.needs_rex_prefix() {
            rex |= rex::R;
        }
        if reg2.needs_rex_prefix() {
            rex |= rex::B;
        }

        let is_64bit = reg1.is_64_bit();
        assert!(
            reg2.is_64_bit() == is_64bit,
            "Source and destination registers must be of the same size"
        );

        if is_64bit {
            rex |= rex::W;
        }

        if rex != 0 {
            self.buffer.push(rex::BASE | rex);
        }

        self.buffer.push(opcodes::TEST_R_R);

        let modrm = modrm::MOD_R | (reg1.reg_field() << 3) | reg2.reg_field();
        self.buffer.push(modrm);
    }

    pub fn jne_bytecode_ip(&mut self, target_bc_ip: Ip) {
        let patch_site = self.offset() + 2;
        if let Some(target_x86_ip) = self.jumps.add_user_reference(target_bc_ip, patch_site) {
            self.jne_imm32(0);
            self.patch_rel32(patch_site, target_x86_ip);
        } else {
            self.jne_imm32(0);
        }
    }

    pub fn jne_internal_label(&mut self, label: InternalLabel) {
        let patch_site = self.offset() + 2;
        self.jne_imm32(0);
        if let Some(target_x86_ip) = self.jumps.add_internal_reference(label, patch_site) {
            self.patch_rel32(patch_site, target_x86_ip);
        }
    }

    pub fn jmp_internal_label(&mut self, label: InternalLabel) {
        let patch_site = self.offset() + 1;
        self.jmp_imm32(0);
        if let Some(target_x86_ip) = self.jumps.add_internal_reference(label, patch_site) {
            self.patch_rel32(patch_site, target_x86_ip);
        }
    }

    pub fn jmp_imm32(&mut self, offset: i32) {
        self.buffer.push(opcodes::JMP_REL32);
        self.buffer.extend_from_slice(&offset.to_le_bytes());
    }

    pub fn jmp_bytecode_ip(&mut self, target_bc_ip: Ip) {
        let patch_site = self.offset() + 1;
        if let Some(target_x86_ip) = self.jumps.add_user_reference(target_bc_ip, patch_site) {
            self.jmp_imm32(0);
            self.patch_rel32(patch_site, target_x86_ip);
        } else {
            self.jmp_imm32(0);
        }
    }
}
