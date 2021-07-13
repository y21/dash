use crate::gc::Handle;

use super::value::Value;

/// A value from another frame
#[derive(Debug, Clone)]
pub struct Upvalue(pub Handle<Value>);

impl Upvalue {
    pub(crate) fn mark_visited(&self) {
        Value::mark(&self.0)
    }
}
