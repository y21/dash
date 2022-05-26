use std::borrow::Cow;

use either::Either;

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

impl<'a> Error<'a> {
    /// Formats this error
    pub fn to_string(&self) -> Cow<str> {
        match &self.kind {
            ErrorKind::UnknownCharacter(c) => {
                Cow::Owned(
                    self.loc
                        .to_string(self.source, Either::Right(*c as char), "unknown character", true),
                )
            }
            ErrorKind::UnexpectedEof => Cow::Borrowed("Unexpected end of input"),
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
