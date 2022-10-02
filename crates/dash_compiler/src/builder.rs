use std::convert::TryInto;
use std::hash::Hash;
use std::ops::Deref;
use std::ops::DerefMut;

use dash_middle::compiler::instruction as inst;
use dash_middle::compiler::instruction::Instruction;

use crate::jump_container;
use crate::jump_container::JumpContainer;
use crate::FunctionCompiler;

#[derive(PartialOrd, Ord, Hash, Eq, PartialEq, Debug, Clone)]
pub enum Label {
    IfEnd,
    /// A branch of an if statement
    IfBranch {
        branch_id: usize,
    },
    LoopCondition {
        loop_id: usize,
    },
    LoopEnd {
        loop_id: usize,
    },
    LoopIncrement {
        loop_id: usize,
    },
    SwitchCase {
        case_id: u16,
    },
    SwitchEnd {
        switch_id: usize,
    },
    Catch,
    TryEnd,
}

pub struct InstructionBuilder<'cx, 'inp> {
    inner: &'cx mut FunctionCompiler<'inp>,
    jc: JumpContainer,
}

impl<'cx, 'inp> InstructionBuilder<'cx, 'inp> {
    pub fn new(fc: &'cx mut FunctionCompiler<'inp>) -> Self {
        Self {
            inner: fc,
            jc: JumpContainer::new(),
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
        jump_container::add_label(&mut self.jc, label, &mut self.inner.buf)
    }

    /// Emits a jump instruction to a local label
    ///
    /// Requirement for calling this function: there must be two bytes in the buffer, reserved for this jump
    pub fn add_local_jump(&mut self, label: Label) {
        jump_container::add_jump(&mut self.jc, label, &mut self.inner.buf)
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
