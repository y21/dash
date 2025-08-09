use crate::localscope::LocalScope;
use crate::throw;
use crate::value::function::native::CallContext;
use crate::value::map::Map;
use crate::value::object::OrdObject;
use crate::value::ops::conversions::ValueConversion;
use crate::value::propertykey::ToPropertyKey;
use crate::value::{Root, Value, ValueContext};
use dash_middle::interner::sym;

use super::receiver_t;

pub fn constructor(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let Some(new_target) = cx.new_target else {
        throw!(scope, TypeError, "Map constructor requires new")
    };

    let map = Map::with_obj(OrdObject::instance_for_new_target(new_target, scope)?);
    if let Some(iter) = cx.args.first() {
        let len = iter.length_of_array_like(scope)?;

        for i in 0..len {
            let item = iter.get_property(i.to_key(scope), scope).root(scope)?;
            let k = item.get_property(sym::zero.to_key(scope), scope).root(scope)?;
            let v = item.get_property(sym::one.to_key(scope), scope).root(scope)?;
            map.set(k, v);
        }
    }

    Ok(Value::object(scope.register(map)))
}

pub fn set(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let k = cx.args.first().unwrap_or_undefined();
    let v = cx.args.get(1).unwrap_or_undefined();
    receiver_t::<Map>(scope, &cx.this, "Map.prototype.set")?.set(k, v);

    Ok(cx.this)
}

pub fn has(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let item = cx.args.first().unwrap_or_undefined();
    Ok(Value::boolean(
        receiver_t::<Map>(scope, &cx.this, "Map.prototype.has")?.has(&item),
    ))
}

pub fn get(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let item = cx.args.first().unwrap_or_undefined();
    Ok(receiver_t::<Map>(scope, &cx.this, "Map.prototype.get")?
        .get(&item)
        .unwrap_or_undefined())
}

pub fn delete(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let item = cx.args.first().unwrap_or_undefined();
    let did_delete = receiver_t::<Map>(scope, &cx.this, "Map.prototype.delete")?.delete(&item);
    Ok(Value::boolean(did_delete))
}

pub fn clear(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    receiver_t::<Map>(scope, &cx.this, "Map.prototype.clear")?.clear();
    Ok(Value::undefined())
}

pub fn size(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    Ok(Value::number(
        receiver_t::<Map>(scope, &cx.this, "Map.prototype.size")?.size() as f64,
    ))
}
