use std::ops::ControlFlow;

use crate::gc::ObjectId;
use crate::localscope::LocalScope;
use crate::throw;
use crate::value::array::Array;
use crate::value::function::native::CallContext;
use crate::value::object::{IntegrityLevel, Object, OrdObject, PropertyDataDescriptor, PropertyValue};
use crate::value::ops::conversions::ValueConversion;
use crate::value::propertykey::{PropertyKey, ToPropertyKey};
use crate::value::root_ext::RootErrExt;
use crate::value::{Root, Typeof, Unpack, Value, ValueContext, ValueKind};
use dash_middle::interner::sym;

pub fn constructor(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    match cx.args.first() {
        Some(v) => v.to_object(scope).map(Value::object),
        None => {
            let new_target = cx.new_target.unwrap_or(scope.statics.object_ctor);
            let instance = OrdObject::instance_for_new_target(new_target, scope)?;
            Ok(Value::object(scope.register(instance)))
        }
    }
}

pub fn create(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let prototype = cx.args.first().unwrap_or_undefined();

    let new_target = cx.new_target.unwrap_or(scope.statics.object_ctor);
    let obj = OrdObject::instance_for_new_target(new_target, scope)?;
    obj.set_prototype(prototype, scope)?;

    // TODO: second argument: ObjectDefineProperties

    Ok(scope.register(obj).into())
}

pub fn keys(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let obj = cx.args.first().unwrap_or_undefined().to_object(scope)?;
    // FIXME: own_keys should probably takes an `enumerable: bool`
    let keys = obj.own_keys(scope)?;
    let array = Array::from_vec(keys.into_iter().map(PropertyValue::static_default).collect(), scope);
    Ok(scope.register(array).into())
}

pub fn to_string(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    fn to_string_inner(scope: &mut LocalScope<'_>, o: ObjectId) -> Result<Value, Value> {
        let constructor = o
            .get_property(sym::constructor.to_key(scope), scope)
            .root(scope)?
            .get_property(sym::name.to_key(scope), scope)
            .root(scope)?
            .to_js_string(scope)?;

        let constructor = format!("[object {}]", constructor.res(scope));

        Ok(Value::string(scope.intern(constructor).into()))
    }

    let value = match cx.this.unpack() {
        ValueKind::Undefined(_) => Value::string(scope.intern("[object Undefined]").into()),
        ValueKind::Null(_) => Value::string(scope.intern("[object Null]").into()),
        _ => {
            let object = cx.this.to_object(scope)?;
            to_string_inner(scope, object)?
        }
    };

    Ok(value)
}

pub fn get_own_property_descriptor(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let o = cx.args.first().unwrap_or_undefined();
    let o = match o.unpack() {
        ValueKind::Object(o) => o,
        _ => throw!(scope, TypeError, "Object.getOwnPropertyDescriptor called on non-object"),
    };
    let k = cx.args.get(1).unwrap_or_undefined();
    let k = PropertyKey::from_value(scope, k)?;

    Ok(o.get_property_descriptor(k, scope)
        .root_err(scope)?
        .map(|d| d.to_descriptor_value(scope))
        .transpose()?
        .unwrap_or_undefined())
}

pub fn get_own_property_descriptors(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let o = cx.args.first().unwrap_or_undefined();
    let o = match o.unpack() {
        ValueKind::Object(o) => o,
        _ => throw!(
            scope,
            TypeError,
            "Object.getOwnPropertyDescriptors called on non-object"
        ),
    };

    let mut descriptors = Vec::new();
    let keys = o.own_keys(scope)?;

    for key in keys {
        let key = PropertyKey::from_value(scope, key)?;
        let descriptor = o
            .get_property_descriptor(key, scope)
            .root_err(scope)?
            .map(|d| d.to_descriptor_value(scope))
            .transpose()?
            .unwrap_or_undefined();

        descriptors.push(PropertyValue::static_default(descriptor));
    }

    let descriptors = Array::from_vec(descriptors, scope);
    Ok(Value::object(scope.register(descriptors)))
}

pub fn has_own_property(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let o = match cx.this.unpack() {
        ValueKind::Object(o) => o,
        _ => throw!(scope, TypeError, "Object.prototype.hasOwnProperty called on non-object"),
    };

    let key = cx.args.first().unwrap_or_undefined();
    let key = PropertyKey::from_value(scope, key)?;
    let desc = o.get_property_descriptor(key, scope).root_err(scope)?;
    Ok(Value::boolean(desc.is_some()))
}

