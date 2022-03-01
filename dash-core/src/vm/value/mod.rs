pub mod array;
pub mod conversions;
pub mod function;
pub mod object;
pub mod ops;

use std::rc::Rc;

use crate::{
    compiler::constant::Constant,
    gc::{handle::Handle, trace::Trace},
};

use self::object::Object;

use super::local::LocalScope;
#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    String(Rc<str>),
    Undefined,
    Null,
    Object(Handle<dyn Object>),
}

unsafe impl Trace for Value {
    fn trace(&self) {
        if let Value::Object(handle) = self {
            handle.trace();
        }
    }
}

impl Value {
    pub fn from_constant(constant: Constant) -> Self {
        match constant {
            Constant::Number(n) => Value::Number(n),
            Constant::Boolean(b) => Value::Boolean(b),
            Constant::String(s) => Value::String(s.into()),
            Constant::Undefined => Value::Undefined,
            Constant::Null => Value::Null,
            _ => unimplemented!(),
        }
    }

    pub fn get_property(&self, sc: &mut LocalScope, key: &str) -> Result<Value, Value> {
        match self {
            Self::Object(o) => o.get_property(sc, key),
            _ => unimplemented!(),
        }
    }

    pub fn apply(
        &self,
        sc: &mut LocalScope,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        match self {
            Value::Object(object) => object.apply(sc, this, args),
            _ => unimplemented!(),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            _ => unimplemented!(),
        }
    }
}

pub trait ValueContext {
    fn unwrap_or_undefined(self) -> Value;
}

impl ValueContext for Option<Value> {
    fn unwrap_or_undefined(self) -> Value {
        match self {
            Some(x) => x,
            None => Value::Undefined,
        }
    }
}

impl ValueContext for Option<&Value> {
    fn unwrap_or_undefined(self) -> Value {
        match self {
            Some(x) => x.clone(), // Values are cheap to clone
            None => Value::Undefined,
        }
    }
}
