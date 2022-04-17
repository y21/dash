use std::any::Any;

use crate::throw;
use crate::vm::value::array::Array;
use crate::vm::value::boxed::Boolean as BoxedBoolean;
use crate::vm::value::boxed::Number as BoxedNumber;
use crate::vm::value::boxed::String as BoxedString;
use crate::vm::value::error::Error;
use crate::vm::value::function::native::CallContext;
use crate::vm::value::function::Function;
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

pub fn to_string(cx: CallContext) -> Result<Value, Value> {
    fn to_string_inner(o: &dyn Any) -> &'static str {
        if o.is::<Array>() {
            "[object Array]"
        } else if o.is::<Function>() {
            "[object Function]"
        } else if o.is::<Error>() {
            "[object Error]"
        } else if o.is::<BoxedBoolean>() {
            "[object Boolean]"
        } else if o.is::<BoxedNumber>() {
            "[object Number]"
        } else if o.is::<BoxedString>() {
            "[object String]"
        } else {
            "[object Object]"
        }
    }

    let value = match &cx.this {
        Value::Undefined(_) => "[object Undefined]",
        Value::Null(_) => "[object Null]",
        Value::Object(o) => to_string_inner(o.as_any()),
        Value::External(o) => to_string_inner(o.as_any()),
        _ => unreachable!(), // `this` is always object/null/undefined
    };

    Ok(Value::String(value.into()))
}
