#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftSquareBrace,
    RightSquareBrace,
    Comma,
    Dot,
    Minus,
    Decrement,
    Plus,
    Colon,
    Increment,
    PrefixIncrement,
    PrefixDecrement,
    PostfixIncrement,
    PostfixDecrement,
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
    Exponentiation,
    LeftShift,
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
    NullishCoalescing,
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
    Throw,
    Try,
    Catch,
    Finally,
    FalseLit,
    TrueLit,
    NullLit,
    UndefinedLit,
    Yield,
    New,
    Conditional,
    OptionalChaining,
    For,
    While,
    In,
    Instanceof,
    Await,
    Delete,
    Void,
    Typeof,
    Error,
    Break,
    Continue,
    Import,
    Export,
    Default,
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

pub const VARIABLE_TYPES: &[TokenType] = &[TokenType::Let, TokenType::Const, TokenType::Var];

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
            b"throw" => Self::Throw,
            b"try" => Self::Try,
            b"catch" => Self::Catch,
            b"finally" => Self::Finally,
            b"true" => Self::TrueLit,
            b"false" => Self::FalseLit,
            b"null" => Self::NullLit,
            b"undefined" => Self::UndefinedLit,
            b"yield" => Self::Yield,
            b"new" => Self::New,
            b"for" => Self::For,
            b"while" => Self::While,
            b"in" => Self::In,
            b"instanceof" => Self::Instanceof,
            b"await" => Self::Await,
            b"delete" => Self::Delete,
            b"void" => Self::Void,
            b"typeof" => Self::Typeof,
            b"continue" => Self::Continue,
            b"break" => Self::Break,
            b"import" => Self::Import,
            b"export" => Self::Export,
            b"default" => Self::Default,
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

#[derive(Debug, Clone, Copy)]
pub struct Token<'a> {
    pub ty: TokenType,
    pub full: &'a [u8],
    pub loc: Location,
}

#[derive(Debug, Clone, Copy)]
pub struct Location {
    pub line: usize,
}

#[derive(Debug)]
pub enum ErrorKind<'a> {
    UnknownToken(Token<'a>),
    UnexpectedToken(Token<'a>, TokenType),
    UnexpectedTokenMultiple(Token<'a>, &'static [TokenType]),
}

#[derive(Debug)]
pub struct Error<'a> {
    pub kind: ErrorKind<'a>,
}
