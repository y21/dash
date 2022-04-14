use core::fmt;

use super::constant::LimitExceededError as ConstantLimitExceededError;
use super::scope::LimitExceededError as LocalLimitExceededError;

#[derive(Debug)]
pub enum CompileError {
    ConstantPoolLimitExceeded,
    LocalLimitExceeded,
    IfBranchLimitExceeded,
    ArrayLitLimitExceeded,
    ObjectLitLimitExceeded,
    ConstAssignment,
    Unimplemented(String),
    ParameterLimitExceeded,
    YieldOutsideGenerator,
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

impl fmt::Display for CompileError {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConstantPoolLimitExceeded => f.write_str("Maximum number of entries in constant pool exceedeed"),
            Self::LocalLimitExceeded => f.write_str("Maximum number of local variables exceedeed"),
            Self::IfBranchLimitExceeded => f.write_str("Maximum number of if branches exceedeed"),
            Self::ArrayLitLimitExceeded => f.write_str("Maximum number of array literal elements exceedeed"),
            Self::ObjectLitLimitExceeded => f.write_str("Maximum number of object literal properties exceedeed"),
            Self::ConstAssignment => f.write_str("Cannot assign to constant"),
            Self::Unimplemented(s) => write!(f, "Unimplemented: {}", s),
            Self::ParameterLimitExceeded => f.write_str("Maximum number of function parameters exceedeed"),
            Self::YieldOutsideGenerator => f.write_str("`yield` is only available in generator functions"),
        }
    }
}
