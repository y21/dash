pub mod instruction;
pub mod stack;
pub mod value;

use value::{JsValue, Value, Object, JsString};
use instruction::{
    Instruction,
    Opcode
};

use self::stack::Stack;

#[derive(Debug)]
pub enum VMError {

}

macro_rules! binary_op {
    ($self:ident, $op:tt) => {
        let (b, a) = (
            $self.read_number().unwrap(),
            $self.read_number().unwrap()
        );

        $self.stack.push(Value::Number(a $op b));
    }
}

pub struct VM {
    pub(crate) buffer: Box<dyn Iterator<Item = Instruction>>,
    pub(crate) stack: Stack<Value, 512>
}

impl VM {
    pub fn new(ins: Vec<Instruction>) -> Self {
        Self {
            buffer: Box::new(ins.into_iter()),
            stack: Stack::new()
        }
    }
    
    pub fn interpret(&mut self) -> Result<(), VMError> {
        while let Some(instruction) = self.buffer.next() {
            let instruction = instruction.into_op();

            match instruction {
                Opcode::Eof => return Ok(()),
                Opcode::Constant => {
                    let constant = self.read_constant()
                        .unwrap();

                    self.stack.push(constant);
                },
                Opcode::Negate => {
                    let maybe_number = self.read_number()
                        .unwrap();
                    
                    self.stack.push(Value::Number(-maybe_number));
                },
                Opcode::Add => { binary_op!(self, +); },
                Opcode::Sub => { binary_op!(self, -); },
                Opcode::Mul => { binary_op!(self, *); },
                Opcode::Div => { binary_op!(self, /); },
                _ => unimplemented!()
            };
        }

        unreachable!()
    }

    pub fn read_constant(&mut self) -> Option<Value> {
        self.buffer.next().map(|c| c.into_operand())
    }

    pub fn read_number(&mut self) -> Option<f64> {
        self.stack.pop().as_number()
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
            Instruction::Op(Opcode::Eof)
        ]);

        
        let result = vm.interpret();
    }
}