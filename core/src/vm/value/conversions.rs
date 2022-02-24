use crate::gc::handle::Handle;

use super::{object::Object, Value};

impl From<Handle<dyn Object>> for Value {
    fn from(object: Handle<dyn Object>) -> Self {
        Value::Object(object)
    }
}
