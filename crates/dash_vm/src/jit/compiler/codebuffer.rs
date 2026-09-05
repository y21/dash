use crate::jit::jumpresolver::{CpuIp, JumpResolver};

pub struct CodeBuffer {
    pub buffer: Vec<u8>,
    pub jumps: JumpResolver,
}

impl CodeBuffer {
    pub fn new(bytecode_len: usize) -> Self {
        Self {
            buffer: Vec::new(),
            jumps: JumpResolver::new(bytecode_len),
        }
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    pub fn offset(&self) -> CpuIp {
        self.buffer.len().try_into().unwrap()
    }

    pub fn push(&mut self, byte: u8) {
        self.buffer.push(byte);
    }

    pub fn extend_from_slice(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }
}
