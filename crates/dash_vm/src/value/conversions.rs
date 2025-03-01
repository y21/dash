use crate::gc::ObjectId;

use super::Value;
use super::primitive::Symbol;

impl From<ObjectId> for Value {
    fn from(object: ObjectId) -> Self {
        Value::object(object)
    }
}

impl From<Symbol> for Value {
    fn from(symbol: Symbol) -> Self {
        Value::symbol(symbol)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::boolean(b)
    }
}
