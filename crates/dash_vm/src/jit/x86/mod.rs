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
}

pub enum Register {
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
            Register::Eax => false,
        }
    }

    pub fn reg_field(&self) -> u8 {
        match self {
            Register::Rax | Register::Eax | Register::R8 => 0,
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
            | Register::Rdx => false,
        }
    }
}

pub struct Emitter(Vec<u8>);

impl Emitter {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn buffer(&self) -> &[u8] {
        &self.0
    }

    pub fn mov_reg_imm32(&mut self, reg: Register, imm: u32) {
        if reg.needs_rex_prefix() {
            self.0.push(rex::BASE | rex::B);
        }
        self.0.push(opcodes::MOV_REG_IMM32 + reg.reg_field());
        self.0.extend_from_slice(&imm.to_le_bytes());
    }

    pub fn ret(&mut self) {
        self.0.push(opcodes::RET);
    }

    pub fn push(&mut self, reg: Register) {
        if reg.needs_rex_prefix() {
            self.0.push(rex::BASE | rex::B);
        }
        self.0.push(opcodes::PUSH_REG + reg.reg_field());
    }

    pub fn pop(&mut self, reg: Register) {
        if reg.needs_rex_prefix() {
            self.0.push(rex::BASE | rex::B);
        }
        self.0.push(opcodes::POP_REG + reg.reg_field());
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
            self.0.push(rex::BASE | rex);
        }

        self.0.push(opcodes::MOV_RM_R);

        let modrm = modrm::MOD_R | (src.reg_field() << 3) | dest.reg_field();
        self.0.push(modrm);
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
            self.0.push(rex::BASE | rex);
        }

        self.0.push(opcodes::MOV_R_RM);

        let modrm = modrm::MOD_M8 | (dest.reg_field() << 3) | base.reg_field();
        self.0.push(modrm);

        self.0.extend_from_slice(&offset.to_le_bytes());
    }

    pub fn call_reg(&mut self, register: Register) {
        let mut rex = 0;
        if register.needs_rex_prefix() {
            rex |= rex::B;
        }

        if rex != 0 {
            self.0.push(rex::BASE | rex);
        }

        self.0.push(0xFF);

        let modrm = modrm::MOD_R | (2 << 3) | register.reg_field();
        self.0.push(modrm);
    }
}
