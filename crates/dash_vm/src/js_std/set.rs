use crate::throw;
use crate::value::function::native::CallContext;
use crate::value::object::PropertyKey;
use crate::value::ops::conversions::ValueConversion;
use crate::value::set::Set;
use crate::value::{Root, Value, ValueContext};

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let set = Set::new(cx.scope);
    if let Some(iter) = cx.args.first() {
        let len = iter.length_of_array_like(cx.scope)?;

        for i in 0..len {
            let i = i.to_string();
            let item = iter
                .get_property(cx.scope, PropertyKey::String(i.into()))
                .root(cx.scope)?;

            set.add(item);
        }
    }

    Ok(Value::Object(cx.scope.register(set)))
}

pub fn add(cx: CallContext) -> Result<Value, Value> {
    let this = match cx.this.downcast_ref::<Set>() {
        Some(set) => set,
        _ => throw!(cx.scope, TypeError, "Incompatible receiver"),
    };

    let item = cx.args.first().unwrap_or_undefined();
    this.add(item);

    Ok(cx.this)
}

pub fn has(cx: CallContext) -> Result<Value, Value> {
    let this = match cx.this.downcast_ref::<Set>() {
        Some(set) => set,
        _ => throw!(cx.scope, TypeError, "Incompatible receiver"),
    };

    let item = cx.args.first().unwrap_or_undefined();
    Ok(Value::Boolean(this.has(&item)))
}

pub fn delete(cx: CallContext) -> Result<Value, Value> {
    let this = match cx.this.downcast_ref::<Set>() {
        Some(set) => set,
        _ => throw!(cx.scope, TypeError, "Incompatible receiver"),
    };

    let item = cx.args.first().unwrap_or_undefined();
    let did_delete = this.delete(&item);

    Ok(Value::Boolean(did_delete))
}

pub fn clear(cx: CallContext) -> Result<Value, Value> {
    let this = match cx.this.downcast_ref::<Set>() {
        Some(set) => set,
        _ => throw!(cx.scope, TypeError, "Incompatible receiver"),
    };

    this.clear();

    Ok(Value::undefined())
}

pub fn size(cx: CallContext) -> Result<Value, Value> {
    let this = match cx.this.downcast_ref::<Set>() {
        Some(set) => set,
        _ => throw!(cx.scope, TypeError, "Incompatible receiver"),
    };

    Ok(Value::number(this.size() as f64))
}
