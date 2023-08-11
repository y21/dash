use strum_macros::FromRepr;

use crate::parser;

use self::constant::ConstantPool;
use self::external::External;

#[cfg(feature = "format")]
use serde::{Deserialize, Serialize};
pub mod constant;
pub mod external;
#[cfg(feature = "format")]
pub mod format;
pub mod instruction;
pub mod instruction_iter;
pub mod scope;

#[cfg_attr(feature = "format", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
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
#[derive(FromRepr, Clone, Copy)]
pub enum StaticImportKind {
    All,
    Default,
}

#[repr(u8)]
#[derive(FromRepr)]
pub enum ObjectMemberKind {
    Getter,
    Setter,
    Static,
    Dynamic,
    Spread,
}

use parser::expr::ObjectMemberKind as ParserObjectMemberKind;

impl From<&ParserObjectMemberKind> for ObjectMemberKind {
    fn from(v: &ParserObjectMemberKind) -> Self {
        match v {
            ParserObjectMemberKind::Dynamic(..) => Self::Dynamic,
            ParserObjectMemberKind::Getter(..) => Self::Getter,
            ParserObjectMemberKind::Setter(..) => Self::Setter,
            ParserObjectMemberKind::Static(..) => Self::Static,
            ParserObjectMemberKind::Spread => Self::Spread,
        }
    }
}

#[repr(u8)]
#[derive(FromRepr, Debug)]
pub enum ArrayMemberKind {
    Item,
    Spread,
}

use parser::expr::ArrayMemberKind as ParserArrayMemberKind;

impl From<&ParserArrayMemberKind> for ArrayMemberKind {
    fn from(v: &ParserArrayMemberKind) -> Self {
        match v {
            ParserArrayMemberKind::Item(..) => Self::Item,
            ParserArrayMemberKind::Spread(..) => Self::Spread,
        }
    }
}
