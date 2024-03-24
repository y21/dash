use std::ops::ControlFlow;

use crate::gc::handle::Handle;
use crate::gc::interner::sym;
use crate::localscope::LocalScope;
use crate::throw;
use crate::value::array::Array;
use crate::value::function::native::CallContext;
use crate::value::object::{NamedObject, Object, PropertyDataDescriptor, PropertyKey, PropertyValue};
use crate::value::ops::conversions::ValueConversion;
use crate::value::root_ext::RootErrExt;
use crate::value::{Root, Typeof, Value, ValueContext};

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    match cx.args.first() {
        Some(v) => v.to_object(cx.scope).map(Value::Object),
        None => Ok(Value::Object(cx.scope.register(NamedObject::new(cx.scope)))),
    }
}

pub fn create(cx: CallContext) -> Result<Value, Value> {
    let prototype = cx.args.first().unwrap_or_undefined();

    let obj = NamedObject::new(cx.scope);
    obj.set_prototype(cx.scope, prototype)?;

    // TODO: second argument: ObjectDefineProperties

    Ok(cx.scope.register(obj).into())
}

pub fn keys(cx: CallContext) -> Result<Value, Value> {
    let obj = cx.args.first().unwrap_or_undefined().to_object(cx.scope)?;
    let keys = obj.own_keys(cx.scope)?;
    let array = Array::from_vec(cx.scope, keys.into_iter().map(PropertyValue::static_default).collect());
    Ok(cx.scope.register(array).into())
}

pub fn to_string(cx: CallContext) -> Result<Value, Value> {
    fn to_string_inner(scope: &mut LocalScope<'_>, o: Handle) -> Result<Value, Value> {
        let constructor = o
            .get_property(scope, sym::constructor.into())
            .root(scope)?
            .get_property(scope, sym::name.into())
            .root(scope)?
            .to_js_string(scope)?;

        let constructor = format!("[object {}]", constructor.res(scope));

        Ok(Value::String(scope.intern(constructor).into()))
    }

    let value = match &cx.this {
        Value::Undefined(_) => Value::String(cx.scope.intern("[object Undefined]").into()),
        Value::Null(_) => Value::String(cx.scope.intern("[object Null]").into()),
        Value::Object(o) => to_string_inner(cx.scope, o.clone())?,
        _ => unreachable!(), // `this` is always object/null/undefined. TODO: wrong, `Object.prototype.toString..call('a')` crashes
    };

    Ok(value)
}

pub fn get_own_property_descriptor(cx: CallContext) -> Result<Value, Value> {
    let o = cx.args.first().unwrap_or_undefined();
    let o = match &o {
        Value::Object(o) => o,
        _ => throw!(
            cx.scope,
            TypeError,
            "Object.getOwnPropertyDescriptor called on non-object"
        ),
    };
    let k = cx.args.get(1).unwrap_or_undefined();
    let k = PropertyKey::from_value(cx.scope, k)?;

    Ok(o.get_property_descriptor(cx.scope, k)
        .root_err(cx.scope)?
        .map(|d| d.to_descriptor_value(cx.scope))
        .transpose()?
        .unwrap_or_undefined())
}

pub fn get_own_property_descriptors(cx: CallContext) -> Result<Value, Value> {
    let o = cx.args.first().unwrap_or_undefined();
    let o = match &o {
        Value::Object(o) => o,
        _ => throw!(
            cx.scope,
            TypeError,
            "Object.getOwnPropertyDescriptors called on non-object"
        ),
    };

    let mut descriptors = Vec::new();
    let keys = o.own_keys(cx.scope)?;

    for key in keys {
        let key = PropertyKey::from_value(cx.scope, key)?;
        let descriptor = o
            .get_property_descriptor(cx.scope, key)
            .root_err(cx.scope)?
            .map(|d| d.to_descriptor_value(cx.scope))
            .transpose()?
            .unwrap_or_undefined();

        descriptors.push(PropertyValue::static_default(descriptor));
    }

    let descriptors = Array::from_vec(cx.scope, descriptors);
    Ok(Value::Object(cx.scope.register(descriptors)))
}

