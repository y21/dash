use std::fmt;

use either::Either;

use super::token::FormattableError;
use super::token::Location;

/// An error that may occur during lexing
#[derive(Debug)]
pub struct Error<'a> {
    /// The kind of error
    pub kind: ErrorKind,
    /// Where this error is located in the source string
    pub loc: Location,
    /// The input string
    ///
    /// Errors carry the input string with them because this is necessary
    /// when formatting errors. In the future, we might be able to avoid storing
    /// it here.
    pub source: &'a [u8],
}

impl fmt::Display for Error<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::UnknownCharacter(c) => {
                let format_err = FormattableError {
                    source: self.source,
                    loc: &self.loc,
                    display_token: true,
                    message: "unknown character",
                    tok: Either::Right(*c as char),
                    help: None,
                };
                format_err.fmt(f)
            }
            ErrorKind::UnexpectedEof => f.write_str("unexpected end of input"),
        }
    }
}

/// The type of error
#[derive(Debug)]
pub enum ErrorKind {
    /// An unknown character/byte
    UnknownCharacter(u8),
    /// Unexpected end of file
    UnexpectedEof,
}
