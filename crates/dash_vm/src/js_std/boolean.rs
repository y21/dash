use crate::gc::interner::sym;
use crate::throw;
use crate::value::function::native::CallContext;
use crate::value::object::Object;
use crate::value::ops::conversions::ValueConversion;
use crate::value::primitive::InternalSlots;
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
    if let Some(value) = cx.this.internal_slots().and_then(InternalSlots::boolean_value) {
        Ok(Value::String(
            value.then(|| sym::true_.into()).unwrap_or_else(|| sym::false_.into()),
        ))
    } else {
        throw!(
            cx.scope,
            TypeError,
            "Boolean.prototype.toString called on non-boolean value"
        )
    }
}

pub fn value_of(cx: CallContext) -> Result<Value, Value> {
    if let Some(value) = cx.this.internal_slots().and_then(InternalSlots::boolean_value) {
        Ok(Value::Boolean(value))
    } else {
        throw!(
            cx.scope,
            TypeError,
            "Boolean.prototype.valueOf called on non-boolean value"
        )
    }
}
