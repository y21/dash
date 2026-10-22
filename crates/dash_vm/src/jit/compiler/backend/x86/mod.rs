mod modrm {
    pub const MOD_R: u8 = 0b11_000_000;
    pub const MOD_M8: u8 = 0b01_000_000;
    pub const MOD_M8_SIB: u8 = 0b01_000_100;
}

#[expect(dead_code)]
mod rex {
    // TODO: remove these in favor of the Rex enum
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
    pub const MOVSD_REG_RM: &[u8] = &[0x0F, 0x10];
    pub const ADDSD_REG_RM: &[u8] = &[0x0F, 0x58];
    pub const MOVSD_RM_REG: &[u8] = &[0x0F, 0x11];
    pub const UCOMISD_REG_RM: &[u8] = &[0x0F, 0x2E];
    pub const DEC_RM64: u8 = 0xFF;
    pub const CALL_RM64: u8 = 0xFF;
    pub const XOR: u8 = 0x31;
    pub const SETA_RM8: &[u8] = &[0x0F, 0x97];
}

use std::mem::offset_of;
use std::ops::BitOrAssign;

use arrayvec::ArrayVec;
use dash_middle::indexvec::IndexVec;

use crate::frame::Ip;
use crate::jit::JitVtable;
use crate::jit::compiler::backend::x86::register::Register;
use crate::jit::compiler::codebuffer::CodeBuffer;
use crate::jit::compiler::jumpresolver::{CpuIp, PatchData, SyntheticLabel};
use crate::jit::ir::{BasicBlockId, Body};

pub mod register;

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
    pub out_data_rbp_offset: i32,
}

