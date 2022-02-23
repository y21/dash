use crate::vm::value::Value;

pub type NativeFunction = fn(cx: CallContext) -> Result<Value, Value>;

pub struct CallContext {
    pub args: Vec<Value>,
}
