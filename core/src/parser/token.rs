#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Decrement,
    Plus,
    Increment,
    Star,
    Slash,
    Semicolon,
    Assignment,
    AdditionAssignment,
    SubtractionAssignment,
    MultiplicationAssignment,
    DivisionAssignment,
    RemainderAssignment,
    Remainder,
    ExponentiationAssignment,
    Exponential,
    LeftShiftAssignment,
    RightShiftAssignment,
    RightShift,
    UnsignedRightShiftAssignment,
    UnsignedRightShift,
    BitwiseAndAssignment,
    BitwiseAnd,
    BitwiseOrAssignment,
    BitwiseOr,
    BitwiseXorAssignment,
    BitwiseXor,
    BitwiseNot,
    LogicalAndAssignment,
    LogicalAnd,
    LogicalOrAssignment,
    LogicalOr,
    LogicalNullishAssignment,
    LogicalNot,
    Equality,
    StrictEquality,
    Inequality,
    StrictInequality,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Identifier,
    String,
    Number,
    If,
    Else,
    Function,
    Var,
    Let,
    Const,
    Return,
    FalseLit,
    TrueLit,
    NullLit,
    UndefinedLit,
    Yield,
    New,
    Conditional,
    OptionalChaining,
}

pub const ASSIGNMENT_TYPES: &[TokenType] = &[
    TokenType::Assignment,
    TokenType::AdditionAssignment,
    TokenType::SubtractionAssignment,
    TokenType::MultiplicationAssignment,
    TokenType::DivisionAssignment,
    TokenType::RemainderAssignment,
    TokenType::ExponentiationAssignment,
    TokenType::LeftShiftAssignment,
    TokenType::RightShiftAssignment,
    TokenType::UnsignedRightShiftAssignment,
    TokenType::BitwiseAndAssignment,
    TokenType::BitwiseOrAssignment,
    TokenType::BitwiseXorAssignment,
    TokenType::LogicalAndAssignment,
    TokenType::LogicalOrAssignment,
    TokenType::LogicalNullishAssignment,
];

impl From<&[u8]> for TokenType {
    fn from(s: &[u8]) -> Self {
        match s {
            b"if" => Self::If,
            b"else" => Self::Else,
            b"function" => Self::Function,
            b"var" => Self::Var,
            b"let" => Self::Let,
            b"const" => Self::Const,
            b"return" => Self::Return,
            b"true" => Self::TrueLit,
            b"false" => Self::FalseLit,
            b"null" => Self::NullLit,
            b"undefined" => Self::UndefinedLit,
            b"yield" => Self::Yield,
            b"new" => Self::New,
            _ => Self::Identifier,
        }
    }
}

impl From<TokenType> for &str {
    fn from(tt: TokenType) -> Self {
        match tt {
            TokenType::Plus => "+",
            TokenType::Minus => "-",
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug)]
pub struct Token<'a> {
    pub ty: TokenType,
    pub full: &'a [u8],
    pub loc: Location,
}

#[derive(Debug)]
pub struct Location {
    pub line: usize,
}
