// use core::fmt;

// use dash_middle::compiler::constant::LimitExceededError;
// use dash_middle::compiler::scope::LimitExceededError as LocalLimitExceededError;

// #[derive(Debug)]
// pub enum CompileError {
//     ConstantPoolLimitExceeded,
//     LocalLimitExceeded,
//     IfBranchLimitExceeded,
//     SwitchCaseLimitExceeded,
//     ArrayLitLimitExceeded,
//     ObjectLitLimitExceeded,
//     ExportNameListLimitExceeded,
//     DestructureLimitExceeded,
//     ConstAssignment,
//     Unimplemented(String),
//     ParameterLimitExceeded,
//     YieldOutsideGenerator,
//     AwaitOutsideAsync,
//     UnknownBinding,
//     IllegalBreak,
//     MissingInitializerInDestructuring,
// }

// impl From<LimitExceededError> for CompileError {
//     fn from(_: LimitExceededError) -> Self {
//         CompileError::ConstantPoolLimitExceeded
//     }
// }

// impl From<LocalLimitExceededError> for CompileError {
//     fn from(_: LocalLimitExceededError) -> Self {
//         CompileError::LocalLimitExceeded
//     }
// }

// impl fmt::Display for CompileError {
//     #[rustfmt::skip]
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             Self::ConstantPoolLimitExceeded => f.write_str("Maximum number of entries in constant pool exceedeed"),
//             Self::LocalLimitExceeded => f.write_str("Maximum number of local variables exceedeed"),
//             Self::IfBranchLimitExceeded => f.write_str("Maximum number of if branches exceedeed"),
//             Self::SwitchCaseLimitExceeded => f.write_str("Maximum number of switch cases exceeded"),
//             Self::ArrayLitLimitExceeded => f.write_str("Maximum number of array literal elements exceedeed"),
//             Self::ObjectLitLimitExceeded => f.write_str("Maximum number of object literal properties exceedeed"),
//             Self::DestructureLimitExceeded => f.write_str("Maximum number of object or array destructuring properties exceedeed"),
//             Self::ConstAssignment => f.write_str("Cannot assign to constant"),
//             Self::Unimplemented(s) => write!(f, "Unimplemented: {s}"),
//             Self::ParameterLimitExceeded => f.write_str("Maximum number of function parameters exceedeed"),
//             Self::YieldOutsideGenerator => f.write_str("`yield` is only available in generator functions"),
//             Self::ExportNameListLimitExceeded => f.write_str("Maximum number of export names exceedeed"),
//             Self::UnknownBinding => f.write_str("Attempted to visit unknown binding"),
//             Self::AwaitOutsideAsync => f.write_str("`await` is only available in async functions"),
//             Self::IllegalBreak => f.write_str("`break` is only available in switch-case and loop"),
//             Self::MissingInitializerInDestructuring => f.write_str("Missing initializer in destructuring pattern"),
//         }
//     }
// }
