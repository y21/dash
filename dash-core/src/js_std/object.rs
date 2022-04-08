use crate::throw;
use crate::vm::value::array::Array;
use crate::vm::value::function::native::CallContext;
use crate::vm::value::object::NamedObject;
use crate::vm::value::object::Object;
use crate::vm::value::ops::abstractions::conversions::ValueConversion;
use crate::vm::value::Value;
use crate::vm::value::ValueContext;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    throw!(cx.scope, "unimplemented")
}

pub fn create(cx: CallContext) -> Result<Value, Value> {
    let prototype = cx.args.first().unwrap_or_undefined();

    let obj = NamedObject::new(cx.scope);
    obj.set_prototype(cx.scope, prototype)?;

    // TODO: second argument: ObjectDefineProperties

    Ok(cx.scope.gc_mut().register(obj).into())
}

pub fn keys(cx: CallContext) -> Result<Value, Value> {
    let obj = cx.args.first().unwrap_or_undefined().to_object(cx.scope)?;
    let keys = obj.own_keys()?;
    let array = Array::from_vec(cx.scope, keys);
    Ok(cx.scope.gc_mut().register(array).into())
}
