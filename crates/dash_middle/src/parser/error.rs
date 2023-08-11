use std::fmt;
use std::num::ParseIntError;

use crate::lexer::token::Token;
use crate::lexer::token::TokenType;

/// The type of parser error
#[derive(Debug)]
pub enum ErrorKind {
    /// An unknown token was found
    UnknownToken(Token),
    /// An token was found that we didn't expect, we expect a certain other token type
    UnexpectedToken(Token, TokenType),
    /// Same as UnexpectedToken, but we expected any of the given token types
    UnexpectedTokenMultiple(Token, &'static [TokenType]),
    /// Unexpected end of file
    UnexpectedEof,
    /// Integer parsing failed
    ParseIntError(Token, ParseIntError),
    /// More than one default clause in a switch statement
    MultipleDefaultInSwitch(Token),
    InvalidAccessorParams {
        got: usize,
        expect: usize,
        token: Token,
    },
    MultipleRestInDestructuring(Token),
    RegexSyntaxError(Token, dash_regex::Error),
    IncompleteSpread(Token),
}

/// An error that occurred during parsing
#[derive(Debug)]
pub struct Error {
    /// The type of error
    pub kind: ErrorKind,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
