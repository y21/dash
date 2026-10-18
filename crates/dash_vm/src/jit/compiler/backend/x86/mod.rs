mod modrm {
    pub const MOD_R: u8 = 0b11_000_000;
    pub const MOD_M8: u8 = 0b01_000_000;
    pub const MOD_M8_SIB: u8 = 0b01_000_100;
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
    pub const MOVSD_REG_RM: &[u8] = &[0x0F, 0x10];
    pub const ADDSD_REG_RM: &[u8] = &[0x0F, 0x58];
    pub const MOVSD_RM_REG: &[u8] = &[0x0F, 0x11];
    pub const DEC_RM64: u8 = 0xff;
}

use std::mem::offset_of;
use std::ops::BitOrAssign;

use arrayvec::ArrayVec;
use dash_middle::indexvec::IndexVec;

use crate::frame::Ip;
use crate::jit::JitVtable;
use crate::jit::compiler::codebuffer::CodeBuffer;
use crate::jit::compiler::jumpresolver::{CpuIp, PatchData, SyntheticLabel};
use crate::jit::ir::{BasicBlockId, Body};

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

    Xmm0,
    Xmm1,
    Xmm2,
    Xmm3,
    Xmm4,
    Xmm5,
    Xmm6,
    Xmm7,
    Xmm8,
    Xmm9,
    Xmm10,
    Xmm11,
    Xmm12,
    Xmm13,
    Xmm14,
    Xmm15,
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
            | Register::R11
            | Register::Xmm0
            | Register::Xmm1
            | Register::Xmm2
            | Register::Xmm3
            | Register::Xmm4
            | Register::Xmm5
            | Register::Xmm6
            | Register::Xmm7
            | Register::Xmm8
            | Register::Xmm9
            | Register::Xmm10
            | Register::Xmm11
            | Register::Xmm12
            | Register::Xmm13
            | Register::Xmm14
            | Register::Xmm15 => 64,
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
            Register::Rax | Register::Eax | Register::Al | Register::R8 | Register::Xmm0 | Register::Xmm8 => 0,
            Register::Rcx | Register::R9 | Register::Xmm1 | Register::Xmm9 => 1,
            Register::Rdx | Register::R10 | Register::Xmm2 | Register::Xmm10 => 2,
            Register::Rbx | Register::R11 | Register::Xmm3 | Register::Xmm11 => 3,
            Register::Rsp | Register::R12 | Register::Xmm4 | Register::Xmm12 => 4,
            Register::Rbp | Register::R13 | Register::Xmm5 | Register::Xmm13 => 5,
            Register::Rsi | Register::R14 | Register::Xmm6 | Register::Xmm14 => 6,
            Register::Rdi | Register::R15 | Register::Xmm7 | Register::Xmm15 => 7,
        }
    }

    const RM_FIELD_NEEDS_SIB: u8 = 0b100;
    pub fn rm_field(self) -> u8 {
        match self {
            Register::Al | Register::Eax | Register::Rax | Register::Xmm0 | Register::R8 | Register::Xmm8 => 0b000,
            Register::Rcx | Register::Xmm1 | Register::R9 | Register::Xmm9 => 0b001,
            Register::Rdx | Register::Xmm2 | Register::R10 | Register::Xmm10 => 0b010,
            Register::Rbx | Register::Xmm3 | Register::R11 | Register::Xmm11 => 0b011,
            Register::Rsp | Register::Xmm4 | Register::R12 | Register::Xmm12 => 0b100,
            Register::Rbp | Register::Xmm5 | Register::R13 | Register::Xmm13 => 0b101,
            Register::Rsi | Register::Xmm6 | Register::R14 | Register::Xmm14 => 0b110,
            Register::Rdi | Register::Xmm7 | Register::R15 | Register::Xmm15 => 0b111,
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
            | Register::R15
            | Register::Xmm8
            | Register::Xmm9
            | Register::Xmm10
            | Register::Xmm11
            | Register::Xmm12
            | Register::Xmm13
            | Register::Xmm14
            | Register::Xmm15 => true,
            Register::Rax
            | Register::Eax
            | Register::Rbp
            | Register::Rsp
            | Register::Rdi
            | Register::Rsi
            | Register::Rdx
            | Register::Al
            | Register::Rcx
            | Register::Rbx
            | Register::Xmm0
            | Register::Xmm1
            | Register::Xmm2
            | Register::Xmm3
            | Register::Xmm4
            | Register::Xmm5
            | Register::Xmm6
            | Register::Xmm7 => false,
        }
    }
}

