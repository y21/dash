use std::convert::TryInto;

use super::instruction;

pub fn force_utf8(s: &[u8]) -> String {
    std::str::from_utf8(s).expect("Invalid UTF8").into()
}

pub struct InstructionBuilder {
    buf: Vec<u8>,
}

impl InstructionBuilder {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn append(&mut self, other: &mut Vec<u8>) {
        self.buf.append(other)
    }

    pub fn write(&mut self, instruction: u8) {
        self.buf.push(instruction);
    }

    pub fn writew(&mut self, instruction: u16) {
        self.buf.extend(instruction.to_ne_bytes());
    }

    pub fn write_all(&mut self, instruction: &[u8]) {
        self.buf.extend(instruction)
    }

    pub fn write_arr<const N: usize>(&mut self, instruction: [u8; N]) {
        self.buf.extend(instruction)
    }

    pub fn write_wide_instr(&mut self, instr: u8, instrw: u8, value: u16) {
        if let Ok(value) = value.try_into() {
            self.write_arr([instr, value]);
        } else {
            self.write(instrw);
            self.writew(value);
        }
    }

    pub fn remove_pop_end(&mut self) {
        if let Some(&instruction::POP) = self.buf.last() {
            self.buf.pop();
        }
    }

    pub fn build(self) -> Vec<u8> {
        self.buf
    }
}

impl From<Vec<u8>> for InstructionBuilder {
    fn from(buf: Vec<u8>) -> Self {
        Self { buf }
    }
}
