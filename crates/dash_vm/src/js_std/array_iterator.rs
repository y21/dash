use crate::localscope::LocalScope;
use crate::value::array::ArrayIterator;
use crate::value::function::native::CallContext;
use crate::value::object::{Object, OrdObject, PropertyValue};
use crate::value::propertykey::ToPropertyKey;
use crate::value::{Root, Value, ValueContext};
use dash_middle::interner::sym;

use super::receiver_t;

pub fn next(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let iterator = receiver_t::<ArrayIterator>(scope, &cx.this, "ArrayIterator.prototype.next")?;

    let next = iterator.next(scope).root(scope)?;
    let done = next.is_none();

    let obj = OrdObject::new(scope);
    obj.set_property(
        sym::value.to_key(scope),
        PropertyValue::static_default(next.unwrap_or_undefined()),
        scope,
    )?;
    obj.set_property(
        sym::done.to_key(scope),
        PropertyValue::static_default(Value::boolean(done)),
        scope,
    )?;

    Ok(scope.register(obj).into())
}
