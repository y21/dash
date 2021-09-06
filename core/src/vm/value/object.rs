use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt::Debug;

use super::exotic::Exotic;
use super::promise::Promise;
use super::weak::WeakMap;
use super::weak::WeakSet;
use super::{array::Array, function::FunctionKind, Value};

/// A type of weak collection
// TODO: move to weak.rs?
#[derive(Debug, Clone)]
pub enum Weak {
    /// Represents a JavaScript WeakSet
    Set(WeakSet<RefCell<Value>>),
    /// Represents a JavaScript WeakMap
    Map(WeakMap<RefCell<Value>, RefCell<Value>>),
}

impl Weak {
    /// Returns a reference to the underlying WeakSet, if it is one
    pub fn as_set(&self) -> Option<&WeakSet<RefCell<Value>>> {
        match self {
            Self::Set(s) => Some(s),
            _ => None,
        }
    }

    /// Returns a mutable reference to the underlying WeakSet, if it is one
    pub fn as_set_mut(&mut self) -> Option<&mut WeakSet<RefCell<Value>>> {
        match self {
            Self::Set(s) => Some(s),
            _ => None,
        }
    }

    /// Returns a reference to the underlying WeakMap, if it is one
    pub fn as_map(&self) -> Option<&WeakMap<RefCell<Value>, RefCell<Value>>> {
        match self {
            Self::Map(m) => Some(m),
            _ => None,
        }
    }

    /// Returns a mutable reference to the underlying WeakMap, if it is one
    pub fn as_map_mut(&mut self) -> Option<&mut WeakMap<RefCell<Value>, RefCell<Value>>> {
        match self {
            Self::Map(m) => Some(m),
            _ => None,
        }
    }

    /// Converts this weak collection to a string
    pub fn to_string(&self) -> Cow<str> {
        match self {
            Self::Set(_) => Cow::Borrowed("[object WeakSet]"),
            Self::Map(_) => Cow::Borrowed("[object WeakMap]"),
        }
    }

    /// Inspects this weak collection
    pub fn inspect(&self) -> Cow<str> {
        match self {
            Self::Set(s) => Cow::Owned(format!("WeakSet {{ <{} items> }}", s.0.len())),
            Self::Map(m) => Cow::Owned(format!("WeakMap {{ <{} items> }}", m.0.len())),
        }
    }
}

/// A JavaScript exotic object
///
/// Any kind of object that is "magic" in some way is exotic.
/// For example, functions are callable objects.
#[derive(Debug, Clone)]
pub enum ExoticObject {
    /// A JavaScript String
    String(String),
    /// A JavaScript function
    Function(FunctionKind),
    /// A JavaScript array
    Array(Array),
    /// A JavaScript weak type
    Weak(Weak),
    /// A JavaScript promise
    Promise(Promise),
    /// Custom exotic types
    Custom(Box<dyn Exotic>),
}

/// A JavaScript object
#[derive(Debug, Clone)]
pub enum Object {
    /// Exotic object
    Exotic(ExoticObject),
    /// Ordinary, regular object
    Ordinary,
}
