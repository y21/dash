use crate::throw;
use crate::vm::value::function::native::CallContext;
use crate::vm::value::ops::abstractions::conversions::ValueConversion;
use crate::vm::value::Value;
use crate::vm::value::ValueContext;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let value = cx.args.get(0).unwrap_or_undefined().to_boolean()?;
    Ok(Value::Boolean(value))
}

pub fn to_string(cx: CallContext) -> Result<Value, Value> {
    if let Value::Boolean(b) = cx.this {
        let s = b
            .then(|| cx.scope.statics().get_true())
            .unwrap_or_else(|| cx.scope.statics().get_false());

        Ok(Value::String(s))
    } else {
        todo!()
    }
}

pub fn value_of(cx: CallContext) -> Result<Value, Value> {
    match cx.this {
        Value::Boolean(b) => Ok(Value::Boolean(b)),
        _ => throw!(cx.scope, "Boolean.valueOf called on non-boolean"),
    }
}
