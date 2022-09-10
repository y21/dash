use std::collections::BTreeMap;
use std::convert::TryInto;
use std::hash::Hash;
use std::ops::Deref;
use std::ops::DerefMut;

use dash_middle::compiler::instruction as inst;
use dash_middle::compiler::instruction::Instruction;

use crate::FunctionCompiler;

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

pub struct InstructionBuilder<'cx, 'inp> {
    inner: &'cx mut FunctionCompiler<'inp>,
    jumps: BTreeMap<Label, Vec<usize>>,
    labels: BTreeMap<Label, usize>,
}

impl<'cx, 'inp> InstructionBuilder<'cx, 'inp> {
    pub fn new(fc: &'cx mut FunctionCompiler<'inp>) -> Self {
        Self {
            inner: fc,
            jumps: BTreeMap::new(),
            labels: BTreeMap::new(),
        }
    }

    pub fn append(&mut self, other: &mut Vec<u8>) {
        self.buf.append(other)
    }

    pub fn write(&mut self, instruction: u8) {
        self.buf.push(instruction);
    }

    pub fn write_instr(&mut self, instruction: Instruction) {
        self.buf.push(instruction as u8);
    }

    pub fn writew(&mut self, instruction: u16) {
        self.buf.extend_from_slice(&instruction.to_ne_bytes());
    }

    pub fn write_all(&mut self, instruction: &[u8]) {
        self.buf.extend_from_slice(instruction)
    }

    pub fn write_wide_instr(&mut self, instr: Instruction, instrw: Instruction, value: u16) {
        if let Ok(value) = value.try_into() {
            self.write_instr(instr);
            self.write(value);
        } else {
            self.write_instr(instrw);
            self.writew(value);
        }
    }

    pub fn remove_pop_end(&mut self) {
        if let Some(&inst::POP) = self.buf.last() {
            self.buf.pop();
        }
    }

    /// Adds a **local** label at the current instruction pointer, which can be jumped to using add_local_jump
    pub fn add_local_label(&mut self, label: Label) {
        let ip = self.inner.buf.len();

        // get vector of existing jumps to this label
        if let Some(assoc_jumps) = self.jumps.remove(&label) {
            for jump in assoc_jumps {
                let offset = (ip - jump - 2) as u16; // TODO: don't hardcast..? and use i16

                // write jump offset
                let pt = &mut self.inner.buf[jump..jump + 2];
                pt.copy_from_slice(&u16::to_ne_bytes(offset));
            }
        }

        self.labels.insert(label, ip);
    }

    /// Emits a jump instruction to a local label
    ///
    /// Requirement for calling this function: there must be two bytes in the buffer, reserved for this jump
    pub fn add_local_jump(&mut self, label: Label) {
        if let Some(&ip) = self.labels.get(&label) {
            let ip = ip as isize;
            let len = self.inner.buf.len() as isize;
            let offset = (ip - len) as i16; // TODO: don't hardcast..?

            let pt = &mut self.inner.buf[len as usize - 2..];
            pt.copy_from_slice(&i16::to_ne_bytes(offset));
        } else {
            self.jumps
                .entry(label)
                .or_insert_with(Vec::new)
                .push(self.inner.buf.len() - 2);
        }
    }
}

impl<'cx, 'inp> Deref for InstructionBuilder<'cx, 'inp> {
    type Target = FunctionCompiler<'inp>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'cx, 'inp> DerefMut for InstructionBuilder<'cx, 'inp> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