pub fn has_own_property(cx: CallContext) -> Result<Value, Value> {
    let o = match &cx.this {
        Value::Object(o) => o,
        _ => throw!(
            cx.scope,
            TypeError,
            "Object.prototype.hasOwnProperty called on non-object"
        ),
    };

    let key = cx.args.first().unwrap_or_undefined();
    let key = PropertyKey::from_value(cx.scope, key)?;
    let desc = o.get_property_descriptor(cx.scope, key).root_err(cx.scope)?;
    Ok(Value::Boolean(desc.is_some()))
}

pub fn define_property(cx: CallContext) -> Result<Value, Value> {
    let object = match cx.args.first() {
        Some(Value::Object(o)) => o,
        _ => throw!(
            cx.scope,
            TypeError,
            "Object.prototype.hasOwnProperty called on non-object"
        ),
    };

    let property = match cx.args.get(1) {
        Some(Value::Symbol(sym)) => PropertyKey::from(sym.clone()),
        Some(other) => PropertyKey::from(other.to_js_string(cx.scope)?),
        _ => throw!(cx.scope, TypeError, "Property must be a string or symbol"),
    };
    let descriptor = match cx.args.get(2) {
        Some(Value::Object(o)) => o,
        _ => throw!(cx.scope, TypeError, "Property descriptor must be an object"),
    };

    let value = PropertyValue::from_descriptor_value(cx.scope, Value::Object(descriptor.clone()))?;

    object.set_property(cx.scope, property, value)?;

    Ok(Value::Object(object.clone()))
}

pub fn assign(cx: CallContext) -> Result<Value, Value> {
    let mut args = cx.args.into_iter();
    let to = args.next().unwrap_or_undefined().to_object(cx.scope)?;
    for source in args {
        let source = source.to_object(cx.scope)?;
        for key in source.own_keys(cx.scope)? {
            let key = PropertyKey::from_value(cx.scope, key)?;
            let desc = source.get_own_property(cx.scope, key.clone()).root(cx.scope)?;
            to.set_property(cx.scope, key, PropertyValue::static_default(desc))?;
        }
    }
    Ok(Value::Object(to))
}

pub fn entries(cx: CallContext) -> Result<Value, Value> {
    let mut entries = Vec::new();
    let obj = cx.args.first().unwrap_or_undefined().to_object(cx.scope)?;
    for key in obj.own_keys(cx.scope)? {
        let key = PropertyKey::from_value(cx.scope, key)?;
        let value = obj.get_own_property(cx.scope, key.clone()).root(cx.scope)?;
        let entry = Array::from_vec(
            cx.scope,
            vec![
                PropertyValue::static_default(key.as_value()),
                PropertyValue::static_default(value),
            ],
        );
        entries.push(PropertyValue::static_default(Value::Object(cx.scope.register(entry))));
    }

    let entries = Array::from_vec(cx.scope, entries);
    Ok(Value::Object(cx.scope.register(entries)))
}

pub fn get_prototype_of(cx: CallContext) -> Result<Value, Value> {
    let obj = cx.args.first().unwrap_or_undefined().to_object(cx.scope)?;
    obj.get_prototype(cx.scope)
}

pub fn is_prototype_of(cx: CallContext) -> Result<Value, Value> {
    let target_proto = Value::Object(cx.this.to_object(cx.scope)?);
    let this_proto = cx.args.first().unwrap_or_undefined();
    if this_proto.type_of() != Typeof::Object {
        return Ok(Value::Boolean(false));
    }

    Ok(Value::Boolean(
        this_proto
            .for_each_prototype(cx.scope, |_, proto| {
                if proto == &target_proto {
                    Ok(ControlFlow::Break(()))
                } else {
                    Ok(ControlFlow::Continue(()))
                }
            })?
            .is_break(),
    ))
}

pub fn property_is_enumerable(cx: CallContext) -> Result<Value, Value> {
    let prop = PropertyKey::from_value(cx.scope, cx.args.first().unwrap_or_undefined())?;
    let obj = cx.this.to_object(cx.scope)?;
    let desc = obj.get_own_property_descriptor(cx.scope, prop).root_err(cx.scope)?;
    Ok(Value::Boolean(desc.is_some_and(|val| {
        val.descriptor.contains(PropertyDataDescriptor::ENUMERABLE)
    })))
}
