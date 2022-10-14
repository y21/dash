use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unexpected end of file")]
    UnexpectedEof,

    #[error("unexpected character: {}", *.0 as char)]
    UnexpectedChar(u8),
}
