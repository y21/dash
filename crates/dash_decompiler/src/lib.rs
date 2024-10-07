use dash_middle::compiler::constant::ConstantPool;
use dash_middle::compiler::instruction::Instruction;
use dash_middle::interner::StringInterner;
use decompiler::FunctionDecompiler;
use thiserror::Error;

mod decompiler;

#[derive(Debug, Error)]
pub enum DecompileError {
    #[error("Abrupt end of file")]
    AbruptEof,
    #[error("Invalid instruction opcode: {_0}")]
    InvalidOp(u8),
    #[error("Referenced unused or unimplemented instruction: {_0:?}")]
    Unimplemented(Instruction),
    #[error("Invalid object member kind variant")]
    InvalidObjectMemberKind,
    #[error("Invalid intrinsic operation: {_0}")]
    InvalidIntrinsicOp(u8),
}

pub fn decompile(
    interner: &StringInterner,
    constants: &ConstantPool,
    instructions: &[u8],
) -> Result<String, DecompileError> {
    FunctionDecompiler::new(interner, instructions, constants, "<main>").run()
}
