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

pub struct VM {
    buffer: Box<dyn Iterator<Item = Instruction>>,
    stack: Stack<Value, 512>
}

impl VM {
    pub fn new(ins: Vec<Instruction>) -> Self {
        Self {
            buffer: Box::new(ins.into_iter()),
            stack: Stack::new()
        }
    }
    
    pub fn interpret(mut self) -> Result<(), VMError> {
        while let Some(instruction) = self.buffer.next() {
            let instruction = instruction.into_op();

            match instruction {
                Opcode::Eof => return Ok(()),
                Opcode::Constant => {
                    let constant = self.read_constant()
                        .unwrap();

                    self.stack.push(constant);
                }
            };
        }

        unreachable!()
    }

    pub fn read_constant(&mut self) -> Option<Value> {
        self.buffer.next().map(|c| c.into_operand())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn aaa() {
        let vm = VM::new(vec![
            Instruction::Op(Opcode::Constant),
            Instruction::Operand(Value::Object(Box::new(JsString::new("aa".to_owned())))),
            Instruction::Op(Opcode::Eof)
        ]);

        let result = vm.interpret();

        dbg!(result);
    }
}