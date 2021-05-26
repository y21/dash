use std::cell::RefCell;
use std::rc::Rc;

use crate::js_std;

use super::weak::WeakSet;
use super::{
    array::Array,
    function::{FunctionKind, NativeFunction, NativeFunctionCallback, Receiver},
    Value, ValueKind,
};

#[derive(Debug, Clone)]
pub enum Object {
    String(String),
    Function(FunctionKind),
    Array(Array),
    WeakSet(WeakSet<RefCell<Value>>),
    Any(AnyObject),
}

#[derive(Debug, Clone)]
pub struct AnyObject {}

pub enum PropertyLookup {
    Function(NativeFunctionCallback, &'static str, bool),
    Value(ValueKind),
    ValueRef(Rc<RefCell<Value>>),
}

impl PropertyLookup {
    pub fn into_function(self) -> Option<(NativeFunctionCallback, &'static str, bool)> {
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
            Self::WeakSet(s) => match k {
                "has" => Some(PropertyLookup::Function(js_std::weakset::has, "has", false)),
                "add" => Some(PropertyLookup::Function(js_std::weakset::add, "add", false)),
                "delete" => Some(PropertyLookup::Function(
                    js_std::weakset::delete,
                    "delete",
                    false,
                )),
                _ => None,
            },
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