pub fn codegen_prologue(buf: &mut CodeBuffer, callee_regs: &mut Vec<Register>) -> PrologueData {
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
    mov_reg_mem(
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

pub fn codegen_epilogue(buf: &mut CodeBuffer, prologue_data: &PrologueData) {
    add_rsp_imm8(buf, 8); // rdx - not a callee saved register and we have the return payload in that reg already, so just discard directly
    pop(buf, prologue_data.vtable_csreg);
    pop(buf, prologue_data.stub_csreg);
    pop(buf, prologue_data.vm_csreg);
    pop(buf, Register::Rbp);
    ret(buf);
}

pub fn mov_reg_reg(buf: &mut CodeBuffer, dest: Register, src: Register) {
    let mut encoding = encode_modrm_reg_reg(RegField::Reg(src), dest);
    if dest.join_register_size(src) == 64 {
        encoding.rex |= Rex::W;
    }
    encoding.write_into(buf, &[opcodes::MOV_RM_R]);
}

pub fn mov_mem_reg_scaled(
    buf: &mut CodeBuffer,
    src: Register,
    base: Register,
    index: Register,
    scale: Scale,
    offset: i32,
) {
    let mut encoding = encode_modrm_reg_mem(
        RegField::Reg(src),
        Mem::RegScaledIndexImm {
            base,
            index,
            scale,
            offset,
        },
    );
    if src.size_bits() == 64 {
        encoding.rex |= Rex::W;
    }
    encoding.write_into(buf, &[opcodes::MOV_RM_R]);
}

pub fn push(buf: &mut CodeBuffer, reg: Register) {
    if reg.needs_rex_prefix() {
        buf.push(rex::BASE | rex::B);
    }
    buf.push(opcodes::PUSH_REG + reg.fields());
}

/// Encodes `mov <dest reg>, [<base reg> + <offset imm8>]`
pub fn mov_reg_mem(buf: &mut CodeBuffer, dest: Register, base: Register, offset: i32) {
    let mut encoding = encode_modrm_reg_mem(RegField::Reg(dest), Mem::RegDispImm(base, offset));
    if dest.size_bits() == 64 {
        encoding.rex |= Rex::W;
    }
    encoding.write_into(buf, &[opcodes::MOV_R_RM]);
}

pub enum Scale {
    X1 = 0b00,
    X2 = 0b01,
    X4 = 0b10,
    X8 = 0b11,
}

impl Scale {
    pub fn target_pointer_with() -> Self {
        if cfg!(target_pointer_width = "64") {
            Scale::X8
        } else if cfg!(target_pointer_width = "32") {
            Scale::X4
        } else {
            panic!("Unsupported target pointer width");
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rex(u8);
impl Rex {
    pub const BASE: Self = Self(0x40);
    pub const R: Self = Self(0b0100);
    pub const X: Self = Self(0b0010);
    pub const B: Self = Self(0b0001);
    pub const W: Self = Self(0b1000);
    pub const EMPTY: Self = Self(0b0000);
}
impl BitOrAssign for Rex {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[derive(Debug)]
struct ModrmEncoding {
    rex: Rex,
    modrm: u8,
    sib: Option<u8>,
    displacement: ArrayVec<u8, 4>,
}

impl ModrmEncoding {
    fn write_into(self, buf: &mut CodeBuffer, opcode: &[u8]) {
        assert!((self.rex.0 & Rex::BASE.0) == Rex::BASE.0);

        if self.rex != Rex::BASE {
            buf.push(self.rex.0);
        }
        buf.extend_from_slice(opcode);
        buf.push(self.modrm);
        if let Some(sib) = self.sib {
            buf.push(sib);
        }
        buf.extend_from_slice(&self.displacement);
    }
}

enum RegField {
    Reg(Register),
    Bits(u8),
}

fn build_sib(scale: Scale, index: Option<Register>, base: Register) -> u8 {
    (scale as u8) << 6 | (index.map_or(0b100, |r| r.fields()) << 3) | base.fields()
}

fn encode_modrm_reg_reg(reg: RegField, rm: Register) -> ModrmEncoding {
    let mut rex = Rex::BASE;

    let reg_bits = match reg {
        RegField::Reg(register) => {
            if register.needs_rex_prefix() {
                rex |= Rex::R;
            }
            register.fields()
        }
        RegField::Bits(bits) => bits,
    };
    assert!(reg_bits <= 0b111);
    let rm_bits = rm.rm_field();
    if rm.needs_rex_prefix() {
        rex |= Rex::B;
    }
    let modrm = modrm::MOD_R | (reg_bits << 3) | rm_bits;

    ModrmEncoding {
        rex,
        modrm,
        sib: None,
        displacement: ArrayVec::new(),
    }
}

enum Mem {
    // [reg + imm]
    RegDispImm(Register, i32),
    RegScaledIndexImm {
        base: Register,
        index: Register,
        scale: Scale,
        offset: i32,
    },
}

fn encode_modrm_reg_mem(reg: RegField, rm: Mem) -> ModrmEncoding {
    let mut rex = Rex::BASE;
    let mut modrm = 0;

    match reg {
        RegField::Reg(register) => {
            if register.needs_rex_prefix() {
                rex |= Rex::R;
            }
            modrm |= register.fields() << 3;
        }
        RegField::Bits(bits) => modrm |= bits << 3,
    }

    let (sib, displacement) = match rm {
        Mem::RegDispImm(reg, imm) => {
            if reg.needs_rex_prefix() {
                rex |= Rex::B;
            }

            if let Ok(imm) = i8::try_from(imm) {
                let rm = reg.rm_field();
                modrm |= modrm::MOD_M8 | rm;

                let disp = ArrayVec::<u8, 4>::try_from([imm as u8].as_slice()).unwrap();

                if rm == Register::RM_FIELD_NEEDS_SIB {
                    (Some(build_sib(Scale::X1, None, reg)), disp)
                } else {
                    (None, disp)
                }
            } else {
                unimplemented!()
            }
        }
        Mem::RegScaledIndexImm {
            base,
            index,
            scale,
            offset,
        } => {
            if base.needs_rex_prefix() {
                rex |= Rex::B;
            }
            if index.needs_rex_prefix() {
                rex |= Rex::X;
            }

            if let Ok(offset) = i8::try_from(offset) {
                let sib = build_sib(scale, Some(index), base);
                modrm |= modrm::MOD_M8_SIB | Register::RM_FIELD_NEEDS_SIB;
                let displacement = ArrayVec::<u8, 4>::try_from([offset as u8].as_slice()).unwrap();

                (Some(sib), displacement)
            } else {
                unimplemented!()
            }
        }
    };

    ModrmEncoding {
        rex,
        modrm,
        sib,
        displacement,
    }
}

/// Encodes `movsd <dest reg>, [<base reg> + <index reg> * <scale> + <offset imm>]` (SIB addressing)
pub fn movsd_reg_mem_sib(
    buf: &mut CodeBuffer,
    dest: Register,
    base: Register,
    index: Register,
    scale: Scale,
    offset: i32,
) {
    buf.push(0xf2);

    let encoding = encode_modrm_reg_mem(
        RegField::Reg(dest),
        Mem::RegScaledIndexImm {
            base,
            index,
            scale,
            offset,
        },
    );

    encoding.write_into(buf, opcodes::MOVSD_REG_RM);
}

/// Encodes `ucomisd <lhs reg>, [<base reg> + <index reg> * <scale> + <offset imm>]` (SIB addressing)
pub fn ucomisd_reg_mem_sib(
    buf: &mut CodeBuffer,
    lhs: Register,
    base: Register,
    index: Register,
    scale: Scale,
    offset: i32,
) {
    buf.push(0x66);

    let encoding = encode_modrm_reg_mem(
        RegField::Reg(lhs),
        Mem::RegScaledIndexImm {
            base,
            index,
            scale,
            offset,
        },
    );

    encoding.write_into(buf, opcodes::UCOMISD_REG_RM);
}

/// Encodes `xor <dest reg>, <src reg>` instruction
pub fn xor_reg_reg(buf: &mut CodeBuffer, dest: Register, src: Register) {
    let mut encoding = encode_modrm_reg_reg(RegField::Reg(dest), src);
    if dest.join_register_size(src) == 64 {
        encoding.rex |= Rex::W;
    }
    encoding.write_into(buf, &[opcodes::XOR]);
}

/// Encodes `seta <dest reg>` instruction
pub fn seta_reg(buf: &mut CodeBuffer, dest: Register) {
    assert!(dest.size_bits() == 8);
    let encoding = encode_modrm_reg_reg(RegField::Bits(0), dest);
    encoding.write_into(buf, opcodes::SETA_RM8);
}

/// Encodes `addsd <dest reg>, [<base reg> + <index reg> * <scale> + <offset imm8>]` (SIB addressing)
pub fn addsd_reg_mem_sib(
    buf: &mut CodeBuffer,
    dest: Register,
    base: Register,
    index: Register,
    scale: Scale,
    offset: i32,
) {
    buf.push(0xf2);

    encode_modrm_reg_mem(
        RegField::Reg(dest),
        Mem::RegScaledIndexImm {
            base,
            index,
            scale,
            offset,
        },
    )
    .write_into(buf, opcodes::ADDSD_REG_RM);
}

/// Encodes `movsd [<base reg> + <index reg> * <scale> + <offset imm8>], <src reg>` (SIB addressing)
pub fn movsd_mem_sib_reg(
    buf: &mut CodeBuffer,
    source: Register,
    base: Register,
    index: Register,
    scale: Scale,
    offset: i32,
) {
    buf.push(0xf2);

    encode_modrm_reg_mem(
        RegField::Reg(source),
        Mem::RegScaledIndexImm {
            base,
            index,
            scale,
            offset,
        },
    )
    .write_into(buf, opcodes::MOVSD_RM_REG);
}

/// Encodes `decq <mem u8>` instruction
pub fn decq_mem(buf: &mut CodeBuffer, reg: Register, offset: i32) {
    let mut encoding = encode_modrm_reg_mem(RegField::Bits(1), Mem::RegDispImm(reg, offset));
    encoding.rex |= Rex::W;
    encoding.write_into(buf, &[opcodes::DEC_RM64]);
}

/// Encodes `dec <reg>` instruction
pub fn dec_reg(buf: &mut CodeBuffer, reg: Register) {
    let mut encoding = encode_modrm_reg_reg(RegField::Bits(1), reg);
    if reg.size_bits() == 64 {
        encoding.rex |= Rex::W;
    }
    encoding.write_into(buf, &[opcodes::DEC_RM64]);
}

pub fn mov_reg_imm32(buf: &mut CodeBuffer, reg: Register, imm: u32) {
    if reg.needs_rex_prefix() {
        buf.push(rex::BASE | rex::B);
    }
    buf.push(opcodes::MOV_REG_IMM32 + reg.fields());
    buf.extend_from_slice(&imm.to_le_bytes());
}

pub fn call_reg(buf: &mut CodeBuffer, register: Register) {
    encode_modrm_reg_reg(RegField::Bits(2), register).write_into(buf, &[opcodes::CALL_RM64]);
}

pub fn test_reg_reg(buf: &mut CodeBuffer, reg1: Register, reg2: Register) {
    let mut encoding = encode_modrm_reg_reg(RegField::Reg(reg1), reg2);
    if reg1.join_register_size(reg2) == 64 {
        encoding.rex |= Rex::W;
    }
    encoding.write_into(buf, &[opcodes::TEST_R_R]);
}

pub fn mov_mem_imm32(buf: &mut CodeBuffer, base: Register, offset: i32, imm: i32) {
    encode_modrm_reg_mem(RegField::Bits(0), Mem::RegDispImm(base, offset)).write_into(buf, &[opcodes::MOV_MEM_IMM32]);
    buf.extend_from_slice(&imm.to_le_bytes());
}

pub fn ret(buf: &mut CodeBuffer) {
    buf.push(opcodes::RET);
}

pub fn pop(buf: &mut CodeBuffer, reg: Register) {
    if reg.needs_rex_prefix() {
        buf.push(rex::BASE | rex::B);
    }
    buf.push(opcodes::POP_REG + reg.fields());
}

pub fn add_rsp_imm8(buf: &mut CodeBuffer, imm: u8) {
    let mut encoding = encode_modrm_reg_reg(RegField::Bits(0), Register::Rsp);
    encoding.rex |= Rex::W;
    encoding.write_into(buf, &[opcodes::ADD_RM_IMM8]);
    buf.push(imm as u8);
}

#[expect(dead_code)]
pub fn sub_rsp_imm8(buf: &mut CodeBuffer, imm: u8) {
    let mut encoding = encode_modrm_reg_reg(RegField::Bits(5), Register::Rsp);
    encoding.rex |= Rex::W;
    encoding.write_into(buf, &[opcodes::SUB_RM_IMM8]);
    buf.push(imm as u8);
}

pub fn cmp_reg_al_imm8(buf: &mut CodeBuffer, imm: u8) {
    buf.push(opcodes::CMP_AL_IMM8);
    buf.push(imm);
}

pub fn jne_imm32(buf: &mut CodeBuffer, offset: i32) -> CpuIp {
    buf.push(0x0f);
    buf.push(opcodes::JNE_REL32);
    let offset_ip = buf.offset();
    buf.extend_from_slice(&offset.to_le_bytes());
    offset_ip
}

pub fn je_imm32(buf: &mut CodeBuffer, offset: i32) -> CpuIp {
    buf.push(0x0f);
    buf.push(opcodes::JE_REL32);
    let offset_ip = buf.offset();
    buf.extend_from_slice(&offset.to_le_bytes());
    offset_ip
}

pub fn jne_block(buf: &mut CodeBuffer, target: BasicBlockId) {
    let patch_site = jne_imm32(buf, 0);
    if let Some(target_x86_ip) = buf.jumps.add_user_reference(target, patch_site) {
        patch_rel32(buf, patch_site, target_x86_ip);
    }
}

pub fn je_block(buf: &mut CodeBuffer, target: BasicBlockId) {
    let patch_site = je_imm32(buf, 0);
    if let Some(target_x86_ip) = buf.jumps.add_user_reference(target, patch_site) {
        patch_rel32(buf, patch_site, target_x86_ip);
    }
}

pub fn je_synthetic_label(buf: &mut CodeBuffer, label: SyntheticLabel) {
    let patch_site = je_imm32(buf, 0);
    if let Some(target_x86_ip) = buf.jumps.add_synthetic_reference(label, patch_site) {
        patch_rel32(buf, patch_site, target_x86_ip);
    }
}

pub fn jne_synthetic_label(buf: &mut CodeBuffer, label: SyntheticLabel) {
    let patch_site = jne_imm32(buf, 0);
    if let Some(target_x86_ip) = buf.jumps.add_synthetic_reference(label, patch_site) {
        patch_rel32(buf, patch_site, target_x86_ip);
    }
}

pub fn jmp_synthetic_label(buf: &mut CodeBuffer, label: SyntheticLabel) {
    let patch_site = jmp_imm32(buf, 0);
    if let Some(target_x86_ip) = buf.jumps.add_synthetic_reference(label, patch_site) {
        patch_rel32(buf, patch_site, target_x86_ip);
    }
}

pub fn jmp_imm32(buf: &mut CodeBuffer, offset: i32) -> CpuIp {
    buf.push(opcodes::JMP_REL32);
    let offset_ip = buf.offset();
    buf.extend_from_slice(&offset.to_le_bytes());
    offset_ip
}

pub fn jmp_block(buf: &mut CodeBuffer, target: BasicBlockId) {
    let patch_site = jmp_imm32(buf, 0);
    if let Some(target_x86_ip) = buf.jumps.add_user_reference(target, patch_site) {
        patch_rel32(buf, patch_site, target_x86_ip);
    }
}

pub fn lea_reg_mem(buf: &mut CodeBuffer, dest: Register, base: Register, offset: i32) {
    let mut encoding = encode_modrm_reg_mem(RegField::Reg(dest), Mem::RegDispImm(base, offset));
    if dest.size_bits() == 64 {
        encoding.rex |= Rex::W;
    }
    encoding.write_into(buf, &[opcodes::LEA]);
}

fn patch_rel32(buf: &mut CodeBuffer, patch_site: CpuIp, target: CpuIp) {
    let patch_site = patch_site as usize;
    let disp = target as i64 - (patch_site as i64 + 4);
    let disp = i32::try_from(disp).expect("rel32 displacement out of range");
    buf.buffer[patch_site..patch_site + 4].copy_from_slice(&disp.to_le_bytes());
}

fn patch_sites_to(buf: &mut CodeBuffer, patch_sites: Vec<CpuIp>, target: CpuIp) {
    for patch_site in patch_sites {
        patch_rel32(buf, patch_site, target);
    }
}

pub fn register_user_block(buf: &mut CodeBuffer, block: BasicBlockId) -> bool {
    let x86_ip = buf.offset();
    match buf.jumps.register_user_label(block, x86_ip) {
        PatchData::Unresolved { references } => {
            patch_sites_to(buf, references, x86_ip);
            true
        }
        PatchData::Resolved { ip: _ } => false,
    }
}

pub fn register_synthetic_block(buf: &mut CodeBuffer, label: SyntheticLabel) -> bool {
    let x86_ip = buf.offset();
    match buf.jumps.register_synthetic_label(label, x86_ip) {
        PatchData::Unresolved { references } => {
            patch_sites_to(buf, references, x86_ip);
            true
        }
        PatchData::Resolved { ip: _ } => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_buf() -> CodeBuffer {
        CodeBuffer::new(&Body {
            blocks: Default::default(),
            root: BasicBlockId(0),
        })
    }

    #[track_caller]
    fn assert_enc(expected: &[u8], emit: impl FnOnce(&mut CodeBuffer)) {
        let mut buf = make_buf();
        emit(&mut buf);
        assert_eq!(buf.buffer(), expected, "expected {expected:02x?}");
    }

    #[test]
    fn enc_mov_reg_reg() {
        // 64-bit, no REX extensions needed besides W
        assert_enc(&[0x48, 0x89, 0xC0], |b| mov_reg_reg(b, Register::Rax, Register::Rax));
        assert_enc(&[0x48, 0x89, 0xC8], |b| mov_reg_reg(b, Register::Rax, Register::Rcx));
        assert_enc(&[0x48, 0x89, 0xC1], |b| mov_reg_reg(b, Register::Rcx, Register::Rax));
        // 32-bit: no W bit, REX suppressed entirely
        assert_enc(&[0x89, 0xC0], |b| mov_reg_reg(b, Register::Eax, Register::Eax));
        // REX.B: dest is r8+
        assert_enc(&[0x49, 0x89, 0xC0], |b| mov_reg_reg(b, Register::R8, Register::Rax));
        // REX.R: src is r8+
        assert_enc(&[0x4C, 0x89, 0xE8], |b| mov_reg_reg(b, Register::Rax, Register::R13));
        // REX.R | REX.B
        assert_enc(&[0x4D, 0x89, 0xFD], |b| mov_reg_reg(b, Register::R13, Register::R15));
    }

    #[test]
    fn enc_push() {
        assert_enc(&[0x55], |b| push(b, Register::Rbp));
        assert_enc(&[0x50], |b| push(b, Register::Rax));
        // REX.B for r8+
        assert_enc(&[0x41, 0x55], |b| push(b, Register::R13));
        assert_enc(&[0x41, 0x50], |b| push(b, Register::R8));
    }

    #[test]
    fn enc_mov_reg_mem() {
        // [rbp - 16]: mod=01, rm=101, disp8
        assert_enc(&[0x48, 0x8B, 0x45, 0xF0], |b| {
            mov_reg_mem(b, Register::Rax, Register::Rbp, -16)
        });
        assert_enc(&[0x48, 0x8B, 0x45, 0x08], |b| {
            mov_reg_mem(b, Register::Rax, Register::Rbp, 8)
        });
        // 32-bit dest: no W
        assert_enc(&[0x8B, 0x45, 0xF8], |b| {
            mov_reg_mem(b, Register::Eax, Register::Rbp, -8)
        });
        // [rsp]: rm=100 forces a SIB byte with no index
        assert_enc(&[0x48, 0x8B, 0x44, 0x24, 0x00], |b| {
            mov_reg_mem(b, Register::Rax, Register::Rsp, 0)
        });
        // [r12]: SIB + REX.B
        assert_enc(&[0x4D, 0x8B, 0x54, 0x24, 0x08], |b| {
            mov_reg_mem(b, Register::R10, Register::R12, 8)
        });
        // [r13]: rm=101 with mod=01 is plain disp8, no SIB needed; REX.B only
        assert_enc(&[0x49, 0x8B, 0x45, 0xF0], |b| {
            mov_reg_mem(b, Register::Rax, Register::R13, -16)
        });
        // REX.W with rdi/rsi
        assert_enc(&[0x48, 0x8B, 0x7E, 0x04], |b| {
            mov_reg_mem(b, Register::Rdi, Register::Rsi, 4)
        });
    }

    #[test]
    fn enc_movsd_reg_mem_sib() {
        // F2 REX.XB 0F 10, [r12 + r11*8 - 16]
        assert_enc(&[0xF2, 0x43, 0x0F, 0x10, 0x44, 0xDC, 0xF0], |b| {
            movsd_reg_mem_sib(b, Register::Xmm0, Register::R12, Register::R11, Scale::X8, -16)
        });
        // No REX at all: [rbp + rsi*4 + 8]
        assert_enc(&[0xF2, 0x0F, 0x10, 0x4C, 0xB5, 0x08], |b| {
            movsd_reg_mem_sib(b, Register::Xmm1, Register::Rbp, Register::Rsi, Scale::X4, 8)
        });
        // REX.R: dest is xmm13
        assert_enc(&[0xF2, 0x44, 0x0F, 0x10, 0x6C, 0x08, 0x00], |b| {
            movsd_reg_mem_sib(b, Register::Xmm13, Register::Rax, Register::Rcx, Scale::X1, 0)
        });
        // REX.R with scale X2
        assert_enc(&[0xF2, 0x44, 0x0F, 0x10, 0x44, 0x50, 0x10], |b| {
            movsd_reg_mem_sib(b, Register::Xmm8, Register::Rax, Register::Rdx, Scale::X2, 16)
        });
        // REX.XB: [r13 + r12*8 - 8]
        assert_enc(&[0xF2, 0x43, 0x0F, 0x10, 0x44, 0xE5, 0xF8], |b| {
            movsd_reg_mem_sib(b, Register::Xmm0, Register::R13, Register::R12, Scale::X8, -8)
        });
    }

    #[test]
    fn enc_addsd_reg_mem_sib() {
        assert_enc(&[0xF2, 0x43, 0x0F, 0x58, 0x44, 0xDC, 0xF0], |b| {
            addsd_reg_mem_sib(b, Register::Xmm0, Register::R12, Register::R11, Scale::X8, -16)
        });
        assert_enc(&[0xF2, 0x44, 0x0F, 0x58, 0x6C, 0x08, 0x00], |b| {
            addsd_reg_mem_sib(b, Register::Xmm13, Register::Rax, Register::Rcx, Scale::X1, 0)
        });
    }

    #[test]
    fn enc_movsd_mem_sib_reg() {
        assert_enc(&[0xF2, 0x43, 0x0F, 0x11, 0x44, 0xDC, 0xF0], |b| {
            movsd_mem_sib_reg(b, Register::Xmm0, Register::R12, Register::R11, Scale::X8, -16)
        });
        assert_enc(&[0xF2, 0x0F, 0x11, 0x4C, 0xB5, 0x08], |b| {
            movsd_mem_sib_reg(b, Register::Xmm1, Register::Rbp, Register::Rsi, Scale::X4, 8)
        });
        // REX.R: source is xmm15
        assert_enc(&[0xF2, 0x44, 0x0F, 0x11, 0x7C, 0xC0, 0x04], |b| {
            movsd_mem_sib_reg(b, Register::Xmm15, Register::Rax, Register::Rax, Scale::X8, 4)
        });
    }

    #[test]
    fn enc_decq_mem() {
        // REX.W FF /1, [rbp - 8]
        assert_enc(&[0x48, 0xFF, 0x4D, 0xF8], |b| decq_mem(b, Register::Rbp, -8));
        // [r12]: SIB + REX.WB
        assert_enc(&[0x49, 0xFF, 0x4C, 0x24, 0xF0], |b| decq_mem(b, Register::R12, -16));
        // [r13]: REX.WB, disp8
        assert_enc(&[0x49, 0xFF, 0x4D, 0x04], |b| decq_mem(b, Register::R13, 4));
        assert_enc(&[0x48, 0xFF, 0x48, 0x00], |b| decq_mem(b, Register::Rax, 0));
    }

    #[test]
    fn enc_dec_reg() {
        assert_enc(&[0x48, 0xFF, 0xC8], |b| dec_reg(b, Register::Rax));
        // REX.WB
        assert_enc(&[0x49, 0xFF, 0xCA], |b| dec_reg(b, Register::R10));
        assert_enc(&[0x49, 0xFF, 0xCD], |b| dec_reg(b, Register::R13));
        assert_enc(&[0x49, 0xFF, 0xCF], |b| dec_reg(b, Register::R15));
        // 32-bit: no W, REX suppressed
        assert_enc(&[0xFF, 0xC8], |b| dec_reg(b, Register::Eax));
    }

    #[test]
    fn enc_mov_reg_imm32() {
        assert_enc(&[0xB8, 0x44, 0x33, 0x22, 0x11], |b| {
            mov_reg_imm32(b, Register::Rax, 0x1122_3344)
        });
        assert_enc(&[0xB9, 0xEF, 0xBE, 0xAD, 0xDE], |b| {
            mov_reg_imm32(b, Register::Rcx, 0xDEAD_BEEF)
        });
        assert_enc(&[0xB8, 0x2A, 0x00, 0x00, 0x00], |b| mov_reg_imm32(b, Register::Eax, 42));
        // REX.B
        assert_enc(&[0x41, 0xB9, 0x44, 0x33, 0x22, 0x11], |b| {
            mov_reg_imm32(b, Register::R9, 0x1122_3344)
        });
        assert_enc(&[0x41, 0xBF, 0x00, 0x00, 0x00, 0x00], |b| {
            mov_reg_imm32(b, Register::R15, 0)
        });
    }

    #[test]
    fn enc_mov_mem_reg_scaled() {
        assert_enc(&[0x48, 0x89, 0x54, 0xC8, 0xF0], |b| {
            mov_mem_reg_scaled(b, Register::Rdx, Register::Rax, Register::Rcx, Scale::X8, -16)
        });
        assert_enc(&[0x89, 0x44, 0xC8, 0xF0], |b| {
            mov_mem_reg_scaled(b, Register::Eax, Register::Rax, Register::Rcx, Scale::X8, -16)
        });
        assert_enc(&[0x4c, 0x89, 0x5c, 0xC8, 0xF0], |b| {
            mov_mem_reg_scaled(b, Register::R11, Register::Rax, Register::Rcx, Scale::X8, -16)
        });
    }

    #[test]
    fn enc_call_reg() {
        assert_enc(&[0xFF, 0xD0], |b| call_reg(b, Register::Rax));
        assert_enc(&[0xFF, 0xD1], |b| call_reg(b, Register::Rcx));
        // REX.B
        assert_enc(&[0x41, 0xFF, 0xD2], |b| call_reg(b, Register::R10));
        assert_enc(&[0x41, 0xFF, 0xD5], |b| call_reg(b, Register::R13));
    }

    #[test]
    fn enc_test_reg_reg() {
        assert_enc(&[0x48, 0x85, 0xC0], |b| test_reg_reg(b, Register::Rax, Register::Rax));
        // 32-bit: no W
        assert_enc(&[0x85, 0xC0], |b| test_reg_reg(b, Register::Eax, Register::Eax));
        // REX.B: rm is r10
        assert_enc(&[0x49, 0x85, 0xC2], |b| test_reg_reg(b, Register::Rax, Register::R10));
        // REX.R: reg is r13
        assert_enc(&[0x4C, 0x85, 0xE8], |b| test_reg_reg(b, Register::R13, Register::Rax));
        // REX.RB
        assert_enc(&[0x4D, 0x85, 0xFF], |b| test_reg_reg(b, Register::R15, Register::R15));
    }

    #[test]
    fn enc_xor() {
        assert_enc(&[0x48, 0x31, 0xC0], |b| xor_reg_reg(b, Register::Rax, Register::Rax));
        // 32-bit: no W
        assert_enc(&[0x31, 0xC0], |b| xor_reg_reg(b, Register::Eax, Register::Eax));
        // REX.B: source is r10
        assert_enc(&[0x49, 0x31, 0xC2], |b| xor_reg_reg(b, Register::Rax, Register::R10));
        // REX.R: dest is r13
        assert_enc(&[0x4C, 0x31, 0xE8], |b| xor_reg_reg(b, Register::R13, Register::Rax));
        // REX.RB
        assert_enc(&[0x4D, 0x31, 0xFF], |b| xor_reg_reg(b, Register::R15, Register::R15));
    }

    #[test]
    fn enc_mov_mem_imm32() {
        assert_enc(&[0xC7, 0x45, 0xF0, 0x2A, 0x00, 0x00, 0x00], |b| {
            mov_mem_imm32(b, Register::Rbp, -16, 42)
        });
        // REX.B + SIB for [r12]
        assert_enc(&[0x41, 0xC7, 0x44, 0x24, 0xF0, 0x2A, 0x00, 0x00, 0x00], |b| {
            mov_mem_imm32(b, Register::R12, -16, 42)
        });
        // REX.B for [r13], negative immediate
        assert_enc(&[0x41, 0xC7, 0x45, 0x08, 0xFF, 0xFF, 0xFF, 0xFF], |b| {
            mov_mem_imm32(b, Register::R13, 8, -1)
        });
        assert_enc(&[0xC7, 0x40, 0x00, 0xFF, 0xFF, 0xFF, 0x7F], |b| {
            mov_mem_imm32(b, Register::Rax, 0, i32::MAX)
        });
    }

    #[test]
    fn enc_ret() {
        assert_enc(&[0xC3], ret);
    }

    #[test]
    fn enc_pop() {
        assert_enc(&[0x5D], |b| pop(b, Register::Rbp));
        assert_enc(&[0x58], |b| pop(b, Register::Rax));
        // REX.B for r8+
        assert_enc(&[0x41, 0x5D], |b| pop(b, Register::R13));
        assert_enc(&[0x41, 0x5F], |b| pop(b, Register::R15));
    }

    #[test]
    fn enc_add_rsp_imm8() {
        assert_enc(&[0x48, 0x83, 0xC4, 0x08], |b| add_rsp_imm8(b, 8));
        assert_enc(&[0x48, 0x83, 0xC4, 0x20], |b| add_rsp_imm8(b, 32));
    }

    #[test]
    fn enc_sub_rsp_imm8() {
        assert_enc(&[0x48, 0x83, 0xEC, 0x08], |b| sub_rsp_imm8(b, 8));
        assert_enc(&[0x48, 0x83, 0xEC, 0x28], |b| sub_rsp_imm8(b, 40));
    }

    #[test]
    fn enc_cmp_reg_al_imm8() {
        assert_enc(&[0x3C, 0x2A], |b| cmp_reg_al_imm8(b, 42));
        assert_enc(&[0x3C, 0x00], |b| cmp_reg_al_imm8(b, 0));
        assert_enc(&[0x3C, 0xFF], |b| cmp_reg_al_imm8(b, u8::MAX));
    }

    #[test]
    fn enc_jne_imm32() {
        let mut buf = make_buf();
        let ip = jne_imm32(&mut buf, 0x1122_3344);
        assert_eq!(ip, 2);
        assert_eq!(buf.buffer(), &[0x0F, 0x85, 0x44, 0x33, 0x22, 0x11]);

        let mut buf = make_buf();
        let ip = jne_imm32(&mut buf, -1);
        assert_eq!(ip, 2);
        assert_eq!(buf.buffer(), &[0x0F, 0x85, 0xFF, 0xFF, 0xFF, 0xFF]);

        let mut buf = make_buf();
        let ip = jne_imm32(&mut buf, i32::MAX);
        assert_eq!(ip, 2);
        assert_eq!(buf.buffer(), &[0x0F, 0x85, 0xFF, 0xFF, 0xFF, 0x7F]);
    }

    #[test]
    fn enc_je_imm32() {
        let mut buf = make_buf();
        let ip = je_imm32(&mut buf, 0x1122_3344);
        assert_eq!(ip, 2);
        assert_eq!(buf.buffer(), &[0x0F, 0x84, 0x44, 0x33, 0x22, 0x11]);

        let mut buf = make_buf();
        let ip = je_imm32(&mut buf, -16);
        assert_eq!(ip, 2);
        assert_eq!(buf.buffer(), &[0x0F, 0x84, 0xF0, 0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn enc_jmp_imm32() {
        let mut buf = make_buf();
        let ip = jmp_imm32(&mut buf, 0x1122_3344);
        assert_eq!(ip, 1);
        assert_eq!(buf.buffer(), &[0xE9, 0x44, 0x33, 0x22, 0x11]);

        let mut buf = make_buf();
        let ip = jmp_imm32(&mut buf, -16);
        assert_eq!(ip, 1);
        assert_eq!(buf.buffer(), &[0xE9, 0xF0, 0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn enc_lea_reg_mem() {
        assert_enc(&[0x48, 0x8D, 0x45, 0xF0], |b| {
            lea_reg_mem(b, Register::Rax, Register::Rbp, -16)
        });
        // REX.WB + SIB for [r12 - 16]
        assert_enc(&[0x49, 0x8D, 0x44, 0x24, 0xF0], |b| {
            lea_reg_mem(b, Register::Rax, Register::R12, -16)
        });
        // REX.WB for [r13 + 8], no SIB
        assert_enc(&[0x49, 0x8D, 0x45, 0x08], |b| {
            lea_reg_mem(b, Register::Rax, Register::R13, 8)
        });
        // 32-bit: no W
        assert_enc(&[0x8D, 0x45, 0xF0], |b| {
            lea_reg_mem(b, Register::Eax, Register::Rbp, -16)
        });
        // REX.WR
        assert_enc(&[0x4C, 0x8D, 0x55, 0x10], |b| {
            lea_reg_mem(b, Register::R10, Register::Rbp, 16)
        });
    }

    #[test]
    fn enc_ucomisd_reg_mem_sib() {
        assert_enc(&[0x66, 0x0F, 0x2E, 0x44, 0xC8, 0xF0], |b| {
            ucomisd_reg_mem_sib(b, Register::Xmm0, Register::Rax, Register::Rcx, Scale::X8, -16)
        });
        // REX.R
        assert_enc(&[0x66, 0x44, 0x0F, 0x2E, 0x7C, 0xC8, 0xF0], |b| {
            ucomisd_reg_mem_sib(b, Register::Xmm15, Register::Rax, Register::Rcx, Scale::X8, -16)
        });
    }

    #[test]
    fn enc_seta_reg() {
        assert_enc(&[0x0F, 0x97, 0xC0], |b| seta_reg(b, Register::Al));
        assert_enc(&[0x0F, 0x97, 0xC1], |b| seta_reg(b, Register::Cl));
        assert_eq!(Register::R8.to_8(), Register::R8b);
        assert_eq!(Register::Rcx.to_8(), Register::Cl);
        assert_eq!(Register::Edx.to_8(), Register::Dl);
        assert_enc(&[0x41, 0x0F, 0x97, 0xC0], |b| seta_reg(b, Register::R8.to_8()));
    }
}
