use dash_middle::compiler::instruction::Instruction;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unsupported instruction")]
    UnsupportedInstruction { instr: Instruction },
    #[error("unknown type of local {index}")]
    UnknownLocalType { index: u16 },
    #[error("unknown type of constant {index}")]
    UnknownConstantType { index: u16 },
}
