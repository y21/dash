use super::constant::LimitExceededError as ConstantLimitExceededError;
use super::scope::LimitExceededError as LocalLimitExceededError;

#[derive(Debug)]
pub enum CompileError {
    ConstantPoolLimitExceeded,
    LocalLimitExceeded,
    IfBranchLimitExceeded,
    Unimplemented(String),
    ParameterLimitExceeded,
}

impl From<ConstantLimitExceededError> for CompileError {
    fn from(_: ConstantLimitExceededError) -> Self {
        CompileError::ConstantPoolLimitExceeded
    }
}

impl From<LocalLimitExceededError> for CompileError {
    fn from(_: LocalLimitExceededError) -> Self {
        CompileError::LocalLimitExceeded
    }
}
