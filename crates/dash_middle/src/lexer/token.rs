use std::fmt;

use derive_more::Display;
use either::Either;

/// The type of a token
///
/// These are generated by the lexer, and used by the Parser to
/// produce an abstract syntax tree.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Display)]
pub enum TokenType {
    #[display(fmt = "(")]
    LeftParen,

    #[display(fmt = ")")]
    RightParen,

    #[display(fmt = "{{")]
    LeftBrace,

    #[display(fmt = "}}")]
    RightBrace,

    #[display(fmt = "[")]
    LeftSquareBrace,

    #[display(fmt = "]")]
    RightSquareBrace,

    #[display(fmt = "[]")]
    EmptySquareBrace,

    #[display(fmt = ",")]
    Comma,

    #[display(fmt = ".")]
    Dot,

    #[display(fmt = "-")]
    Minus,

    #[display(fmt = "--")]
    Decrement,

    #[display(fmt = "+")]
    Plus,

    #[display(fmt = ":")]
    Colon,

    #[display(fmt = "++")]
    Increment,

    #[display(fmt = "++")]
    PrefixIncrement,

    #[display(fmt = "--")]
    PrefixDecrement,

    #[display(fmt = "++")]
    PostfixIncrement,

    #[display(fmt = "--")]
    PostfixDecrement,

    #[display(fmt = "*")]
    Star,

    #[display(fmt = "/")]
    Slash,

    #[display(fmt = ";")]
    Semicolon,

    #[display(fmt = "=")]
    Assignment,

    #[display(fmt = "+=")]
    AdditionAssignment,

    #[display(fmt = "-=")]
    SubtractionAssignment,

    #[display(fmt = "*=")]
    MultiplicationAssignment,

    #[display(fmt = "/=")]
    DivisionAssignment,

    #[display(fmt = "%=")]
    RemainderAssignment,

    #[display(fmt = "%")]
    Remainder,

    #[display(fmt = "**=")]
    ExponentiationAssignment,

    #[display(fmt = "**")]
    Exponentiation,

    #[display(fmt = "<<")]
    LeftShift,

    #[display(fmt = "<<=")]
    LeftShiftAssignment,

    #[display(fmt = ">>=")]
    RightShiftAssignment,

    #[display(fmt = ">>")]
    RightShift,

    #[display(fmt = ">>=")]
    UnsignedRightShiftAssignment,

    #[display(fmt = ">>>")]
    UnsignedRightShift,

    #[display(fmt = "&=")]
    BitwiseAndAssignment,

    #[display(fmt = "&")]
    BitwiseAnd,

    #[display(fmt = "|=")]
    BitwiseOrAssignment,

    #[display(fmt = "|")]
    BitwiseOr,

    #[display(fmt = "^=")]
    BitwiseXorAssignment,

    #[display(fmt = "^")]
    BitwiseXor,

    #[display(fmt = "~")]
    BitwiseNot,

    #[display(fmt = "&&=")]
    LogicalAndAssignment,

    #[display(fmt = "&&")]
    LogicalAnd,

    #[display(fmt = "||=")]
    LogicalOrAssignment,

    #[display(fmt = "||")]
    LogicalOr,

    #[display(fmt = "??=")]
    LogicalNullishAssignment,

    #[display(fmt = "??")]
    NullishCoalescing,

    #[display(fmt = "!")]
    LogicalNot,

    #[display(fmt = "==")]
    Equality,

    #[display(fmt = "===")]
    StrictEquality,

    #[display(fmt = "!=")]
    Inequality,

    #[display(fmt = "!==")]
    StrictInequality,

    #[display(fmt = ">")]
    Greater,

    #[display(fmt = ">=")]
    GreaterEqual,

    #[display(fmt = "<")]
    Less,

    #[display(fmt = "<=")]
    LessEqual,

    /// Identifier: foo
    Identifier,

    /// String: "foo"
    String,

    /// Number: 42
    NumberDec,

    #[display(fmt = "0x")]
    NumberHex,

    #[display(fmt = "0b")]
    NumberBin,

    #[display(fmt = "0o")]
    NumberOct,

    #[display(fmt = "if")]
    If,

    #[display(fmt = "else")]
    Else,

    #[display(fmt = "function")]
    Function,

    #[display(fmt = "class")]
    Class,

    #[display(fmt = "extends")]
    Extends,

