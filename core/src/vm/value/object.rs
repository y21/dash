use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

use crate::js_std;

use super::function::Constructor;
use super::weak::WeakMap;
use super::weak::WeakSet;
use super::{
    array::Array,
    function::{FunctionKind, NativeFunction, NativeFunctionCallback, Receiver},
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

    pub fn get_property_unboxed(&self, k: &str) -> Option<PropertyLookup> {
        match self {
            Self::Set(_) => match k {
                "has" => Some(PropertyLookup::Function(
                    js_std::weakset::has,
                    "has",
                    Constructor::NoCtor,
                )),
                "add" => Some(PropertyLookup::Function(
                    js_std::weakset::add,
                    "add",
                    Constructor::NoCtor,
                )),
                "delete" => Some(PropertyLookup::Function(
                    js_std::weakset::delete,
                    "delete",
                    Constructor::NoCtor,
                )),
                _ => None,
            },
            Self::Map(_) => match k {
                "has" => Some(PropertyLookup::Function(
                    js_std::weakmap::has,
                    "has",
                    Constructor::NoCtor,
                )),
                "add" => Some(PropertyLookup::Function(
                    js_std::weakmap::add,
                    "add",
                    Constructor::NoCtor,
                )),
                "delete" => Some(PropertyLookup::Function(
                    js_std::weakmap::delete,
                    "delete",
                    Constructor::NoCtor,
                )),
                "get" => Some(PropertyLookup::Function(
                    js_std::weakmap::get,
                    "get",
                    Constructor::NoCtor,
                )),
                _ => None,
            },
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

impl Object {
    pub fn get_property_unboxed(&self, k: &str) -> Option<PropertyLookup> {
        match self {
            Self::String(s) => super::string::get_property_unboxed(s, k),
            Self::Array(a) => a.get_property_unboxed(k),
            Self::Weak(w) => w.get_property_unboxed(k),
            _ => None,
        }
    }

    pub fn get_property(&self, cell: &Rc<RefCell<Value>>, k: &str) -> Option<Rc<RefCell<Value>>> {
        let result = self.get_property_unboxed(k)?;

        Some(match result {
            PropertyLookup::ValueRef(r) => r,
            PropertyLookup::Function(func, name, ctor) => Value::from(NativeFunction::new(
                name,
                func,
                Some(Receiver::Bound(cell.clone())),
                ctor,
            ))
            .into(),
            PropertyLookup::Value(v) => Value::new(v).into(),
        })
    }
}
