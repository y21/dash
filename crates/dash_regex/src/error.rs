use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unexpected end of file")]
    UnexpectedEof,

    #[error("unexpected character: {_0}")]
    UnexpectedChar(u8),
}
