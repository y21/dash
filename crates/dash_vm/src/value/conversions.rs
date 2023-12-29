use crate::gc::handle::Handle;

use super::primitive::Symbol;
use super::Value;

impl From<Handle> for Value {
    fn from(object: Handle) -> Self {
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
