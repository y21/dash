use std::hash::Hash;
use std::ops::{Deref, DerefMut};

use dash_middle::compiler::instruction as inst;
use dash_middle::compiler::instruction::Instruction;

use crate::jump_container::JumpContainer;
use crate::{jump_container, FunctionCompiler};

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
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
    SwitchCaseCondition {
        case_id: u16,
    },
    SwitchCaseCode {
        case_id: u16,
    },
    SwitchEnd {
        switch_id: usize,
    },
    Catch,
    Finally {
        finally_id: usize,
    },
    TryEnd,
    InitParamWithDefaultValue,
    FinishParamDefaultValueInit,
    UserDefinedEnd {
        id: usize,
    },
}

pub struct InstructionBuilder<'cx, 'interner> {
    inner: &'cx mut FunctionCompiler<'interner>,
    jc: JumpContainer,
}

impl<'cx, 'interner> InstructionBuilder<'cx, 'interner> {
    pub fn new(fc: &'cx mut FunctionCompiler<'interner>) -> Self {
        Self {
            inner: fc,
            jc: JumpContainer::new(),
        }
    }

    pub fn append(&mut self, other: &mut Vec<u8>) {
        self.current_function_mut().buf.append(other)
    }

    pub fn write(&mut self, instruction: u8) {
        self.current_function_mut().buf.push(instruction);
    }

    pub fn write_instr(&mut self, instruction: Instruction) {
        self.current_function_mut().buf.push(instruction as u8);
    }

    pub fn writew(&mut self, instruction: u16) {
        self.current_function_mut()
            .buf
            .extend_from_slice(&instruction.to_ne_bytes());
    }

    pub fn write_all(&mut self, instruction: &[u8]) {
        self.current_function_mut().buf.extend_from_slice(instruction)
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
        if let Some(&inst::POP) = self.current_function_mut().buf.last() {
            self.current_function_mut().buf.pop();
        }
    }

    /// Adds a **local** label at the current instruction pointer, which can be jumped to using add_local_jump
    pub fn add_local_label(&mut self, label: Label) {
        jump_container::add_label(&mut self.jc, label, &mut self.inner.current_function_mut().buf)
    }

    /// Emits a jump instruction to a local label
    ///
    /// Requirement for calling this function: there must be two bytes in the buffer, reserved for this jump
    pub fn add_local_jump(&mut self, label: Label) {
        jump_container::add_jump(&mut self.jc, label, &mut self.inner.current_function_mut().buf)
    }
}

impl<'cx, 'interner> Deref for InstructionBuilder<'cx, 'interner> {
    type Target = FunctionCompiler<'interner>;
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'cx, 'interner> DerefMut for InstructionBuilder<'cx, 'interner> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}
