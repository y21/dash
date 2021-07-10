use std::fmt::Display;

use crate::util::Either;

/// The type of a token
///
/// These are generated by the lexer, and used by the Parser to
/// produce an abstract syntax tree.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum TokenType {
    /// Left paren: (
    LeftParen,
    /// Right paren: )
    RightParen,
    /// Left brace: {
    LeftBrace,
    /// Right brace: }
    RightBrace,
    /// Left square brace: [
    LeftSquareBrace,
    /// Right square brace: ]
    RightSquareBrace,
    /// Comma: ,
    Comma,
    /// Dot: .
    Dot,
    /// Minus: -
    Minus,
    /// (Postfix) Decrement: x--
    Decrement,
    /// Plus: +
    Plus,
    /// Colon: :
    Colon,
    /// (Postfix) Increment: x++
    Increment,
    /// Prefix Increment: ++x
    PrefixIncrement,
    /// Prefix Decrement: --x
    PrefixDecrement,
    /// Explicit postfix increment: x++
    PostfixIncrement,
    /// Explicit postfix decrement: x--
    PostfixDecrement,
    /// Star: *
    Star,
    /// Slash: /
    Slash,
    /// Semicolon: ;
    Semicolon,
    /// Assignment: =
    Assignment,
    /// Addition assignment: +=
    AdditionAssignment,
    /// Subtraction assignment: -=
    SubtractionAssignment,
    /// Multiplication assignment: *=
    MultiplicationAssignment,
    /// Division assignment: /=
    DivisionAssignment,
    /// Remainder assignment: %=
    RemainderAssignment,
    /// Remainder: %
    Remainder,
    /// Exponetiation assignment: **=
    ExponentiationAssignment,
    /// Exponentiation: **
    Exponentiation,
    /// Left shift: <<
    LeftShift,
    /// Left shift assignment: <<=
    LeftShiftAssignment,
    /// Right shift assignment: >>=
    RightShiftAssignment,
    /// Right shift: >>
    RightShift,
    /// Unsigned right shift assignment: >>>=
    UnsignedRightShiftAssignment,
    /// Unsigned right shift: >>>
    UnsignedRightShift,
    /// Bitwise and assignment: &=
    BitwiseAndAssignment,
    /// Bitwise and: &
    BitwiseAnd,
    /// Bitwise or assignment: |=
    BitwiseOrAssignment,
    /// Bitwise or: |
    BitwiseOr,
    /// Bitwise xor assignment: ^=
    BitwiseXorAssignment,
    /// Bitwise xor: ^
    BitwiseXor,
    /// Bitwise not: ~
    BitwiseNot,
    /// Logical and assignment: &&=
    LogicalAndAssignment,
    /// Logical and: &&
    LogicalAnd,
    /// Logical or assignment: ||=
    LogicalOrAssignment,
    /// Logical or: ||
    LogicalOr,
    /// Logical nullish assignment: ??=
    LogicalNullishAssignment,
    /// Nullish coalescing: ??
    NullishCoalescing,
    /// Logical not: !
    LogicalNot,
    /// Equality: ==
    Equality,
    /// Strict equality: ===
    StrictEquality,
    /// Inequality: !=
    Inequality,
    /// Strict inequality: !==
    StrictInequality,
    /// Greater: >
    Greater,
    /// Greater equal: >=
    GreaterEqual,
    /// Less: <
    Less,
    /// Less equal: <=
    LessEqual,
    /// Identifier: foo
    Identifier,
    /// String: "foo"
    String,
    /// Number: 42
    Number,
    /// If: if
    If,
    /// Else: else
    Else,
    /// Function: function
    Function,
    /// Var: var
    Var,
    /// Let: let
    Let,
    /// Const: const
    Const,
    /// Return: return foo;
    Return,
    /// Throw: throw
    Throw,
    /// Try: try {}
    Try,
    /// Catch: catch {}
    Catch,
    /// Finally {}
    Finally,
    /// False literal: false
    FalseLit,
    /// True literal: true
    TrueLit,
    /// Null literal: null
    NullLit,
    /// Undefined literal: undefined
    UndefinedLit,
    /// Yield keyword: yield foo
    Yield,
    /// New keyword: new foo
    New,
    /// Condition: foo ? bar : baz
    Conditional,
    /// Optional chaining: ?.
    OptionalChaining,
    /// For: for(init; const; final)
    For,
    /// While: while(cond)
    While,
    /// In: foo in bar
    In,
    /// Instance of: foo instanceof bar
    Instanceof,
    /// Await keyword: await foo
    Await,
    /// Delete keyword: delete foo[bar]
    Delete,
    /// Void keyword: void expr
    Void,
    /// Typeof keyword: typeof expr
    Typeof,
    /// Error slot. Not a real token
    Error,
    /// Break keyword: break
    Break,
    /// Continue keyword: continue
    Continue,
    /// Import keyword: import foo from bar
    Import,
    /// Export keyword: export foo
    Export,
    /// Default keyword: export default foo
    Default,
    /// Debugger keyword: debugger
    Debugger,
    /// Arrow: =>
    ///
    /// Used for arrow functions
    Arrow,
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

/// Tokens that represent the variable kind
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

/// A token
#[derive(Debug, Clone, Copy)]
pub struct Token<'a> {
    /// The type of token
    pub ty: TokenType,
    /// The full string representation of this token
    pub full: &'a [u8],
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

impl Location {
    /// Formats this location.
    ///
    /// The caller is supposed to pass in the same input string.
    /// This function is used to format errors
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

/// The type of parser error
#[derive(Debug)]
pub enum ErrorKind<'a> {
    /// An unknown token was found
    UnknownToken(Token<'a>),
    /// An unexpected token was found
    UnexpectedToken(Token<'a>, TokenType),
    /// An unexpected token was found (one of many others)
    UnexpectedTokenMultiple(Token<'a>, &'static [TokenType]),
    /// Unexpected end of file
    UnexpectedEof,
}

/// An error that occurred during parsing
#[derive(Debug)]
pub struct Error<'a> {
    /// The type of error
    pub kind: ErrorKind<'a>,
    /// The source code string
    ///
    /// We need to carry it in errors to be able to format locations
    pub source: &'a [u8],
}

impl<'a> ErrorKind<'a> {
    /// Formats the error by calling to_string on the underlying Location
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
    /// Formats an error by calling to_string on the underlying [ErrorKind]
    pub fn to_string(&self) -> String {
        self.kind.to_string(self.source)
    }
}
