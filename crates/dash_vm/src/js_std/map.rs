use dash_middle::interner::sym;
use crate::throw;
use crate::value::function::native::CallContext;
use crate::value::map::Map;
use crate::value::object::PropertyKey;
use crate::value::ops::conversions::ValueConversion;
use crate::value::{Root, Unpack, Value, ValueContext};

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let map = Map::new(cx.scope);
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
            map.set(k, v);
        }
    }

    Ok(Value::object(cx.scope.register(map)))
}

pub fn set(cx: CallContext) -> Result<Value, Value> {
    let this = cx.this.unpack();
    let this = match this.downcast_ref::<Map>(&cx.scope) {
        Some(map) => map,
        _ => throw!(cx.scope, TypeError, "Incompatible receiver"),
    };

    let k = cx.args.first().unwrap_or_undefined();
    let v = cx.args.get(1).unwrap_or_undefined();
    this.set(k, v);

    Ok(cx.this)
}

pub fn has(cx: CallContext) -> Result<Value, Value> {
    let this = cx.this.unpack();
    let this = match this.downcast_ref::<Map>(&cx.scope) {
        Some(map) => map,
        _ => throw!(cx.scope, TypeError, "Incompatible receiver"),
    };

    let item = cx.args.first().unwrap_or_undefined();
    Ok(Value::boolean(this.has(&item)))
}

pub fn get(cx: CallContext) -> Result<Value, Value> {
    let this = cx.this.unpack();
    let this = match this.downcast_ref::<Map>(&cx.scope) {
        Some(map) => map,
        _ => throw!(cx.scope, TypeError, "Incompatible receiver"),
    };

    let item = cx.args.first().unwrap_or_undefined();
    Ok(this.get(&item).unwrap_or_undefined())
}

pub fn delete(cx: CallContext) -> Result<Value, Value> {
    let this = cx.this.unpack();
    let this = match this.downcast_ref::<Map>(&cx.scope) {
        Some(map) => map,
        _ => throw!(cx.scope, TypeError, "Incompatible receiver"),
    };

    let item = cx.args.first().unwrap_or_undefined();
    let did_delete = this.delete(&item);

    Ok(Value::boolean(did_delete))
}

pub fn clear(cx: CallContext) -> Result<Value, Value> {
    let this = cx.this.unpack();
    let this = match this.downcast_ref::<Map>(&cx.scope) {
        Some(map) => map,
        _ => throw!(cx.scope, TypeError, "Incompatible receiver"),
    };

    this.clear();

    Ok(Value::undefined())
}

pub fn size(cx: CallContext) -> Result<Value, Value> {
    let this = cx.this.unpack();
    let this = match this.downcast_ref::<Map>(&cx.scope) {
        Some(map) => map,
        _ => throw!(cx.scope, TypeError, "Incompatible receiver"),
    };

    Ok(Value::number(this.size() as f64))
}
