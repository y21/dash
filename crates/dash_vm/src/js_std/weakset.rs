use crate::throw;
use crate::value::function::native::CallContext;
use crate::value::object::NamedObject;
use crate::value::ops::conversions::ValueConversion;
use crate::value::propertykey::PropertyKey;
use crate::value::weakset::WeakSet;
use crate::value::{Root, Value, ValueContext};

use super::receiver_t;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let Some(new_target) = cx.new_target else {
        throw!(cx.scope, TypeError, "WeakSet constructor requires new")
    };

    let weakset = WeakSet::with_obj(NamedObject::instance_for_new_target(new_target, cx.scope)?);
    if let Some(iter) = cx.args.first() {
        let len = iter.length_of_array_like(cx.scope)?;

        for i in 0..len {
            let i = cx.scope.intern_usize(i);
            let item = iter
                .get_property(PropertyKey::String(i.into()), cx.scope)
                .root(cx.scope)?;

            weakset.add(item);
        }
    }

    Ok(Value::object(cx.scope.register(weakset)))
}

pub fn add(cx: CallContext) -> Result<Value, Value> {
    let item = cx.args.first().unwrap_or_undefined();
    receiver_t::<WeakSet>(cx.scope, &cx.this, "WeakSet.prototype.add")?.add(item);

    Ok(cx.this)
}

pub fn has(cx: CallContext) -> Result<Value, Value> {
    let item = cx.args.first().unwrap_or_undefined();
    Ok(Value::boolean(
        receiver_t::<WeakSet>(cx.scope, &cx.this, "WeakSet.prototype.has")?.has(&item),
    ))
}

pub fn delete(cx: CallContext) -> Result<Value, Value> {
    let item = cx.args.first().unwrap_or_undefined();
    let did_delete = receiver_t::<WeakSet>(cx.scope, &cx.this, "WeakSet.prototype.delete")?.delete(&item);

    Ok(Value::boolean(did_delete))
}
