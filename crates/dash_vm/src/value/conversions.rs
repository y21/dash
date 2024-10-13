use crate::gc::handle::Handle;
use crate::gc::ObjectId;

use super::primitive::Symbol;
use super::Value;

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
