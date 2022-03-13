use crate::vm::value::function::native::CallContext;
use crate::vm::value::ops::abstractions::conversions::ValueConversion;
use crate::vm::value::Value;
use crate::vm::value::ValueContext;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let value = cx.args.get(0).unwrap_or_undefined().to_string(cx.scope)?;
    Ok(Value::String(value))
}

pub fn to_string(cx: CallContext) -> Result<Value, Value> {
    Ok(cx.this)
}
