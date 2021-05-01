pub mod environment;
pub mod instruction;
pub mod stack;
pub mod value;

use std::{cell::RefCell, rc::Rc};

use instruction::{Instruction, Opcode};
use value::{JsString, JsValue, Object, Value};

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
    pub(crate) buffer: Box<dyn Iterator<Item = Instruction>>,
    pub(crate) stack: Stack<Rc<RefCell<Value>>, 512>,
    pub(crate) global: Environment,
}

impl VM {
    pub fn new(ins: Vec<Instruction>) -> Self {
        Self {
            buffer: Box::new(ins.into_iter()),
            stack: Stack::new(),
            global: Environment::new(),
        }
    }

    pub fn interpret(&mut self) -> Result<(), VMError> {
        while let Some(instruction) = self.buffer.next() {
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
                    let name = self.pop_owned().unwrap().into_ident();
                    let value = self.stack.pop();

                    self.global.set_var(name, value);
                }
                Opcode::GetGlobal => {
                    let name = self.pop_owned().unwrap().into_ident();

                    // TODO: handle case where var is not defined
                    let value = self.global.get_var(&name).unwrap();

                    self.stack.push(value.clone());
                }
                _ => unimplemented!(),
            };
        }

        // unreachable!()
        Ok(())
    }

    pub fn read_constant(&mut self) -> Option<Value> {
        self.buffer.next().map(|c| c.into_operand())
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
