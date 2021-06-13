use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

use super::function::Constructor;
use super::weak::WeakMap;
use super::weak::WeakSet;
use super::{
    array::Array,
    function::{FunctionKind, NativeFunctionCallback},
    Value, ValueKind,
};

// TODO: move to weak.rs?
#[derive(Debug, Clone)]
pub enum Weak {
    Set(WeakSet<RefCell<Value>>),
    Map(WeakMap<RefCell<Value>, RefCell<Value>>),
}

impl Weak {
    pub fn as_set(&self) -> Option<&WeakSet<RefCell<Value>>> {
        match self {
            Self::Set(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_set_mut(&mut self) -> Option<&mut WeakSet<RefCell<Value>>> {
        match self {
            Self::Set(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&WeakMap<RefCell<Value>, RefCell<Value>>> {
        match self {
            Self::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_map_mut(&mut self) -> Option<&mut WeakMap<RefCell<Value>, RefCell<Value>>> {
        match self {
            Self::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn to_string(&self) -> Cow<str> {
        match self {
            Self::Set(_) => Cow::Borrowed("[object WeakSet]"),
            Self::Map(_) => Cow::Borrowed("[object WeakMap]"),
        }
    }

    pub fn inspect(&self) -> Cow<str> {
        match self {
            Self::Set(s) => Cow::Owned(format!("WeakSet {{ <{} items> }}", s.0.len())),
            Self::Map(m) => Cow::Owned(format!("WeakMap {{ <{} items> }}", m.0.len())),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Object {
    String(String),
    Function(FunctionKind),
    Array(Array),
    Weak(Weak),
    Any(AnyObject),
}

#[derive(Debug, Clone)]
pub struct AnyObject {}

pub enum PropertyLookup {
    Function(NativeFunctionCallback, &'static str, Constructor),
    Value(ValueKind),
    ValueRef(Rc<RefCell<Value>>),
}

impl PropertyLookup {
    pub fn into_function(self) -> Option<(NativeFunctionCallback, &'static str, Constructor)> {
        match self {
            Self::Function(func, name, ctor) => Some((func, name, ctor)),
            _ => None,
        }
    }
}
