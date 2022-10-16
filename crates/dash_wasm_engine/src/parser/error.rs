use std::io;
use std::string::FromUtf8Error;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unsupported wasm version: {0}")]
    UnsupportedWasmVersion(u32),
    #[error("invalid magic number (expected: 1836278016, got {0})")]
    IncorrectMagicNumber(u32),
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("LEB128 decoding error: {0}")]
    Leb128Read(#[from] leb128::read::Error),
    #[error("invalid type kind: {0}")]
    InvalidTypeKind(u8),
    #[error("invalid external kind: {0}")]
    InvalidExternalKind(u8),
    #[error("UTF8 decoding error: {0}")]
    Utf8(#[from] FromUtf8Error),
    #[error("invalid function end marker: {0}")]
    InvalidFunctionEndMarker(u8),
    #[error("invalid memory flags: {0}")]
    InvalidMemoryFlags(u8),
}
