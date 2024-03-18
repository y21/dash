use std::rc::Rc;

use strum_macros::FromRepr;

use crate::parser;
use crate::sourcemap::Span;

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
    pub debug_symbols: DebugSymbols,
    pub source: Rc<str>,
}

/// For error purposes, this contains source code snippets used to improve errors, e.g. `x is not a function`
// IMPL DETAILS: We intentionally use a rather "dense" representation to save memory, even if it slows down the error path.
#[cfg_attr(feature = "format", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct DebugSymbols(Vec<(u16, Span)>);

impl DebugSymbols {
    pub fn add(&mut self, ip: u16, symbol: Span) {
        #[cfg(debug_assertions)]
        {
            if let Some(&(last_ip, _)) = self.0.last() {
                // ensure requirement for binary search
                assert!(last_ip <= ip);
            }
        }

        self.0.push((ip, symbol));
    }

    pub fn get(&self, ip: u16) -> Span {
        self.0
            .binary_search_by_key(&ip, |(ip, _)| *ip)
            .ok()
            .map(|i| self.0[i].1)
            .unwrap()
    }

    pub fn iter(&self) -> impl Iterator<Item = &(u16, Span)> {
        self.0.iter()
    }
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
    pub fn new_checked(value: usize, constructor: bool, object: bool) -> Option<Self> {
        let Ok(mut value) = u8::try_from(value) else {
            return None;
        };

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
    Empty,
    Spread,
}

use parser::expr::ArrayMemberKind as ParserArrayMemberKind;

impl From<&ParserArrayMemberKind> for ArrayMemberKind {
    fn from(v: &ParserArrayMemberKind) -> Self {
        match v {
            ParserArrayMemberKind::Item(..) => Self::Item,
            ParserArrayMemberKind::Empty => Self::Empty,
            ParserArrayMemberKind::Spread(..) => Self::Spread,
        }
    }
}
