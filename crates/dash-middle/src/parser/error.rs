use std::num::ParseIntError;

use either::Either;

use crate::lexer::token::Token;
use crate::lexer::token::TokenType;

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
    /// Integer parsing failed
    ParseIntError(Token<'a>, ParseIntError),
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
            Self::UnknownToken(tok) => tok.loc.to_string(source, Either::Left(tok.full), "unknown token", true),
            Self::UnexpectedToken(tok, _) | Self::UnexpectedTokenMultiple(tok, _) => {
                tok.loc
                    .to_string(source, Either::Left(tok.full), "unexpected token", true)
            }
            Self::ParseIntError(tok, err) => tok.loc.to_string(
                source,
                Either::Left(tok.full),
                &format!("int parsing failed: {}", err),
                false,
            ),
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
