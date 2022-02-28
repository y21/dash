use crate::vm::local::LocalScope;
use crate::vm::value::Value;

pub type NativeFunction = fn(cx: CallContext) -> Result<Value, Value>;

#[derive(Debug)]
pub struct CallContext<'s> {
    pub args: Vec<Value>,
    pub scope: LocalScope<'s>,
}
