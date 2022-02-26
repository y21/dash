use std::collections::HashMap;
use std::convert::TryInto;
use std::hash::Hash;

use super::constant::LimitExceededError;
use super::instruction;

pub fn force_utf8(s: &[u8]) -> String {
    std::str::from_utf8(s).expect("Invalid UTF8").into()
}

pub struct InstructionBuilder {
    buf: Vec<u8>,
    labels: HashMap<Label, usize>,
    jumps: Vec<Label>,
}

impl InstructionBuilder {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            labels: HashMap::new(),
            jumps: Vec::new(),
        }
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

    /// Adds a label at the current instruction pointer, which can be jumped to
    pub fn add_label(&mut self, label: Label) {
        self.labels.insert(label, self.buf.len());
    }

    pub fn add_jump(&mut self, label: Label) -> Result<u16, LimitExceededError> {
        self.jumps.push(label);
        self.jumps
            .len()
            .try_into()
            .map_err(|_| LimitExceededError)
            .map(|x: u16| x - 1)
    }

    pub fn build(self) -> Vec<u8> {
        if self.jumps.is_empty() {
            return self.buf;
        }

        let mut buf = Vec::with_capacity(self.jumps.len());

        let mut iter = self.buf.into_iter();

        while let Some(byte) = iter.next() {
            if byte == instruction::JMPFALSEP {
                let id = iter.next().expect("Missing jump label index");
                let label = &self.jumps[id as usize];
                let position = self.labels[label] as isize;

                let jmpct = position - buf.len() as isize - 2;

                match jmpct {
                    -128..=127 => {
                        buf.push(instruction::JMPFALSEP);
                        buf.push(jmpct as u8);
                    }
                    -32768..=32767 => {
                        buf.push(instruction::JMPFALSEWP);
                        buf.extend((jmpct as u16).to_ne_bytes());
                    }
                    _ => unreachable!("Jump offset out of range"),
                }
            } else {
                // TODO: handle JMPFALSEWP
                buf.push(byte);
            }
        }

        buf
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum Label {
    IfEnd,
    /// A branch of an if statement
    IfBranch(u16),
}

impl From<Vec<u8>> for InstructionBuilder {
    fn from(buf: Vec<u8>) -> Self {
        Self {
            buf,
            labels: HashMap::new(),
            jumps: Vec::new(),
        }
    }
}
