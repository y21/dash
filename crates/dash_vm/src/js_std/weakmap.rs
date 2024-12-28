use dash_middle::interner::sym;

use crate::throw;
use crate::value::function::native::CallContext;
use crate::value::object::{NamedObject, PropertyKey};
use crate::value::ops::conversions::ValueConversion;
use crate::value::weakmap::WeakMap;
use crate::value::{Root, Value, ValueContext};

use super::receiver_t;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let Some(new_target) = cx.new_target else {
        throw!(cx.scope, TypeError, "WeakMap constructor requires new")
    };

    let weakmap = WeakMap::with_obj(NamedObject::instance_for_new_target(new_target, cx.scope)?);
    if let Some(iter) = cx.args.first() {
        let len = iter.length_of_array_like(cx.scope)?;

        for i in 0..len {
            let i = cx.scope.intern_usize(i);
            let item = iter
                .get_property(cx.scope, PropertyKey::String(i.into()))
                .root(cx.scope)?;
            let k = item
                .get_property(cx.scope, PropertyKey::String(sym::zero.into()))
                .root(cx.scope)?;
            let v = item
                .get_property(cx.scope, PropertyKey::String(sym::one.into()))
                .root(cx.scope)?;
            weakmap.set(k, v);
        }
    }

    Ok(Value::object(cx.scope.register(weakmap)))
}

pub fn set(cx: CallContext) -> Result<Value, Value> {
    let k = cx.args.first().unwrap_or_undefined();
    let v = cx.args.get(1).unwrap_or_undefined();
    receiver_t::<WeakMap>(cx.scope, &cx.this, "WeakMap.prototype.set")?.set(k, v);

    Ok(cx.this)
}

pub fn has(cx: CallContext) -> Result<Value, Value> {
    let item = cx.args.first().unwrap_or_undefined();
    Ok(Value::boolean(
        receiver_t::<WeakMap>(cx.scope, &cx.this, "WeakMap.prototype.has")?.has(&item),
    ))
}

pub fn get(cx: CallContext) -> Result<Value, Value> {
    let item = cx.args.first().unwrap_or_undefined();
    Ok(receiver_t::<WeakMap>(cx.scope, &cx.this, "WeakMap.prototype.get")?
        .get(&item)
        .unwrap_or_undefined())
}

pub fn delete(cx: CallContext) -> Result<Value, Value> {
    let item = cx.args.first().unwrap_or_undefined();
    let did_delete = receiver_t::<WeakMap>(cx.scope, &cx.this, "WeakMap.prototype.delete")?.delete(&item);
    Ok(Value::boolean(did_delete))
}
