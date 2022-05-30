use crate::gc::handle::Handle;

use super::{object::Object, primitive::Symbol, Value};

impl From<Handle<dyn Object>> for Value {
    fn from(object: Handle<dyn Object>) -> Self {
        Value::Object(object)
    }
}

impl From<Symbol> for Value {
    fn from(symbol: Symbol) -> Self {
        Value::Symbol(symbol)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Boolean(b)
    }
}
