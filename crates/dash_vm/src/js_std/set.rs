use crate::localscope::LocalScope;
use crate::throw;
use crate::value::function::native::CallContext;
use crate::value::object::OrdObject;
use crate::value::ops::conversions::ValueConversion;
use crate::value::propertykey::ToPropertyKey;
use crate::value::set::Set;
use crate::value::{Root, Value, ValueContext};

use super::receiver_t;

pub fn constructor(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let Some(new_target) = cx.new_target else {
        throw!(scope, TypeError, "Set constructor requires new")
    };

    let set = Set::with_obj(OrdObject::instance_for_new_target(new_target, scope)?);
    if let Some(iter) = cx.args.first() {
        let len = iter.length_of_array_like(scope)?;

        for i in 0..len {
            let item = iter.get_property(i.to_key(scope), scope).root(scope)?;
            set.add(item);
        }
    }

    Ok(Value::object(scope.register(set)))
}

pub fn add(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let item = cx.args.first().unwrap_or_undefined();
    receiver_t::<Set>(scope, &cx.this, "Set.prototype.add")?.add(item);

    Ok(cx.this)
}

pub fn has(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let item = cx.args.first().unwrap_or_undefined();
    Ok(Value::boolean(
        receiver_t::<Set>(scope, &cx.this, "Set.prototype.has")?.has(&item),
    ))
}

pub fn delete(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let item = cx.args.first().unwrap_or_undefined();
    let did_delete = receiver_t::<Set>(scope, &cx.this, "Set.prototype.delete")?.delete(&item);

    Ok(Value::boolean(did_delete))
}

pub fn clear(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    receiver_t::<Set>(scope, &cx.this, "Set.prototype.clear")?.clear();

    Ok(Value::undefined())
}

pub fn size(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    Ok(Value::number(
        receiver_t::<Set>(scope, &cx.this, "Set.prototype.size")?.size() as f64,
    ))
}
