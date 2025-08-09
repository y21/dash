use crate::localscope::LocalScope;
use crate::throw;
use crate::value::function::native::CallContext;
use crate::value::object::OrdObject;
use crate::value::ops::conversions::ValueConversion;
use crate::value::propertykey::ToPropertyKey;
use crate::value::weakset::WeakSet;
use crate::value::{Root, Value, ValueContext};

use super::receiver_t;

pub fn constructor(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let Some(new_target) = cx.new_target else {
        throw!(scope, TypeError, "WeakSet constructor requires new")
    };

    let weakset = WeakSet::with_obj(OrdObject::instance_for_new_target(new_target, scope)?);
    if let Some(iter) = cx.args.first() {
        let len = iter.length_of_array_like(scope)?;

        for i in 0..len {
            let item = iter.get_property(i.to_key(scope), scope).root(scope)?;

            weakset.add(item);
        }
    }

    Ok(Value::object(scope.register(weakset)))
}

pub fn add(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let item = cx.args.first().unwrap_or_undefined();
    receiver_t::<WeakSet>(scope, &cx.this, "WeakSet.prototype.add")?.add(item);

    Ok(cx.this)
}

pub fn has(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let item = cx.args.first().unwrap_or_undefined();
    Ok(Value::boolean(
        receiver_t::<WeakSet>(scope, &cx.this, "WeakSet.prototype.has")?.has(&item),
    ))
}

pub fn delete(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let item = cx.args.first().unwrap_or_undefined();
    let did_delete = receiver_t::<WeakSet>(scope, &cx.this, "WeakSet.prototype.delete")?.delete(&item);

    Ok(Value::boolean(did_delete))
}
