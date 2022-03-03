use crate::vm::value::function::native::CallContext;
use crate::vm::value::Value;

pub fn to_string(cx: CallContext) -> Result<Value, Value> {
    Ok(cx.this)
}
