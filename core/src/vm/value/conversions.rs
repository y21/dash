use std::borrow::Cow;

use super::{
    array::Array,
    function::FunctionKind,
    object::{Object, Weak},
    promise::Promise,
    Value, ValueKind,
};
use crate::vm::{instruction::Constant, value::promise::PromiseState};

impl Value {
    /// Attempts to convert self to a constant
    pub fn as_constant(&self) -> Option<&Constant> {
        match &self.kind {
            ValueKind::Constant(c) => Some(c),
            _ => None,
        }
    }

    /// Attempts to convert self into a constant by consuming self
    pub fn into_constant(self) -> Option<Constant> {
        match self.kind {
            ValueKind::Constant(c) => Some(*c),
            _ => None,
        }
    }

    /// Converts a JavaScript value to a number
    ///
    /// If the value is not a number, [f64::NAN] is returned
    pub fn as_number(&self) -> f64 {
        match &self.kind {
            ValueKind::Number(n) => *n,
            ValueKind::Bool(f) => *f as u8 as f64,
            ValueKind::Object(o) => o.as_number(),
            ValueKind::Null => 0f64,
            _ => f64::NAN,
        }
    }

    /// Converts a JavaScript value to a whole number (i64)
    pub fn as_whole_number(&self) -> i64 {
        self.as_number().floor() as i64
    }

    /// Converts a JavaScript value to a whole number (i32)
    pub fn as_32bit_number(&self) -> i32 {
        self.as_number().floor() as i32
    }

