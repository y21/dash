use std::fmt;
use std::num::ParseIntError;

use either::Either;

use crate::lexer::token::FormattableError;
use crate::lexer::token::Location;
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
    /// More than one default clause in a switch statement
    MultipleDefaultInSwitch(Token<'a>),
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

impl fmt::Display for Error<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (tok, message, help): (_, _, Option<Box<dyn fmt::Display + '_>>) = match &self.kind {
            ErrorKind::UnknownToken(tok) => (tok, "unknown token", None),
            ErrorKind::UnexpectedToken(tok, expect) => {
                (tok, "unexpected token", Some(Box::new(format!("expected `{expect}`"))))
            }
            ErrorKind::UnexpectedTokenMultiple(tok, expect) => (
                tok,
                "unexpected token",
                Some(Box::new(format!(
                    "expected one of: {}",
                    expect.iter().map(|t| format!("`{}`", t)).collect::<Vec<_>>().join(", ")
                ))),
            ),
            ErrorKind::ParseIntError(tok, err) => (tok, "int parsing failed", Some(Box::new(err))),
            ErrorKind::UnexpectedEof => (
                &Token {
                    full: "",
                    ty: TokenType::Eof,
                    loc: Location {
                        line: 0,
                        line_offset: 0,
                        offset: 0,
                    },
                },
                "unexpected end of input",
                Some(Box::new("more tokens are expected for this to be valid")),
            ),
            ErrorKind::MultipleDefaultInSwitch(tok) => (
                tok,
                "more than one default in a switch statement",
                Some(Box::new("consider merging all default clauses into one")),
            ),
        };

        let format_err = FormattableError {
            source: self.source,
            loc: &tok.loc,
            display_token: true,
            message,
            tok: Either::Left(tok.full),
            help,
        };

        format_err.fmt(f)
    }
}
