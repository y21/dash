use dash_middle::compiler::instruction::Instruction;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{_0}")]
    TypedCfg(#[from] dash_typed_cfg::error::Error),
}
