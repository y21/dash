use std::fmt::Display;

use crate::util::Either;

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
    Debugger,
    Arrow,
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
            b"debugger" => Self::Debugger,
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
    pub offset: usize,
    pub line_offset: usize,
}

impl Location {
    pub fn to_string(&self, source: &[u8], full: Either<&str, char>, message: &str) -> String {
        let offset = if self.line <= 1 {
            self.line_offset
        } else {
            self.line_offset + 1
        };

        let line_partial = &source[offset..];
        let end = line_partial
            .iter()
            .position(|&c| c == b'\n')
            .unwrap_or(line_partial.len());
        let line = &line_partial[..end];

        let col = self.offset - offset;

        let full_len = full.as_left().map(|s| s.len()).unwrap_or(1);
        let full = match &full {
            Either::Left(l) => l as &dyn Display,
            Either::Right(r) => r as &dyn Display,
        };

        let mut s = std::str::from_utf8(line).unwrap().to_owned();
        s.push('\n');
        s.push_str(&" ".repeat((self.offset - self.line_offset).saturating_sub(1)));
        s.push_str(&"^".repeat(full_len));
        s.push_str(&format!(
            " {}: {}\n  at script.js:{}:{}",
            message, full, self.line, col
        ));

        s
    }
}

#[derive(Debug)]
pub enum ErrorKind<'a> {
    UnknownToken(Token<'a>),
    UnexpectedToken(Token<'a>, TokenType),
    UnexpectedTokenMultiple(Token<'a>, &'static [TokenType]),
    UnexpectedEof,
}

#[derive(Debug)]
pub struct Error<'a> {
    pub kind: ErrorKind<'a>,
    pub source: &'a [u8],
}

impl<'a> ErrorKind<'a> {
    pub fn to_string(&self, source: &[u8]) -> String {
        match self {
            Self::UnknownToken(tok) => {
                let full_utf8 = std::str::from_utf8(tok.full).unwrap();
                tok.loc
                    .to_string(source, Either::Left(full_utf8), "unknown token")
            }
            Self::UnexpectedToken(tok, _) | Self::UnexpectedTokenMultiple(tok, _) => {
                let full_utf8 = std::str::from_utf8(tok.full).unwrap();
                tok.loc
                    .to_string(source, Either::Left(full_utf8), "unexpected token")
            }
            Self::UnexpectedEof => String::from("unexpected end of input"),
        }
    }
}

impl<'a> Error<'a> {
    pub fn to_string(&self) -> String {
        self.kind.to_string(self.source)
    }
}
