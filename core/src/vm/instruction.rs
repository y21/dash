use crate::parser::token::TokenType;

use super::value::{Value, ValueKind};

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
    GetThis,
    GetSuper,
    GetGlobalThis,
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
    MultiplicationAssignment,
    DivisionAssignment,
    RemainderAssignment,
    ExponentiationAssignment,
    LeftShiftAssignment,
    RightShiftAssignment,
    UnsignedRightShiftAssignment,
    BitwiseAndAssignment,
    BitwiseOrAssignment,
    BitwiseXorAssignment,
    LogicalAndAssignment,
    LogicalOrAssignment,
    LogicalNullishAssignment,
    Assignment,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Exponentiation,
    LeftShift,
    RightShift,
    UnsignedRightShift,
    Positive,
    Negate,
    LogicalNot,
    BitwiseNot,
    ShortJmp,
    ShortJmpIfFalse,
    ShortJmpIfTrue,
    ShortJmpIfNullish,
    LongJmp,
    BackJmp,
    Pop,
    PopUnwindHandler,
    FunctionCall,
    ConstructorCall,
    Return,
    ReturnModule,
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
    Inequality,
    StrictEquality,
    StrictInequality,
    PostfixIncrement,
    PostfixDecrement,
    Void,
    ArrayLiteral,
    ObjectLiteral,
    Try,
    Throw,
    Continue,
    Break,
    LoopStart,
    LoopEnd,
    EvaluateModule,
    ExportDefault,
    ToPrimitive,
    Debugger,
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
            TokenType::LeftShift => Self::LeftShift,
            TokenType::RightShift => Self::RightShift,
            TokenType::Exponentiation => Self::Exponentiation,
            TokenType::AdditionAssignment => Self::AdditionAssignment,
            TokenType::SubtractionAssignment => Self::SubtractionAssignment,
            TokenType::MultiplicationAssignment => Self::MultiplicationAssignment,
            TokenType::DivisionAssignment => Self::DivisionAssignment,
            TokenType::RemainderAssignment => Self::RemainderAssignment,
            TokenType::ExponentiationAssignment => Self::ExponentiationAssignment,
            TokenType::LeftShiftAssignment => Self::LeftShiftAssignment,
            TokenType::RightShiftAssignment => Self::RightShiftAssignment,
            TokenType::UnsignedRightShiftAssignment => Self::UnsignedRightShiftAssignment,
            TokenType::BitwiseAndAssignment => Self::BitwiseAndAssignment,
            TokenType::BitwiseOrAssignment => Self::BitwiseOrAssignment,
            TokenType::BitwiseXorAssignment => Self::BitwiseXorAssignment,
            TokenType::LogicalAndAssignment => Self::LogicalAndAssignment,
            TokenType::LogicalOrAssignment => Self::LogicalOrAssignment,
            TokenType::LogicalNullishAssignment => Self::LogicalNullishAssignment,
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
            TokenType::Inequality => Self::Inequality,
            TokenType::StrictEquality => Self::StrictEquality,
            TokenType::StrictInequality => Self::StrictInequality,
            _ => unimplemented!("{:?}", tt),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Constant {
    JsValue(Value),
    Identifier(String),
    Index(usize),
}

impl Constant {
    pub fn into_value(self) -> Option<Value> {
        match self {
            Self::JsValue(v) => Some(v),
            _ => None,
        }
    }

    pub fn try_into_value(self) -> Value {
        match self {
            Self::JsValue(v) => v,
            _ => Value::new(ValueKind::Constant(Box::new(self))),
        }
    }

    pub fn into_ident(self) -> Option<String> {
        match self {
            Self::Identifier(ident) => Some(ident),
            _ => None,
        }
    }

    pub fn into_index(self) -> Option<usize> {
        match self {
            Self::Index(idx) => Some(idx),
            _ => None,
        }
    }

    pub fn as_index(&self) -> Option<usize> {
        match self {
            Self::Index(idx) => Some(*idx),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Op(Opcode),
    Operand(Constant),
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

    pub fn into_operand(self) -> Constant {
        match self {
            Self::Operand(o) => o,
            _ => unreachable!(),
        }
    }
}
