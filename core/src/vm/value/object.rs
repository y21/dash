use std::cell::RefCell;
use std::rc::Rc;

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
    Any(AnyObject),
}

#[derive(Debug, Clone)]
pub struct AnyObject {}

pub enum PropertyLookup {
    Function(NativeFunctionCallback, &'static str, bool),
    Value(ValueKind),
    ValueRef(Rc<RefCell<Value>>),
}

impl Object {
    pub fn get_property_unboxed(&self, k: &str) -> Option<PropertyLookup> {
        match self {
            Self::String(s) => super::string::get_property_unboxed(s, k),
            Self::Array(a) => a.get_property_unboxed(k),
            _ => None,
        }
    }

    pub fn get_property(&self, cell: &Rc<RefCell<Value>>, k: &str) -> Option<Rc<RefCell<Value>>> {
        let result = self.get_property_unboxed(k)?;

        Some(match result {
            PropertyLookup::ValueRef(r) => r,
            PropertyLookup::Function(func, name, ctor) => Rc::new(RefCell::new(Value::new(
                ValueKind::Object(Box::new(Object::Function(FunctionKind::Native(
                    NativeFunction::new(name, func, Some(Receiver::Bound(cell.clone())), ctor),
                )))),
            ))),
            PropertyLookup::Value(v) => Rc::new(RefCell::new(Value::new(v))),
        })
    }
}