pub fn usable_callee_saved_regs() -> Vec<Register> {
    // Intentionally in reverse order such that pop() will yield regs that don't need a REX prefix first
    vec![
        // Register::Rbx,
        // Register::R12,
        // Register::R13,
        // Register::R14,
        // Register::R15,
        Register::R15,
        Register::R14,
        Register::R13,
        Register::R12,
        Register::Rbx,
    ]
}
pub fn usable_caller_saved_gprs() -> Vec<Register> {
    // Intentionally in reverse order such that pop() will yield regs that don't need a REX prefix first
    vec![
        // Register::Rax,
        // Register::Rcx,
        // Register::Rdx,
        // Register::Rsi,
        // Register::Rdi,
        // Register::R8,
        // Register::R9,
        // Register::R10,
        // Register::R11,
        Register::R11,
        Register::R10,
        Register::R9,
        Register::R8,
        Register::Rdi,
        Register::Rsi,
        Register::Rdx,
        Register::Rcx,
        Register::Rax,
    ]
}
pub fn usable_fprs() -> Vec<Register> {
    // Intentionally in reverse order such that pop() will yield regs that don't need a REX prefix first
    vec![
        Register::Xmm15,
        Register::Xmm14,
        Register::Xmm13,
        Register::Xmm12,
        Register::Xmm11,
        Register::Xmm10,
        Register::Xmm9,
        Register::Xmm8,
        Register::Xmm7,
        Register::Xmm6,
        Register::Xmm5,
        Register::Xmm4,
        Register::Xmm3,
        Register::Xmm2,
        Register::Xmm1,
        Register::Xmm0,
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

pub fn codegen_epilogue(buf: &mut CodeBuffer, prologue_data: &PrologueData) {
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

/// Encodes `mov <dest reg>, [<base reg> + <offset imm8>]`
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

    let modrm = modrm::MOD_M8 | (dest.reg_field() << 3) | base.reg_field(); // base should be using dest.rm_field()!!
    buf.push(modrm);

    buf.push(offset as u8);
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

struct ModrmEncoding {
    rex: Rex,
    modrm: u8,
    sib: Option<u8>,
    displacement: ArrayVec<u8, 4>,
}

impl ModrmEncoding {
    fn write_into(self, buf: &mut CodeBuffer, opcode: &[u8]) {
        buf.push(self.rex.0);
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
    (scale as u8) << 6 | (index.map_or(0b100, |r| r.reg_field()) << 3) | base.reg_field()
}

fn encode_modrm_reg_reg(reg: RegField, rm: Register) -> ModrmEncoding {
    let mut rex = Rex::BASE;

    let reg_bits = match reg {
        RegField::Reg(register) => {
            rex |= Rex::R;
            register.reg_field()
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
            modrm |= register.reg_field() << 3;
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
                modrm |= rm;

                if rm == Register::RM_FIELD_NEEDS_SIB {
                    (
                        Some(build_sib(Scale::X1, None, reg)),
                        ArrayVec::<u8, 4>::try_from([imm as u8].as_slice()).unwrap(),
                    )
                } else {
                    (None, ArrayVec::new())
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

/// Encodes `movsd <dest reg>, [<base reg> + <index reg> * <scale> + <offset imm8>]` (SIB addressing)
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
#[test]
fn test_movsd() {
    let mut cb = CodeBuffer::new(&Body {
        blocks: Default::default(),
        root: BasicBlockId(0),
    });
    // movsd_reg_mem_sib(&mut cb, Register::Xmm0, Register::R12, Register::R11, Scale::X8, -16);
    // addsd_reg_mem_sib(&mut cb, Register::Xmm0, Register::R12, Register::R11, Scale::X8, -16);
    // movsd_mem_sib_reg(&mut cb, Register::Xmm0, Register::R12, Register::R11, Scale::X8, -16);
    /*
      4a:   49 8b 44 08 4c          mov    0x4c(%r8,%rcx,1),%rax

               mov_reg_mem_u8(
                   &mut self.buf,
                   check_last_value_fn,
                   self.prologue.vtable_csreg,
                   offset_of!(JitVtable, check_last_value).try_into().unwrap(),
               );
    */
    // mov_reg_mem_u8(&mut cb, Register::Rax, Register::R12, 8);
    println!("{:x?}", cb.buffer);
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

/// Encodes `dec <mem u8>` instruction
pub fn dec_mem_u8_imm8(buf: &mut CodeBuffer, reg: Register, offset: i8) {
    let mut rex = 0;
    if reg.needs_rex_prefix() {
        rex |= rex::B;
    }
    if reg.size_bits() == 64 {
        rex |= rex::W;
    }

    if rex != 0 {
        buf.push(rex::BASE | rex);
    }

    buf.push(opcodes::DEC_RM64);
    let modrm = modrm::MOD_M8 | (1 << 3) | reg.reg_field();
    buf.push(modrm);
    buf.push(offset as u8);
}

/// Encodes `dec <reg>` instruction
pub fn dec_reg(buf: &mut CodeBuffer, reg: Register) {
    let mut rex = 0;
    if reg.needs_rex_prefix() {
        rex |= rex::B;
    }
    if reg.size_bits() == 64 {
        rex |= rex::W;
    }

    if rex != 0 {
        buf.push(rex::BASE | rex);
    }

    buf.push(opcodes::DEC_RM64);
    let modrm = modrm::MOD_R | (1 << 3) | reg.reg_field();
    buf.push(modrm);
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

// TODO: generate tests for all of these
