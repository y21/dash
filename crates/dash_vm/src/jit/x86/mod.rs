mod opcodes {
    pub const MOV_REG_IMM32: u8 = 0xB8;
    pub const RET: u8 = 0xC3;
}

pub enum Register {
    Eax,
}

impl Register {
    pub fn modrm_byte(&self) -> u8 {
        match self {
            Register::Eax => 0,
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
        self.0.push(opcodes::MOV_REG_IMM32 + reg.modrm_byte());
        self.0.extend_from_slice(&imm.to_le_bytes());
    }

    pub fn ret(&mut self) {
        self.0.push(opcodes::RET);
    }
}
