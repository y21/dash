use std::fmt;

use crate::sourcemap::Span;

/// An error that may occur during lexing
#[derive(Debug)]
pub struct Error {
    /// The kind of error
    pub kind: ErrorKind,
    /// Where this error is located in the source string
    pub loc: Span,
    // TODO: store the source string in a `struct LexerErrors<'buf>(Vec<Error>, &'buf [u8])`
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
        // todo!()
        // match &self.kind {
        //     ErrorKind::UnknownCharacter(c) => {
        //         let format_err = FormattableError {
        //             source: self.source,
        //             loc: &self.loc,
        //             display_token: true,
        //             message: "unknown character",
        //             tok: Either::Right(*c as char),
        //             help: None,
        //         };
        //         format_err.fmt(f)
        //     }
        //     ErrorKind::UnexpectedEof => f.write_str("unexpected end of input"),
        // }
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
