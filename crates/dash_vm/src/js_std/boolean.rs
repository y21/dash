use crate::gc::interner::sym;
use crate::throw;
use crate::value::function::native::CallContext;
use crate::value::ops::conversions::ValueConversion;
use crate::value::{boxed, Value, ValueContext};

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let value = cx.args.first().unwrap_or_undefined().to_boolean(cx.scope)?;
    if cx.is_constructor_call {
        let value = boxed::Boolean::new(cx.scope, value);
        Ok(Value::Object(cx.scope.register(value)))
    } else {
        Ok(Value::Boolean(value))
    }
}

pub fn to_string(cx: CallContext) -> Result<Value, Value> {
    if let Value::Boolean(b) = cx.this {
        let s = b.then(|| sym::true_.into()).unwrap_or_else(|| sym::false_.into());

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
