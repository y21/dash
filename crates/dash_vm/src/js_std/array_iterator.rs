use crate::value::array::ArrayIterator;
use crate::value::function::native::CallContext;
use crate::value::object::{OrdObject, Object, PropertyValue};
use crate::value::propertykey::ToPropertyKey;
use crate::value::{Root, Value, ValueContext};
use dash_middle::interner::sym;

use super::receiver_t;

pub fn next(cx: CallContext) -> Result<Value, Value> {
    let iterator = receiver_t::<ArrayIterator>(cx.scope, &cx.this, "ArrayIterator.prototype.next")?;

    let next = iterator.next(cx.scope).root(cx.scope)?;
    let done = next.is_none();

    let obj = OrdObject::new(cx.scope);
    obj.set_property(
        sym::value.to_key(cx.scope),
        PropertyValue::static_default(next.unwrap_or_undefined()),
        cx.scope,
    )?;
    obj.set_property(
        sym::done.to_key(cx.scope),
        PropertyValue::static_default(Value::boolean(done)),
        cx.scope,
    )?;

    Ok(cx.scope.register(obj).into())
}
