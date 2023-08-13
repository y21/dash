use std::fmt;

use crate::interner::sym;
use crate::interner::Symbol;
use crate::sourcemap::Span;
use derive_more::Display;

/// The type of a token
///
/// These are generated by the lexer, and used by the Parser to
/// produce an abstract syntax tree.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Display)]
pub enum TokenType {
    #[display(fmt = "(")]
    LeftParen,

    #[display(fmt = ")")]
    RightParen,

    #[display(fmt = "{{")]
    LeftBrace,

    #[display(fmt = "}}")]
    RightBrace,

    #[display(fmt = "$")]
    Dollar,

    #[display(fmt = "[")]
    LeftSquareBrace,

    #[display(fmt = "]")]
    RightSquareBrace,

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
    ///
    /// Note: for checking if a [`TokenType`] is an identifier, use the is_identifier method, as
    /// there are certain tokens that are valid identifiers but aren't necessarily of the type `TokenType::Identifier`
    #[display(fmt = "<ident>")]
    Identifier(Symbol),

    /// String: "foo"
    String(Symbol),

    /// Template literal segment: `foo`
    #[display(fmt = "<template literal>")]
    TemplateLiteral(Symbol),

    /// Number: 42
    NumberDec(Symbol),

    /// Regex literal: /a+b/g
    #[display(fmt = "<regex literal>")]
    RegexLiteral(Symbol),

    #[display(fmt = "0x")]
    NumberHex(Symbol),

    #[display(fmt = "0b")]
    NumberBin(Symbol),

    #[display(fmt = "0o")]
    NumberOct(Symbol),

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

    #[display(fmt = "do")]
    Do,

    #[display(fmt = "in")]
    In,

    #[display(fmt = "instanceof")]
    Instanceof,

    #[display(fmt = "await ")]
    Await,

    #[display(fmt = "delete ")]
    Delete,

    #[display(fmt = "void ")]
    Void,

    #[display(fmt = "typeof ")]
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

    #[display(fmt = "get")]
    Get,

    #[display(fmt = "set")]
    Set,

    #[display(fmt = "async")]
    Async,

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

    /// Checks if this token is a possible symbol for identfiers
    pub fn is_identifier(&self) -> bool {
        self.as_identifier().is_some()
    }

    /// Checks if this token is a possible symbol for identfiers or reserved keyword
    pub fn is_identifier_or_reserved_kw(&self) -> bool {
        self.as_identifier().is_some() || self.as_reserved_keyword().is_some()
    }

    pub fn as_identifier(&self) -> Option<Symbol> {
        match self {
            Self::Identifier(sym) => Some(*sym),
            Self::Dollar => Some(sym::DOLLAR),
            _ => None,
        }
    }

    /// This is the reverse operation of `as_token(Symol)`
    pub fn as_reserved_keyword(&self) -> Option<Symbol> {
        match self {
            Self::If => Some(sym::IF),
            Self::Else => Some(sym::ELSE),
            Self::Function => Some(sym::FUNCTION),
            Self::Var => Some(sym::VAR),
            Self::Let => Some(sym::LET),
            Self::Const => Some(sym::CONST),
            Self::Return => Some(sym::RETURN),
            Self::Throw => Some(sym::THROW),
            Self::Try => Some(sym::TRY),
            Self::Catch => Some(sym::CATCH),
            Self::Finally => Some(sym::FINALLY),
            Self::TrueLit => Some(sym::TRUE_LIT),
            Self::FalseLit => Some(sym::FALSE_LIT),
            Self::NullLit => Some(sym::NULL_LIT),
            Self::UndefinedLit => Some(sym::UNDEFINED_LIT),
            Self::Yield => Some(sym::YIELD),
            Self::New => Some(sym::NEW),
            Self::For => Some(sym::FOR),
            Self::Do => Some(sym::DO),
            Self::While => Some(sym::WHILE),
            Self::In => Some(sym::IN),
            Self::Instanceof => Some(sym::INSTANCEOF),
            Self::Async => Some(sym::ASYNC),
            Self::Await => Some(sym::AWAIT),
            Self::Delete => Some(sym::DELETE),
            Self::Void => Some(sym::VOID),
            Self::Typeof => Some(sym::TYPEOF),
            Self::Continue => Some(sym::CONTINUE),
            Self::Break => Some(sym::BREAK),
            Self::Import => Some(sym::IMPORT),
            Self::Export => Some(sym::EXPORT),
            Self::Default => Some(sym::DEFAULT),
            Self::Debugger => Some(sym::DEBUGGER),
            Self::Of => Some(sym::OF),
            Self::Class => Some(sym::CLASS),
            Self::Extends => Some(sym::EXTENDS),
            Self::Static => Some(sym::STATIC),
            Self::Switch => Some(sym::SWITCH),
            Self::Case => Some(sym::CASE),
            Self::Get => Some(sym::GET),
            Self::Set => Some(sym::SET),
            _ => None,
        }
    }

