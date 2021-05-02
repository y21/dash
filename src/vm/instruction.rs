use crate::parser::token::TokenType;

use super::value::Value;

#[derive(Debug)]
pub enum Opcode {
    Constant,
    Eof,
    SetLocalNoValue,
    SetLocal,
    GetLocal,
    SetGlobalNoValue,
    SetGlobal,
    GetGlobal,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Negate, // TODO: ~ ! -
    ShortJmp,
    ShortJmpIfFalse,
    LongJmp,
}

impl From<TokenType> for Opcode {
    fn from(tt: TokenType) -> Self {
        match tt {
            TokenType::Plus => Self::Add,
            TokenType::Minus => Self::Sub,
            TokenType::Star => Self::Mul,
            TokenType::Slash => Self::Div,
            TokenType::Remainder => Self::Rem,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug)]
pub enum Instruction {
    Op(Opcode),
    Operand(Value),
}

impl Instruction {
    pub fn into_op(self) -> Opcode {
        match self {
            Self::Op(o) => o,
            _ => unreachable!(),
        }
    }

    pub fn into_operand(self) -> Value {
        match self {
            Self::Operand(o) => o,
            _ => unreachable!(),
        }
    }
}
