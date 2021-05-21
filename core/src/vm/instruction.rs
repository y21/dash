use crate::parser::token::TokenType;

use super::value::Value;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Opcode {
    Constant,
    Eof,
    SetLocalNoValue,
    SetLocal,
    SetUpvalue,
    UpvalueLocal,
    UpvalueNonLocal,
    GetLocal,
    GetUpvalue,
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
    Assignment,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Negate, // TODO: ~
    LogicalNot,
    ShortJmp,
    ShortJmpIfFalse,
    ShortJmpIfTrue,
    LongJmp,
    BackJmp,
    Pop,
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
    Closure,
    Equality,
    StrictEquality,
    PostfixIncrement,
    PostfixDecrement,
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
            TokenType::PrefixIncrement => Self::AdditionAssignment,
            TokenType::PrefixDecrement => Self::SubtractionAssignment,
            TokenType::PostfixIncrement | TokenType::Increment => Self::PostfixIncrement,
            TokenType::PostfixDecrement | TokenType::Decrement => Self::PostfixDecrement,
            TokenType::Assignment => Self::Assignment,
            TokenType::Less => Self::Less,
            TokenType::LessEqual => Self::LessEqual,
            TokenType::Greater => Self::Greater,
            TokenType::GreaterEqual => Self::GreaterEqual,
            TokenType::Equality => Self::Equality,
            TokenType::StrictEquality => Self::StrictEquality,
            _ => unimplemented!("{:?}", tt),
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
