use crate::gc::interner::sym;
use crate::throw;
use crate::value::function::native::CallContext;
use crate::value::ops::conversions::ValueConversion;
use crate::value::{Value, ValueContext};

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let value = cx.args.get(0).unwrap_or_undefined().to_boolean(cx.scope)?;
    Ok(Value::Boolean(value))
}

pub fn to_string(cx: CallContext) -> Result<Value, Value> {
    if let Value::Boolean(b) = cx.this {
        let s = b.then(|| sym::TRUE.into()).unwrap_or_else(|| sym::FALSE.into());

        Ok(Value::String(s))
    } else {
        todo!()
    }
}

pub fn value_of(cx: CallContext) -> Result<Value, Value> {
    match cx.this {
        Value::Boolean(b) => Ok(Value::Boolean(b)),
        _ => throw!(cx.scope, TypeError, "Boolean.valueOf called on non-boolean"),
    }
}
