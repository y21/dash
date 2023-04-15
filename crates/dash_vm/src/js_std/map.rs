use crate::throw;
use crate::value::function::native::CallContext;
use crate::value::map::Map;
use crate::value::object::PropertyKey;
use crate::value::ops::abstractions::conversions::ValueConversion;
use crate::value::Value;
use crate::value::ValueContext;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let map = Map::new(cx.scope);
    if let Some(iter) = cx.args.first() {
        let len = iter.length_of_array_like(cx.scope)?;

        for i in 0..len {
            let i = i.to_string();
            let item = iter.get_property(cx.scope, PropertyKey::String(i.into()))?;
            let k = item.get_property(cx.scope, PropertyKey::String("0".into()))?;
            let v = item.get_property(cx.scope, PropertyKey::String("1".into()))?;
            map.set(k, v);
        }
    }

    Ok(Value::Object(cx.scope.register(map)))
}

pub fn set(cx: CallContext) -> Result<Value, Value> {
    let this = match cx.this.downcast_ref::<Map>() {
        Some(map) => map,
        _ => throw!(cx.scope, TypeError, "Incompatible receiver"),
    };

    let k = cx.args.first().unwrap_or_undefined();
    let v = cx.args.get(1).unwrap_or_undefined();
    this.set(k, v);

    Ok(cx.this)
}

pub fn has(cx: CallContext) -> Result<Value, Value> {
    let this = match cx.this.downcast_ref::<Map>() {
        Some(set) => set,
        _ => throw!(cx.scope, TypeError, "Incompatible receiver"),
    };

    let item = cx.args.first().unwrap_or_undefined();
    Ok(Value::Boolean(this.has(&item)))
}

pub fn get(cx: CallContext) -> Result<Value, Value> {
    let this = match cx.this.downcast_ref::<Map>() {
        Some(set) => set,
        _ => throw!(cx.scope, TypeError, "Incompatible receiver"),
    };

    let item = cx.args.first().unwrap_or_undefined();
    Ok(this.get(&item).unwrap_or_undefined())
}

pub fn delete(cx: CallContext) -> Result<Value, Value> {
    let this = match cx.this.downcast_ref::<Map>() {
        Some(set) => set,
        _ => throw!(cx.scope, TypeError, "Incompatible receiver"),
    };

    let item = cx.args.first().unwrap_or_undefined();
    let did_delete = this.delete(&item);

    Ok(Value::Boolean(did_delete))
}

pub fn clear(cx: CallContext) -> Result<Value, Value> {
    let this = match cx.this.downcast_ref::<Map>() {
        Some(set) => set,
        _ => throw!(cx.scope, TypeError, "Incompatible receiver"),
    };

    this.clear();

    Ok(Value::undefined())
}

pub fn size(cx: CallContext) -> Result<Value, Value> {
    let this = match cx.this.downcast_ref::<Map>() {
        Some(set) => set,
        _ => throw!(cx.scope, TypeError, "Incompatible receiver"),
    };

    Ok(Value::number(this.size() as f64))
}
