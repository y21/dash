use thiserror::Error;

use crate::function::CompileError;
use crate::passes_legacy::infer::InferError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{}", _0)]
    Infer(#[from] InferError),
    #[error("{}", _0)]
    Compile(#[from] CompileError),
}
