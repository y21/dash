use super::value::Value;

pub enum Opcode {
    Constant,
    Eof
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