pub mod environment;
pub mod instruction;
pub mod stack;
pub mod value;

use std::{cell::RefCell, rc::Rc};

use instruction::{Instruction, Opcode};
use value::{JsValue, Value};

use self::{environment::Environment, stack::Stack};

#[derive(Debug)]
pub enum VMError {}

macro_rules! binary_op {
    ($self:ident, $op:tt) => {
        let (b, a) = (
            $self.read_number().unwrap(),
            $self.read_number().unwrap()
        );

        $self.stack.push(Rc::new(RefCell::new(Value::Number(a $op b))));
    }
}

pub struct VM {
    /// Bytecode
    pub(crate) buffer: Vec<Instruction>,
    /// Stack
    pub(crate) stack: Stack<Rc<RefCell<Value>>, 512>,
    /// Global namespace
    pub(crate) global: Environment,
    /// Instruction pointer
    pub(crate) ip: usize,
}

impl VM {
    pub fn new(ins: Vec<Instruction>) -> Self {
        Self {
            buffer: ins,
            stack: Stack::new(),
            global: Environment::new(),
            ip: 0,
        }
    }

    fn is_eof(&self) -> bool {
        self.ip >= self.buffer.len()
    }

    pub fn interpret(&mut self) -> Result<(), VMError> {
        while !self.is_eof() {
            let instruction = self.buffer.remove(self.ip); //&self.buffer[self.ip];
            let instruction = instruction.into_op();

            match instruction {
                Opcode::Eof => return Ok(()),
                Opcode::Constant => {
                    let constant = self.read_constant().unwrap();

                    self.stack.push(Rc::new(RefCell::new(constant)));
                }
                Opcode::Negate => {
                    let maybe_number = self.read_number().unwrap();

                    self.stack
                        .push(Rc::new(RefCell::new(Value::Number(-maybe_number))));
                }
                Opcode::Add => {
                    binary_op!(self, +);
                }
                Opcode::Sub => {
                    binary_op!(self, -);
                }
                Opcode::Mul => {
                    binary_op!(self, *);
                }
                Opcode::Div => {
                    binary_op!(self, /);
                }
                Opcode::Rem => {
                    binary_op!(self, %);
                }
                Opcode::SetGlobal => {
                    let name = self.pop_owned().unwrap().into_ident().unwrap();
                    let value = self.stack.pop();

                    self.global.set_var(name, value);
                }
                Opcode::GetGlobal => {
                    let name = self.pop_owned().unwrap().into_ident().unwrap();

                    // TODO: handle case where var is not defined
                    let value = self.global.get_var(&name).unwrap();

                    self.stack.push(value.clone());
                }
                Opcode::ShortJmpIfFalse => {
                    let instruction_count = self.pop_owned().unwrap().as_number().unwrap() as usize;

                    let condition_cell = self.stack.pop();
                    let condition = condition_cell.borrow().is_truthy();

                    if !condition {
                        self.ip += instruction_count;
                    }
                }
                _ => unimplemented!(),
            };
        }

        // unreachable!()
        Ok(())
    }

    pub fn read_constant(&mut self) -> Option<Value> {
        if self.is_eof() {
            return None;
        }

        Some(self.buffer.remove(self.ip).into_operand())
    }

    pub fn read_number(&mut self) -> Option<f64> {
        self.stack.pop().borrow().as_number()
    }

    pub fn pop_owned(&mut self) -> Option<Value> {
        Value::try_into_inner(self.stack.pop())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn aaa() {
        let mut vm = VM::new(vec![
            Instruction::Op(Opcode::Constant),
            Instruction::Operand(Value::Number(5.0)),
            Instruction::Op(Opcode::Constant),
            Instruction::Operand(Value::Number(123.0)),
            Instruction::Op(Opcode::Sub),
            Instruction::Op(Opcode::Eof),
        ]);

        let result = vm.interpret();
    }
}
