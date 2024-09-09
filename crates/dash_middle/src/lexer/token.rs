use std::fmt;

use crate::interner::{sym, Symbol};
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
    RegexLiteral { literal: Symbol, flags: Symbol },

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
            Self::Dollar => Some(sym::dollar),
            Self::Set => Some(sym::set),
            Self::Get => Some(sym::get),
            _ => None,
        }
    }

    /// This is the reverse operation of `as_token(Symol)`
    pub fn as_reserved_keyword(&self) -> Option<Symbol> {
        match self {
            Self::If => Some(sym::if_),
            Self::Else => Some(sym::else_),
            Self::Function => Some(sym::function),
            Self::Var => Some(sym::var),
            Self::Let => Some(sym::let_),
            Self::Const => Some(sym::const_),
            Self::Return => Some(sym::return_),
            Self::Throw => Some(sym::throw),
            Self::Try => Some(sym::try_),
            Self::Catch => Some(sym::catch),
            Self::Finally => Some(sym::finally),
            Self::TrueLit => Some(sym::true_),
            Self::FalseLit => Some(sym::false_),
            Self::NullLit => Some(sym::null),
            Self::UndefinedLit => Some(sym::undefined),
            Self::Yield => Some(sym::yield_),
            Self::New => Some(sym::new),
            Self::For => Some(sym::for_),
            Self::Do => Some(sym::do_),
            Self::While => Some(sym::while_),
            Self::In => Some(sym::in_),
            Self::Instanceof => Some(sym::instanceof),
            Self::Async => Some(sym::async_),
            Self::Await => Some(sym::await_),
            Self::Delete => Some(sym::delete),
            Self::Void => Some(sym::void),
            Self::Typeof => Some(sym::typeof_),
            Self::Continue => Some(sym::continue_),
            Self::Break => Some(sym::break_),
            Self::Import => Some(sym::import),
            Self::Export => Some(sym::export),
            Self::Default => Some(sym::default),
            Self::Debugger => Some(sym::debugger),
            Self::Of => Some(sym::of),
            Self::Class => Some(sym::class),
            Self::Extends => Some(sym::extends),
            Self::Static => Some(sym::static_),
            Self::Switch => Some(sym::switch),
            Self::Case => Some(sym::case),
            Self::Get => Some(sym::get),
            Self::Set => Some(sym::set),
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
    pub const DUMMY_IDENTIFIER: Self = Self::Identifier(sym::empty);

    /// Returns a "dummy" string.
    pub const DUMMY_STRING: Self = Self::String(sym::empty);

    /// Returns a "dummy" template literal.
    /// Should only be used in `ErrorKind`s.
    pub const DUMMY_TEMPLATE_LITERAL: Self = Self::TemplateLiteral(sym::empty);

    pub fn fmt_for_expected_tys(&self) -> impl fmt::Display + '_ {
        struct DisplayExpectedTys<'a>(&'a TokenType);
        impl<'a> fmt::Display for DisplayExpectedTys<'a> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match *self.0 {
                    TokenType::DUMMY_IDENTIFIER => write!(f, "<identifier>"),
                    TokenType::DUMMY_TEMPLATE_LITERAL => write!(f, "<template literal>"),
                    TokenType::DUMMY_STRING => write!(f, "<string>"),
                    other => write!(f, "{}", other),
                }
            }
        }
        DisplayExpectedTys(self)
    }
}

pub fn as_token(s: Symbol) -> TokenType {
    match s {
        sym::if_ => TokenType::If,
        sym::else_ => TokenType::Else,
        sym::function => TokenType::Function,
        sym::var => TokenType::Var,
        sym::let_ => TokenType::Let,
        sym::const_ => TokenType::Const,
        sym::return_ => TokenType::Return,
        sym::throw => TokenType::Throw,
        sym::try_ => TokenType::Try,
        sym::catch => TokenType::Catch,
        sym::finally => TokenType::Finally,
        sym::true_ => TokenType::TrueLit,
        sym::false_ => TokenType::FalseLit,
        sym::null => TokenType::NullLit,
        sym::undefined => TokenType::UndefinedLit,
        sym::yield_ => TokenType::Yield,
        sym::new => TokenType::New,
        sym::for_ => TokenType::For,
        sym::do_ => TokenType::Do,
        sym::while_ => TokenType::While,
        sym::in_ => TokenType::In,
        sym::instanceof => TokenType::Instanceof,
        sym::async_ => TokenType::Async,
        sym::await_ => TokenType::Await,
        sym::delete => TokenType::Delete,
        sym::void => TokenType::Void,
        sym::typeof_ => TokenType::Typeof,
        sym::continue_ => TokenType::Continue,
        sym::break_ => TokenType::Break,
        sym::import => TokenType::Import,
        sym::export => TokenType::Export,
        sym::default => TokenType::Default,
        sym::debugger => TokenType::Debugger,
        sym::of => TokenType::Of,
        sym::class => TokenType::Class,
        sym::extends => TokenType::Extends,
        sym::static_ => TokenType::Static,
        sym::switch => TokenType::Switch,
        sym::case => TokenType::Case,
        sym::get => TokenType::Get,
        sym::set => TokenType::Set,
        _ => TokenType::Identifier(s),
    }
}

/// A token
#[derive(Debug, Copy, Clone)]
pub struct Token {
    /// The type of token
    pub ty: TokenType,
    /// Location of this token in the input string
    pub span: Span,
}

/// Tokens that may precede an expression
pub const EXPR_PRECEDED_TOKENS: &[TokenType] = &[
    // Symbols
    TokenType::Dot,
    TokenType::LeftParen,
    TokenType::LeftBrace,
    TokenType::LeftSquareBrace,
    TokenType::Semicolon,
    TokenType::Comma,
    TokenType::Less,
    TokenType::Greater,
    TokenType::LessEqual,
    TokenType::GreaterEqual,
    TokenType::Equality,
    TokenType::Inequality,
    TokenType::StrictEquality,
    TokenType::StrictInequality,
    TokenType::Plus,
    TokenType::Minus,
    TokenType::Star,
    TokenType::Remainder,
    TokenType::Increment,
    TokenType::Decrement,
    TokenType::LeftShift,
    TokenType::RightShift,
    TokenType::UnsignedRightShift,
    TokenType::BitwiseAnd,
    TokenType::BitwiseOr,
    TokenType::BitwiseXor,
    TokenType::LogicalNot,
    TokenType::BitwiseNot,
    TokenType::LogicalAnd,
    TokenType::LogicalOr,
    TokenType::Conditional,
    TokenType::Colon,
    TokenType::Assignment,
    TokenType::AdditionAssignment,
    TokenType::SubtractionAssignment,
    TokenType::MultiplicationAssignment,
    TokenType::RemainderAssignment,
    TokenType::LeftShiftAssignment,
    TokenType::RightShiftAssignment,
    TokenType::UnsignedRightShiftAssignment,
    TokenType::BitwiseAndAssignment,
    TokenType::BitwiseOrAssignment,
    TokenType::BitwiseXorAssignment,
    TokenType::Slash,
    TokenType::DivisionAssignment,
    // Keywords
    TokenType::New,
    TokenType::Delete,
    TokenType::Void,
    TokenType::Typeof,
    TokenType::Instanceof,
    TokenType::In,
    TokenType::Return,
    TokenType::Case,
    TokenType::Throw,
    TokenType::Else,
    TokenType::Await,
    TokenType::Yield,
];
