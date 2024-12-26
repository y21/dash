use crate::throw;
use crate::value::function::native::CallContext;
use crate::value::object::{NamedObject, PropertyKey};
use crate::value::ops::conversions::ValueConversion;
use crate::value::set::Set;
use crate::value::{Root, Unpack, Value, ValueContext};

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
    let this = cx.this.unpack();
    let this = match this.downcast_ref::<Set>(cx.scope) {
        Some(set) => set,
        _ => throw!(cx.scope, TypeError, "Incompatible receiver"),
    };

    let item = cx.args.first().unwrap_or_undefined();
    this.add(item);

    Ok(cx.this)
}

pub fn has(cx: CallContext) -> Result<Value, Value> {
    let this = cx.this.unpack();
    let this = match this.downcast_ref::<Set>(cx.scope) {
        Some(set) => set,
        _ => throw!(cx.scope, TypeError, "Incompatible receiver"),
    };

    let item = cx.args.first().unwrap_or_undefined();
    Ok(Value::boolean(this.has(&item)))
}

pub fn delete(cx: CallContext) -> Result<Value, Value> {
    let this = cx.this.unpack();
    let this = match this.downcast_ref::<Set>(cx.scope) {
        Some(set) => set,
        _ => throw!(cx.scope, TypeError, "Incompatible receiver"),
    };

    let item = cx.args.first().unwrap_or_undefined();
    let did_delete = this.delete(&item);

    Ok(Value::boolean(did_delete))
}

pub fn clear(cx: CallContext) -> Result<Value, Value> {
    let this = cx.this.unpack();
    let this = match this.downcast_ref::<Set>(cx.scope) {
        Some(set) => set,
        _ => throw!(cx.scope, TypeError, "Incompatible receiver"),
    };

    this.clear();

    Ok(Value::undefined())
}

pub fn size(cx: CallContext) -> Result<Value, Value> {
    let this = cx.this.unpack();
    let this = match this.downcast_ref::<Set>(cx.scope) {
        Some(set) => set,
        _ => throw!(cx.scope, TypeError, "Incompatible receiver"),
    };

    Ok(Value::number(this.size() as f64))
}
