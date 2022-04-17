use crate::vm::local::LocalScope;
use crate::vm::value::Value;

pub type NativeFunction = fn(cx: CallContext) -> Result<Value, Value>;

#[derive(Debug)]
pub struct CallContext<'s, 'c> {
    pub args: Vec<Value>,
    pub scope: &'c mut LocalScope<'s>,
    pub this: Value,
}
