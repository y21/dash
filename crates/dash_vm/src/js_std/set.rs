use crate::throw;
use crate::value::function::native::CallContext;
use crate::value::object::{NamedObject, PropertyKey};
use crate::value::ops::conversions::ValueConversion;
use crate::value::set::Set;
use crate::value::{Root, Value, ValueContext};

use super::receiver_t;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let Some(new_target) = cx.new_target else {
        throw!(cx.scope, TypeError, "Set constructor requires new")
    };

    let set = Set::with_obj(NamedObject::instance_for_new_target(new_target, cx.scope)?);
    if let Some(iter) = cx.args.first() {
        let len = iter.length_of_array_like(cx.scope)?;

        for i in 0..len {
            let i = cx.scope.intern_usize(i);
            let item = iter
                .get_property(cx.scope, PropertyKey::String(i.into()))
                .root(cx.scope)?;

            set.add(item);
        }
    }

    Ok(Value::object(cx.scope.register(set)))
}

pub fn add(cx: CallContext) -> Result<Value, Value> {
    let item = cx.args.first().unwrap_or_undefined();
    receiver_t::<Set>(cx.scope, &cx.this, "Set.prototype.add")?.add(item);

    Ok(cx.this)
}

pub fn has(cx: CallContext) -> Result<Value, Value> {
    let item = cx.args.first().unwrap_or_undefined();
    Ok(Value::boolean(
        receiver_t::<Set>(cx.scope, &cx.this, "Set.prototype.has")?.has(&item),
    ))
}

pub fn delete(cx: CallContext) -> Result<Value, Value> {
    let item = cx.args.first().unwrap_or_undefined();
    let did_delete = receiver_t::<Set>(cx.scope, &cx.this, "Set.prototype.delete")?.delete(&item);

    Ok(Value::boolean(did_delete))
}

pub fn clear(cx: CallContext) -> Result<Value, Value> {
    receiver_t::<Set>(cx.scope, &cx.this, "Set.prototype.clear")?.clear();

    Ok(Value::undefined())
}

pub fn size(cx: CallContext) -> Result<Value, Value> {
    Ok(Value::number(
        receiver_t::<Set>(cx.scope, &cx.this, "Set.prototype.size")?.size() as f64,
    ))
}
