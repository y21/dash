use super::value::Value;

#[derive(Debug)]
pub enum Opcode {
    Constant,
    Eof,

    Add,
    Sub,
    Mul,
    Div,
    Negate
}

pub enum Instruction {
    Op(Opcode),
    Operand(Value)
}

impl Instruction {
    pub fn into_op(self) -> Opcode {
        match self {
            Self::Op(o) => o,
            _ => unreachable!()
        }
    }

    pub fn into_operand(self) -> Value {
        match self {
            Self::Operand(o) => o,
            _ => unreachable!()
        }
    }
}