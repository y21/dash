use crate::parser::token::TokenType;

use super::value::Value;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Opcode {
    Constant,
    Eof,
    SetLocalNoValue,
    SetLocal,
    GetLocal,
    GetLocalRef,
    GetGlobalRef,
    SetGlobalNoValue,
    SetGlobal,
    GetGlobal,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    AdditionAssignment,
    SubtractionAssignment,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Negate, // TODO: ~ ! -
    ShortJmp,
    ShortJmpIfFalse,
    ShortJmpIfTrue,
    LongJmp,
    BackJmp,
    Pop,
    Print,
    FunctionCall,
    Return,
    Nop, // Mainly used as a placeholder
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    StaticPropertyAccess,
    ComputedPropertyAccess,
    Typeof,
}

impl From<TokenType> for Opcode {
    fn from(tt: TokenType) -> Self {
        match tt {
            TokenType::Plus => Self::Add,
            TokenType::Minus => Self::Sub,
            TokenType::Star => Self::Mul,
            TokenType::Slash => Self::Div,
            TokenType::Remainder => Self::Rem,
            TokenType::BitwiseAnd => Self::BitwiseAnd,
            TokenType::BitwiseOr => Self::BitwiseOr,
            TokenType::BitwiseXor => Self::BitwiseXor,
            TokenType::AdditionAssignment => Self::AdditionAssignment,
            TokenType::SubtractionAssignment => Self::SubtractionAssignment,
            TokenType::Increment => Self::AdditionAssignment,
            TokenType::Decrement => Self::SubtractionAssignment,
            TokenType::Less => Self::Less,
            TokenType::LessEqual => Self::LessEqual,
            TokenType::Greater => Self::Greater,
            TokenType::GreaterEqual => Self::GreaterEqual,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone)]
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

    pub fn as_op(&self) -> Opcode {
        match self {
            Self::Op(o) => *o,
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
