use std::collections::BTreeMap;
use std::convert::TryInto;
use std::hash::Hash;

use super::instruction;

pub struct InstructionBuilder {
    buf: Vec<u8>,
    jumps: BTreeMap<Label, Vec<usize>>,
    labels: BTreeMap<Label, usize>,
}

impl InstructionBuilder {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            labels: BTreeMap::new(),
            jumps: BTreeMap::new(),
        }
    }

    pub fn append(&mut self, other: &mut Vec<u8>) {
        self.buf.append(other)
    }

    pub fn write(&mut self, instruction: u8) {
        self.buf.push(instruction);
    }

    pub fn writew(&mut self, instruction: u16) {
        self.buf.extend_from_slice(&instruction.to_ne_bytes());
    }

    pub fn write_all(&mut self, instruction: &[u8]) {
        self.buf.extend_from_slice(instruction)
    }

    pub fn write_wide_instr(&mut self, instr: u8, instrw: u8, value: u16) {
        if let Ok(value) = value.try_into() {
            self.write_all(&[instr, value]);
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
        let ip = self.buf.len();

        // get vector of existing jumps to this label
        if let Some(assoc_jumps) = self.jumps.remove(&label) {
            for jump in assoc_jumps {
                let offset = (ip - jump - 2) as u16; // TODO: don't hardcast..? and use i16

                // write jump offset
                let pt = &mut self.buf[jump..jump + 2];
                pt.copy_from_slice(&u16::to_ne_bytes(offset));
            }
        }

        self.labels.insert(label, ip);
    }

    /// Requirement for calling this function: there must be two bytes in the buffer, reserved for this jump
    pub fn add_jump(&mut self, label: Label) {
        if let Some(&ip) = self.labels.get(&label) {
            let ip = ip as isize;
            let len = self.buf.len() as isize;
            let offset = (ip - len) as i16; // TODO: don't hardcast..?

            let pt = &mut self.buf[len as usize - 2..];
            pt.copy_from_slice(&i16::to_ne_bytes(offset));
        } else {
            self.jumps
                .entry(label)
                .or_insert_with(Vec::new)
                .push(self.buf.len() - 2);
        }
    }

    pub fn build(self) -> Vec<u8> {
        debug_assert!(self.jumps.is_empty(), "Unresolved jumps");

        self.buf
    }
}

#[derive(PartialOrd, Ord, Hash, Eq, PartialEq, Debug, Clone)]
pub enum Label {
    IfEnd,
    /// A branch of an if statement
    IfBranch(u16),
    LoopCondition,
    LoopEnd,
    Catch,
    TryEnd,
}

impl From<Vec<u8>> for InstructionBuilder {
    fn from(buf: Vec<u8>) -> Self {
        Self {
            buf,
            labels: BTreeMap::new(),
            jumps: BTreeMap::new(),
        }
    }
}
