use crate::value::error::Error;
use crate::value::function::native::CallContext;
use crate::value::ops::abstractions::conversions::ValueConversion;
use crate::value::Value;

pub fn constructor(mut cx: CallContext) -> Result<Value, Value> {
    let message = cx
        .args
        .first()
        .cloned()
        .map(|v| v.to_string(&mut cx.scope))
        .transpose()?;

    let err = Error::new(&mut cx.scope, message.as_deref().unwrap_or_default());

    Ok(cx.scope.register(err).into())
}

pub fn to_string(mut cx: CallContext) -> Result<Value, Value> {
    cx.this
        .get_property(&mut cx.scope, "stack".into())
        .and_then(|v| v.to_string(&mut cx.scope).map(Value::String))
}
