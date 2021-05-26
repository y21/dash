use std::{borrow::Cow, cell::RefCell, rc::Rc};

use super::{
    array::Array,
    function::{CallContext, FunctionKind},
    object::{Object, PropertyLookup},
    weak::WeakSet,
    Value, ValueKind,
};
use crate::vm::{instruction::Constant, VM};

impl Value {
    pub fn as_constant(&self) -> Option<&Constant> {
        match &self.kind {
            ValueKind::Constant(c) => Some(c),
            _ => None,
        }
    }

    pub fn into_constant(self) -> Option<Constant> {
        match self.kind {
            ValueKind::Constant(c) => Some(*c),
            _ => None,
        }
    }

    pub fn as_number(&self) -> f64 {
        match &self.kind {
            ValueKind::Number(n) => *n,
            ValueKind::Bool(f) => *f as u8 as f64,
            ValueKind::Object(o) => o.as_number(),
            ValueKind::Null => 0f64,
            _ => f64::NAN,
        }
    }

    pub fn as_whole_number(&self) -> i64 {
        self.as_number().floor() as i64
    }

    pub fn as_bool(&self) -> Option<bool> {
        match &self.kind {
            ValueKind::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&Object> {
        match &self.kind {
            ValueKind::Object(o) => Some(o),
            _ => None,
        }
    }

    pub fn as_object_mut(&mut self) -> Option<&mut Object> {
        match &mut self.kind {
            ValueKind::Object(o) => Some(o),
            _ => None,
        }
    }

    pub fn as_function(&self) -> Option<&FunctionKind> {
        match &self.kind {
            ValueKind::Object(o) => o.as_function(),
            _ => None,
        }
    }

    pub fn to_string(&self) -> Cow<str> {
        match &self.kind {
            ValueKind::Bool(b) => Cow::Owned(b.to_string()),
            ValueKind::Constant(_) => unreachable!(),
            ValueKind::Null => Cow::Borrowed("null"),
            ValueKind::Number(n) => Cow::Owned(n.to_string()),
            ValueKind::Object(o) => o.to_string(),
            ValueKind::Undefined => Cow::Borrowed("undefined"),
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        self.as_object().and_then(|o| o.as_string())
    }

    pub fn into_ident(self) -> Option<String> {
        match self.kind {
            ValueKind::Constant(i) => i.into_ident(),
            _ => None,
        }
    }

    pub fn into_object(self) -> Option<Object> {
        match self.kind {
            ValueKind::Object(o) => Some(*o),
            _ => None,
        }
    }

    pub fn into_string(self) -> Option<String> {
        self.into_object().and_then(|c| c.into_string())
    }
}

impl Object {
    pub fn as_number(&self) -> f64 {
        f64::NAN // TODO: try to convert it to number?
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn into_string(self) -> Option<String> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn to_string(&self) -> Cow<str> {
        match self {
            Self::String(s) => Cow::Borrowed(s),
            Self::Function(f) => Cow::Owned(f.to_string()),
            Self::Array(a) => {
                let mut s = String::from("[");
                for (index, element_cell) in a.elements.iter().enumerate() {
                    let element = element_cell.borrow();
                    if index > 0 {
                        s.push(',');
                    }
                    s.push_str(&*element.to_string());
                }
                s.push(']');
                Cow::Owned(s)
            }
            Self::WeakSet(s) => Cow::Owned(format!("WeakSet {{ <{} items> }}", s.0.len())),
            _ => Cow::Borrowed("[object Object]"), // TODO: look if there's a toString function
        }
    }

    pub fn as_function(&self) -> Option<&FunctionKind> {
        match self {
            Self::Function(kind) => Some(kind),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Array> {
        match self {
            Self::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_weakset(&self) -> Option<&WeakSet<RefCell<Value>>> {
        match self {
            Self::WeakSet(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_weakset_mut(&mut self) -> Option<&mut WeakSet<RefCell<Value>>> {
        match self {
            Self::WeakSet(s) => Some(s),
            _ => None,
        }
    }
}