    #[display(fmt = "static")]
    Static,

    #[display(fmt = "var")]
    Var,

    #[display(fmt = "let")]
    Let,

    #[display(fmt = "const")]
    Const,

    #[display(fmt = "return")]
    Return,

    #[display(fmt = "throw")]
    Throw,

    #[display(fmt = "try")]
    Try,

    #[display(fmt = "catch")]
    Catch,

    #[display(fmt = "finally")]
    Finally,

    #[display(fmt = "false")]
    FalseLit,

    #[display(fmt = "true")]
    TrueLit,

    #[display(fmt = "null")]
    NullLit,

    #[display(fmt = "undefined")]
    UndefinedLit,

    #[display(fmt = "yield")]
    Yield,

    #[display(fmt = "new")]
    New,

    /// Condition: foo ? bar : baz
    Conditional,

    #[display(fmt = "?.")]
    OptionalChaining,

    #[display(fmt = "for")]
    For,

    #[display(fmt = "while")]
    While,

    #[display(fmt = "in")]
    In,

    #[display(fmt = "instanceof")]
    Instanceof,

    #[display(fmt = "await")]
    Await,

    #[display(fmt = "delete")]
    Delete,

    #[display(fmt = "void")]
    Void,

    #[display(fmt = "typeof")]
    Typeof,

    #[display(fmt = "break")]
    Break,

    #[display(fmt = "continue")]
    Continue,

    #[display(fmt = "import")]
    Import,

    #[display(fmt = "export")]
    Export,

    #[display(fmt = "default")]
    Default,

    #[display(fmt = "debugger")]
    Debugger,

    #[display(fmt = "of")]
    Of,

    #[display(fmt = "=>")]
    FatArrow,

    #[display(fmt = "#")]
    Hash,

    #[display(fmt = "switch")]
    Switch,

    #[display(fmt = "case")]
    Case,

    #[display(fmt = "EOF")]
    Eof,
}

/// Tokens that are used to assign
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

pub const VARIABLE_TYPES: &[TokenType] = &[TokenType::Var, TokenType::Let, TokenType::Const];

impl TokenType {
    /// Checks if this token is a variable kind
    pub fn is_variable(&self) -> bool {
        VARIABLE_TYPES.contains(self)
    }
}

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
            b"of" => Self::Of,
            b"class" => Self::Class,
            b"extends" => Self::Extends,
            b"static" => Self::Static,
            b"switch" => Self::Switch,
            b"case" => Self::Case,
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

/// A token
#[derive(Debug, Clone, Copy)]
pub struct Token<'a> {
    /// The type of token
    pub ty: TokenType,
    /// The full string representation of this token
    pub full: &'a str,
    /// Location of this token in the input string
    pub loc: Location,
}

/// A location, represents where a token can be found in a source code string
#[derive(Debug, Clone, Copy)]
pub struct Location {
    /// Line number
    pub line: usize,
    /// Byte offset
    pub offset: usize,
    /// Byte offset for the line this token is on
    pub line_offset: usize,
}

pub struct FormattableError<'a> {
    pub loc: &'a Location,
    pub source: &'a [u8],
    pub tok: Either<&'a str, char>,
    pub message: &'a str,
    pub display_token: bool,
    pub help: Option<Box<dyn fmt::Display + 'a>>,
}

impl fmt::Display for FormattableError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let offset = if self.loc.line <= 1 {
            self.loc.line_offset
        } else {
            self.loc.line_offset + 1
        };

        let column = self.loc.offset - offset;

        write!(f, "error: {}\n", self.message)?;
        write!(f, "--> script.js:{}:{}\n\n", self.loc.line, column)?;

        let line = {
            let partial = &self.source[offset..];
            let end = partial.iter().position(|&c| c == b'\n').unwrap_or(partial.len());
            &partial[..end]
        };

        let token_len = match self.tok {
            Either::Left(s) => s.len(),
            Either::Right(c) => c.len_utf8(),
        };

        let pointer_start = self.loc.offset - self.loc.line_offset;

        write!(f, "{}\n", String::from_utf8_lossy(line))?;
        write!(f, "{}", " ".repeat(pointer_start))?;
        write!(f, "{}", "^".repeat(token_len))?;

        if let Some(help) = &self.help {
            write!(f, "\n= help: {}", help)?;
        }

        Ok(())
    }
}