    pub fn as_identifier_or_reserved_kw(&self) -> Option<Symbol> {
        self.as_identifier().or_else(|| self.as_reserved_keyword())
    }

    pub fn as_property_name(&self) -> Option<Symbol> {
        self.as_identifier_or_reserved_kw().or(match self {
            Self::String(s) => Some(*s),
            Self::NumberDec(s) => Some(*s),
            _ => None,
        })
    }

    /// Returns a "dummy" identifier.
    /// Should only be used in `ErrorKind`s.
    pub const DUMMY_IDENTIFIER: Self = Self::Identifier(sym::EMPTY);

    /// Returns a "dummy" template literal.
    /// Should only be used in `ErrorKind`s.
    pub const DUMMY_TEMPLATE_LITERAL: Self = Self::TemplateLiteral(sym::EMPTY);

    pub fn fmt_for_expected_tys(&self) -> impl fmt::Display + '_ {
        struct DisplayExpectedTys<'a>(&'a TokenType);
        impl<'a> fmt::Display for DisplayExpectedTys<'a> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match *self.0 {
                    TokenType::DUMMY_IDENTIFIER => write!(f, "<identifier>"),
                    TokenType::DUMMY_TEMPLATE_LITERAL => write!(f, "<template literal>"),
                    other => write!(f, "{}", other),
                }
            }
        }
        DisplayExpectedTys(self)
    }
}

pub fn as_token(s: Symbol) -> TokenType {
    match s {
        sym::IF => TokenType::If,
        sym::ELSE => TokenType::Else,
        sym::FUNCTION => TokenType::Function,
        sym::VAR => TokenType::Var,
        sym::LET => TokenType::Let,
        sym::CONST => TokenType::Const,
        sym::RETURN => TokenType::Return,
        sym::THROW => TokenType::Throw,
        sym::TRY => TokenType::Try,
        sym::CATCH => TokenType::Catch,
        sym::FINALLY => TokenType::Finally,
        sym::TRUE_LIT => TokenType::TrueLit,
        sym::FALSE_LIT => TokenType::FalseLit,
        sym::NULL_LIT => TokenType::NullLit,
        sym::UNDEFINED_LIT => TokenType::UndefinedLit,
        sym::YIELD => TokenType::Yield,
        sym::NEW => TokenType::New,
        sym::FOR => TokenType::For,
        sym::DO => TokenType::Do,
        sym::WHILE => TokenType::While,
        sym::IN => TokenType::In,
        sym::INSTANCEOF => TokenType::Instanceof,
        sym::ASYNC => TokenType::Async,
        sym::AWAIT => TokenType::Await,
        sym::DELETE => TokenType::Delete,
        sym::VOID => TokenType::Void,
        sym::TYPEOF => TokenType::Typeof,
        sym::CONTINUE => TokenType::Continue,
        sym::BREAK => TokenType::Break,
        sym::IMPORT => TokenType::Import,
        sym::EXPORT => TokenType::Export,
        sym::DEFAULT => TokenType::Default,
        sym::DEBUGGER => TokenType::Debugger,
        sym::OF => TokenType::Of,
        sym::CLASS => TokenType::Class,
        sym::EXTENDS => TokenType::Extends,
        sym::STATIC => TokenType::Static,
        sym::SWITCH => TokenType::Switch,
        sym::CASE => TokenType::Case,
        sym::GET => TokenType::Get,
        sym::SET => TokenType::Set,
        _ => TokenType::Identifier(s),
    }
}

/// A token
#[derive(Debug, Clone)]
pub struct Token {
    /// The type of token
    pub ty: TokenType,
    /// Location of this token in the input string
    pub span: Span,
}