    /// Attempts to return self as a boolean
    ///
    /// This does not *convert* a value to a boolean. To get the effect of `!!value`,
    /// use [Value::is_truthy]
    pub fn as_bool(&self) -> Option<bool> {
        match &self.kind {
            ValueKind::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Attempts to return a reference to the inner object if it is one
    pub fn as_object(&self) -> Option<&Object> {
        match &self.kind {
            ValueKind::Object(o) => Some(o),
            _ => None,
        }
    }

    /// Attempts to return a mutable reference to the inner object if it is one
    pub fn as_object_mut(&mut self) -> Option<&mut Object> {
        match &mut self.kind {
            ValueKind::Object(o) => Some(o),
            _ => None,
        }
    }

    /// Attempts to return a reference to the inner function kind if it is one
    pub fn as_function(&self) -> Option<&FunctionKind> {
        match &self.kind {
            ValueKind::Object(o) => o.as_function(),
            _ => None,
        }
    }

    /// Attempts to return a mutable reference to the inner function kind if it is one
    pub fn as_function_mut(&mut self) -> Option<&mut FunctionKind> {
        match &mut self.kind {
            ValueKind::Object(o) => o.as_function_mut(),
            _ => None,
        }
    }

    /// Converts a JavaScript value to a string
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

    /// Converts a JavaScript value to a JSON string
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

    /// Inspects a JavaScript value, and returns a string that may be more useful
    /// than calling [Value::to_string]
    ///
    /// In particular, this actually returns all entries of an object instead of
    /// returning "[object Object]"
    pub fn inspect(&self, depth: u32) -> Cow<str> {
        match &self.kind {
            ValueKind::Object(o) => o.inspect(self, depth),
            _ => self.to_string(),
        }
    }

    /// Attempts to return self as a string
    ///
    /// This does not *convert* a value to a string. To get the effect of `"" + value`,
    /// use [Value::to_string]
    pub fn as_string(&self) -> Option<&str> {
        self.as_object().and_then(|o| o.as_string())
    }

    /// Attempts to return the identifier of this value if it is a constant
    pub fn into_ident(self) -> Option<String> {
        match self.kind {
            ValueKind::Constant(i) => i.into_ident(),
            _ => None,
        }
    }

    /// Attempts to return the inner object if it is one
    pub fn into_object(self) -> Option<Object> {
        match self.kind {
            ValueKind::Object(o) => Some(*o),
            _ => None,
        }
    }

    /// Attempts to return the inner string if it is one
    pub fn into_string(self) -> Option<String> {
        self.into_object().and_then(|c| c.into_string())
    }
}

impl Object {
    /// Converts a JavaScript object to a number
    pub fn as_number(&self) -> f64 {
        f64::NAN // TODO: try to convert it to number?
    }

    /// Attempts to return self as a string
    ///
    /// This does not *convert* a value to a string. To get the effect of `"" + value`,
    /// use [Value::to_string]
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Attempts to return self as a string if it is one
    pub fn into_string(self) -> Option<String> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Converts a JavaScript value to a string
    pub fn to_string(&self) -> Cow<str> {
        match self {
            Self::String(s) => Cow::Borrowed(s),
            Self::Function(f) => Cow::Owned(f.to_string()),
            Self::Array(_) => Cow::Borrowed("[object Array]"),
            Self::Weak(w) => w.to_string(),
            _ => Cow::Borrowed("[object Object]"), // TODO: look if there's a toString function
        }
    }

    /// Converts a JavaScript value to a JSON string
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

                    let element = unsafe { element_cell.borrow_unbounded() };

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
                    let value = unsafe { value_cell.borrow_unbounded() };
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

    /// Inspects a JavaScript value
    ///
    /// See [Value::inspect] for the difference between to_string and inspect
    pub fn inspect(&self, this: &Value, depth: u32) -> Cow<str> {
        if depth > 5 {
            return Cow::Borrowed("…");
        }

        match self {
            Self::String(s) => {
                if depth > 0 {
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
                    let element = unsafe { element_cell.borrow_unbounded() };
                    if index > 0 {
                        s.push_str(", ");
                    }
                    s.push_str(&*element.inspect(depth + 1));
                }
                s.push_str(" ]");
                Cow::Owned(s)
            }
            Self::Weak(w) => w.inspect(),
            Self::Promise(p) => match &p.value {
                PromiseState::Resolved(value_cell) => {
                    let value = unsafe { value_cell.borrow_unbounded() };
                    Cow::Owned(format!(
                        "Promise {{ <resolved> {} }}",
                        value.inspect(depth + 1)
                    ))
                }
                PromiseState::Rejected(value_cell) => {
                    let value = unsafe { value_cell.borrow_unbounded() };
                    Cow::Owned(format!(
                        "Promise {{ <rejected> {} }}",
                        value.inspect(depth + 1)
                    ))
                }
                PromiseState::Pending => Cow::Borrowed("Promise {<pending>}"),
            },
            Self::Any(_) => {
                let mut s = String::from("{ ");

                for (index, (key, value_cell)) in this.fields.iter().enumerate() {
                    let value = unsafe { value_cell.borrow_unbounded() };
                    if index > 0 {
                        s.push_str(", ");
                    }
                    s.push_str(&format!(r#""{}": {}"#, key, value.inspect(depth + 1)));
                }

                s.push_str(" }");
                Cow::Owned(s)
            }
        }
    }

    /// Attempts to return self as a function if it is one
    pub fn into_function(self) -> Option<FunctionKind> {
        match self {
            Self::Function(kind) => Some(kind),
            _ => None,
        }
    }

    /// Attempts to return self as a reference to the function if it is one
    pub fn as_function(&self) -> Option<&FunctionKind> {
        match self {
            Self::Function(kind) => Some(kind),
            _ => None,
        }
    }

    /// Attempts to return self as a mutable reference to the function if it is one
    pub fn as_function_mut(&mut self) -> Option<&mut FunctionKind> {
        match self {
            Self::Function(kind) => Some(kind),
            _ => None,
        }
    }

    /// Attempts to return self as an array if it is one
    pub fn as_array(&self) -> Option<&Array> {
        match self {
            Self::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Attempts to return self as a reference to [Weak] if it is one
    pub fn as_weak(&self) -> Option<&Weak> {
        match self {
            Self::Weak(w) => Some(w),
            _ => None,
        }
    }

    /// Attempts to return self as a mutable reference to [Weak] if it is one
    pub fn as_weak_mut(&mut self) -> Option<&mut Weak> {
        match self {
            Self::Weak(w) => Some(w),
            _ => None,
        }
    }

    /// Attempts to return self as a reference to [Promise] if it is one
    pub fn as_promise(&self) -> Option<&Promise> {
        match self {
            Self::Promise(p) => Some(p),
            _ => None,
        }
    }
}
