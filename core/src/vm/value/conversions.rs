use std::borrow::Cow;

use super::{
    array::Array,
    function::FunctionKind,
    object::{Object, Weak},
    Value, ValueKind,
};
use crate::vm::instruction::Constant;

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

    pub fn as_32bit_number(&self) -> i32 {
        self.as_number().floor() as i32
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

    pub fn as_function_mut(&mut self) -> Option<&mut FunctionKind> {
        match &mut self.kind {
            ValueKind::Object(o) => o.as_function_mut(),
            _ => None,
        }
    }

    pub fn to_string(&self) -> Cow<str> {
        match &self.kind {
            ValueKind::Bool(b) => Cow::Borrowed(if *b { "true " } else { "false" }),
            ValueKind::Constant(_) => unreachable!(),
            ValueKind::Null => Cow::Borrowed("null"),
            ValueKind::Number(n) => Cow::Owned(n.to_string()),
            ValueKind::Object(o) => o.to_string(),
            ValueKind::Undefined => Cow::Borrowed("undefined"),
        }
    }

    pub fn to_json(&self) -> Option<Cow<str>> {
        match &self.kind {
            ValueKind::Bool(_) => Some(self.to_string()),
            ValueKind::Null => Some(self.to_string()),
            ValueKind::Number(_) => Some(self.to_string()),
            ValueKind::Undefined => Some(self.to_string()),
            ValueKind::Object(o) => o.to_json(self),
            ValueKind::Constant(_) => unreachable!(),
        }
    }

    pub fn inspect(&self, in_object: bool) -> Cow<str> {
        match &self.kind {
            ValueKind::Object(o) => o.inspect(self, in_object),
            _ => self.to_string(),
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
            Self::Array(_) => Cow::Borrowed("[object Array]"),
            Self::Weak(w) => w.to_string(),
            _ => Cow::Borrowed("[object Object]"), // TODO: look if there's a toString function
        }
    }

    pub fn to_json(&self, this: &Value) -> Option<Cow<str>> {
        match self {
            Self::String(s) => Some(Cow::Owned(format!("\"{}\"", s))),
            Self::Function(_) => None,
            Self::Array(a) => {
                let mut s = String::from("[ ");

                for (index, element_cell) in a.elements.iter().enumerate() {
                    if index > 0 {
                        s.push_str(", ");
                    }

                    let element = element_cell.borrow();

                    if let Some(element) = element.to_json() {
                        s.push_str(&element);
                    }
                }

                s.push_str(" ]");
                Some(Cow::Owned(s))
            }
            Self::Any(_) => {
                let mut s = String::from("{ ");

                for (index, (key, value_cell)) in this.fields.iter().enumerate() {
                    let value = value_cell.borrow();
                    if index > 0 {
                        s.push_str(", ");
                    }

                    if let Some(value) = value.to_json() {
                        s.push_str(&format!(r#""{}": {}"#, key, &value));
                    }
                }

                s.push_str(" }");
                Some(Cow::Owned(s))
            }
            _ => None,
        }
    }

    pub fn inspect(&self, this: &Value, in_object: bool) -> Cow<str> {
        match self {
            Self::String(s) => {
                if in_object {
                    Cow::Owned(format!(
                        "\"{}\"",
                        s.replace("\n", "\\n").replace("\"", "\\\"")
                    ))
                } else {
                    Cow::Borrowed(s)
                }
            }
            Self::Function(f) => Cow::Owned(f.to_string()),
            Self::Array(a) => {
                let mut s = String::from("[ ");
                for (index, element_cell) in a.elements.iter().enumerate() {
                    let element = element_cell.borrow();
                    if index > 0 {
                        s.push_str(", ");
                    }
                    s.push_str(&*element.inspect(true));
                }
                s.push_str(" ]");
                Cow::Owned(s)
            }
            Self::Weak(w) => w.inspect(),
            Self::Any(_) => {
                let mut s = String::from("{ ");

                for (index, (key, value_cell)) in this.fields.iter().enumerate() {
                    let value = value_cell.borrow();
                    if index > 0 {
                        s.push_str(", ");
                    }
                    s.push_str(&format!(r#""{}": {}"#, key, value.inspect(true)));
                }

                s.push_str(" }");
                Cow::Owned(s)
            }
        }
    }

    pub fn into_function(self) -> Option<FunctionKind> {
        match self {
            Self::Function(kind) => Some(kind),
            _ => None,
        }
    }

    pub fn as_function(&self) -> Option<&FunctionKind> {
        match self {
            Self::Function(kind) => Some(kind),
            _ => None,
        }
    }

    pub fn as_function_mut(&mut self) -> Option<&mut FunctionKind> {
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

    pub fn as_weak(&self) -> Option<&Weak> {
        match self {
            Self::Weak(w) => Some(w),
            _ => None,
        }
    }

    pub fn as_weak_mut(&mut self) -> Option<&mut Weak> {
        match self {
            Self::Weak(w) => Some(w),
            _ => None,
        }
    }
}
