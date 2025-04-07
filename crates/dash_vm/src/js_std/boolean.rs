use crate::throw;
use crate::value::function::native::CallContext;
use crate::value::object::{Object, OrdObject};
use crate::value::ops::conversions::ValueConversion;
use crate::value::{Value, ValueContext, boxed};
use dash_middle::interner::sym;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let value = cx.args.first().unwrap_or_undefined().to_boolean(cx.scope)?;

    if let Some(new_target) = cx.new_target {
        let value = boxed::Boolean::with_obj(value, OrdObject::instance_for_new_target(new_target, cx.scope)?);
        Ok(Value::object(cx.scope.register(value)))
    } else {
        Ok(Value::boolean(value))
    }
}

pub fn to_string(cx: CallContext) -> Result<Value, Value> {
    if let Some(value) = cx
        .this
        .internal_slots(cx.scope)
        .and_then(|slots| slots.boolean_value(cx.scope))
    {
        Ok(Value::string(if value {
            sym::true_.into()
        } else {
            sym::false_.into()
        }))
    } else {
        throw!(
            cx.scope,
            TypeError,
            "Boolean.prototype.toString called on non-boolean value"
        )
    }
}

pub fn value_of(cx: CallContext) -> Result<Value, Value> {
    if let Some(value) = cx
        .this
        .internal_slots(cx.scope)
        .and_then(|slots| slots.boolean_value(cx.scope))
    {
        Ok(Value::boolean(value))
    } else {
        throw!(
            cx.scope,
            TypeError,
            "Boolean.prototype.valueOf called on non-boolean value"
        )
    }
}
