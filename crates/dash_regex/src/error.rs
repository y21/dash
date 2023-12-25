use thiserror::Error;

use crate::flags;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unexpected end of file")]
    UnexpectedEof,

    #[error("unexpected character: {}", *.0 as char)]
    UnexpectedChar(u8),

    #[error("{0}")]
    Flags(#[from] flags::Error),
}