pub fn define_property(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let object = match cx.args.first().unpack() {
        Some(ValueKind::Object(o)) => o,
        _ => throw!(scope, TypeError, "Object.prototype.hasOwnProperty called on non-object"),
    };

    let property = match cx.args.get(1) {
        Some(other) => {
            if let ValueKind::Symbol(sym) = other.unpack() {
                sym.to_key(scope)
            } else {
                // TODO: we should just do this in PropertyKey directly
                other.to_js_string(scope)?.to_key(scope)
            }
        }
        _ => throw!(scope, TypeError, "Property must be a string or symbol"),
    };
    let descriptor = match cx.args.get(2).unpack() {
        Some(ValueKind::Object(o)) => o,
        _ => throw!(scope, TypeError, "Property descriptor must be an object"),
    };

    let value = PropertyValue::from_descriptor_value(scope, Value::object(descriptor))?;

    object.set_property(property, value, scope)?;

    Ok(Value::object(object))
}

pub fn define_properties(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let object = match cx.args.first().unpack() {
        Some(ValueKind::Object(o)) => o,
        _ => throw!(scope, TypeError, "Object.prototype.hasOwnProperty called on non-object"),
    };

    let properties = cx.args.get(1).unwrap_or_undefined();
    for key in properties.own_keys(scope)? {
        let key = key.to_js_string(scope)?;
        let descriptor = properties.get_property(key.to_key(scope), scope).root(scope)?;
        let descriptor = PropertyValue::from_descriptor_value(scope, descriptor)?;
        object.set_property(key.to_key(scope), descriptor, scope)?;
    }

    Ok(Value::object(object))
}

pub fn assign(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let mut args = cx.args.into_iter();
    let to = args.next().unwrap_or_undefined().to_object(scope)?;
    for source in args {
        let source = source.to_object(scope)?;
        for key in source.own_keys(scope)? {
            let key = PropertyKey::from_value(scope, key)?;
            let desc = source.get_own_property(key, scope).root(scope)?;
            to.set_property(key, PropertyValue::static_default(desc), scope)?;
        }
    }
    Ok(Value::object(to))
}

pub fn entries(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let mut entries = Vec::new();
    let obj = cx.args.first().unwrap_or_undefined().to_object(scope)?;
    for key in obj.own_keys(scope)? {
        let key = PropertyKey::from_value(scope, key)?;
        let value = obj.get_own_property(key, scope).root(scope)?;
        let entry = Array::from_vec(
            vec![
                PropertyValue::static_default(key.to_value(scope)),
                PropertyValue::static_default(value),
            ],
            scope,
        );
        entries.push(PropertyValue::static_default(Value::object(scope.register(entry))));
    }

    let entries = Array::from_vec(entries, scope);
    Ok(Value::object(scope.register(entries)))
}

pub fn get_prototype_of(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let obj = cx.args.first().unwrap_or_undefined().to_object(scope)?;
    obj.get_prototype(scope)
}

pub fn set_prototype_of(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let obj = cx.args.first().unwrap_or_undefined().to_object(scope)?;
    let target = cx.args.get(1).unwrap_or_undefined();
    obj.set_prototype(target, scope)?;
    Ok(Value::object(obj))
}

pub fn is_prototype_of(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let target_proto = Value::object(cx.this.to_object(scope)?);
    let this_proto = cx.args.first().unwrap_or_undefined();
    if this_proto.type_of(scope) != Typeof::Object {
        return Ok(Value::boolean(false));
    }

    Ok(Value::boolean(
        this_proto
            .for_each_prototype(scope, |_, proto| {
                if proto == &target_proto {
                    Ok(ControlFlow::Break(()))
                } else {
                    Ok(ControlFlow::Continue(()))
                }
            })?
            .is_break(),
    ))
}

pub fn property_is_enumerable(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let prop = PropertyKey::from_value(scope, cx.args.first().unwrap_or_undefined())?;
    let obj = cx.this.to_object(scope)?;
    let desc = obj.get_own_property_descriptor(prop, scope).root_err(scope)?;
    Ok(Value::boolean(desc.is_some_and(|val| {
        val.descriptor.contains(PropertyDataDescriptor::ENUMERABLE)
    })))
}

pub fn freeze(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let arg = cx.args.first().unwrap_or_undefined();
    if let ValueKind::Object(o) = arg.unpack() {
        o.set_integrity_level(IntegrityLevel::Frozen, scope)?;
        Ok(Value::object(o))
    } else {
        Ok(arg)
    }
}

pub fn seal(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let arg = cx.args.first().unwrap_or_undefined();
    if let ValueKind::Object(o) = arg.unpack() {
        o.set_integrity_level(IntegrityLevel::Sealed, scope)?;
        Ok(Value::object(o))
    } else {
        Ok(arg)
    }
}
