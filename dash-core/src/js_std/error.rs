use crate::vm::value::function::native::CallContext;
use crate::vm::value::ops::abstractions::conversions::ValueConversion;
use crate::vm::value::Value;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    todo!()
}

pub fn to_string(mut cx: CallContext) -> Result<Value, Value> {
    cx.this
        .get_property(&mut cx.scope, "stack".into())
        .and_then(|v| v.to_string(&mut cx.scope).map(Value::String))
}
