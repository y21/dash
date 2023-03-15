use dash_middle::compiler::instruction::Instruction;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unsupported instruction")]
    UnsupportedInstruction { instr: Instruction },
}
