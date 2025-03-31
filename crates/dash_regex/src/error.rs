use thiserror::Error;

use crate::flags;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unexpected end of file")]
    UnexpectedEof,

    #[error("unexpected character: {}", *.0 as char)]
    UnexpectedChar(u8),

    #[error("number too large to fit in a u32")]
    Overflow,

    #[error("{0}")]
    Flags(#[from] flags::Error),
}
