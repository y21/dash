use strum_macros::FromRepr;

use self::constant::ConstantPool;
use self::external::External;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
pub mod constant;
pub mod external;
pub mod instruction;
pub mod instruction_iter;
#[cfg(feature = "serde")]
pub mod format;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct CompileResult {
    pub instructions: Vec<u8>,
    pub cp: ConstantPool,
    pub locals: usize,
    pub externals: Vec<External>,
}

/// Function call metadata
///
/// Highest bit = set if constructor call
/// 2nd highest bit = set if object call
/// remaining 6 bits = number of arguments
#[repr(transparent)]
pub struct FunctionCallMetadata(u8);

impl From<u8> for FunctionCallMetadata {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<FunctionCallMetadata> for u8 {
    fn from(value: FunctionCallMetadata) -> Self {
        value.0
    }
}

impl FunctionCallMetadata {
    pub fn new_checked(mut value: u8, constructor: bool, object: bool) -> Option<Self> {
        if value & 0b11000000 == 0 {
            if constructor {
                value |= 0b10000000;
            }

            if object {
                value |= 0b01000000;
            }

            Some(Self(value))
        } else {
            None
        }
    }

    pub fn value(&self) -> u8 {
        self.0 & !0b11000000
    }

    pub fn is_constructor_call(&self) -> bool {
        self.0 & (1 << 7) != 0
    }

    pub fn is_object_call(&self) -> bool {
        self.0 & (1 << 6) != 0
    }
}

#[repr(u8)]
#[derive(FromRepr)]
pub enum StaticImportKind {
    All,
    Default,
}
