use std::ops::ControlFlow;

use crate::gc::ObjectId;
use crate::localscope::LocalScope;
use crate::throw;
use crate::value::array::Array;
use crate::value::function::native::CallContext;
use crate::value::object::{IntegrityLevel, OrdObject, Object, PropertyDataDescriptor, PropertyValue};
use crate::value::ops::conversions::ValueConversion;
use crate::value::propertykey::{PropertyKey, ToPropertyKey};
use crate::value::root_ext::RootErrExt;
use crate::value::{Root, Typeof, Unpack, Value, ValueContext, ValueKind};
use dash_middle::interner::sym;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    match cx.args.first() {
        Some(v) => v.to_object(cx.scope).map(Value::object),
        None => {
            let new_target = cx.new_target.unwrap_or(cx.scope.statics.object_ctor);
            let instance = OrdObject::instance_for_new_target(new_target, cx.scope)?;
            Ok(Value::object(cx.scope.register(instance)))
        }
    }
}

pub fn create(cx: CallContext) -> Result<Value, Value> {
    let prototype = cx.args.first().unwrap_or_undefined();

    let new_target = cx.new_target.unwrap_or(cx.scope.statics.object_ctor);
    let obj = OrdObject::instance_for_new_target(new_target, cx.scope)?;
    obj.set_prototype(prototype, cx.scope)?;

    // TODO: second argument: ObjectDefineProperties

    Ok(cx.scope.register(obj).into())
}

pub fn keys(cx: CallContext) -> Result<Value, Value> {
    let obj = cx.args.first().unwrap_or_undefined().to_object(cx.scope)?;
    // FIXME: own_keys should probably takes an `enumerable: bool`
    let keys = obj.own_keys(cx.scope)?;
    let array = Array::from_vec(keys.into_iter().map(PropertyValue::static_default).collect(), cx.scope);
    Ok(cx.scope.register(array).into())
}

pub fn to_string(cx: CallContext) -> Result<Value, Value> {
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
        ValueKind::Undefined(_) => Value::string(cx.scope.intern("[object Undefined]").into()),
        ValueKind::Null(_) => Value::string(cx.scope.intern("[object Null]").into()),
        _ => {
            let object = cx.this.to_object(cx.scope)?;
            to_string_inner(cx.scope, object)?
        }
    };

    Ok(value)
}

pub fn get_own_property_descriptor(cx: CallContext) -> Result<Value, Value> {
    let o = cx.args.first().unwrap_or_undefined();
    let o = match o.unpack() {
        ValueKind::Object(o) => o,
        _ => throw!(
            cx.scope,
            TypeError,
            "Object.getOwnPropertyDescriptor called on non-object"
        ),
    };
    let k = cx.args.get(1).unwrap_or_undefined();
    let k = PropertyKey::from_value(cx.scope, k)?;

    Ok(o.get_property_descriptor(k, cx.scope)
        .root_err(cx.scope)?
        .map(|d| d.to_descriptor_value(cx.scope))
        .transpose()?
        .unwrap_or_undefined())
}

pub fn get_own_property_descriptors(cx: CallContext) -> Result<Value, Value> {
    let o = cx.args.first().unwrap_or_undefined();
    let o = match o.unpack() {
        ValueKind::Object(o) => o,
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
            .get_property_descriptor(key, cx.scope)
            .root_err(cx.scope)?
            .map(|d| d.to_descriptor_value(cx.scope))
            .transpose()?
            .unwrap_or_undefined();

        descriptors.push(PropertyValue::static_default(descriptor));
    }

    let descriptors = Array::from_vec(descriptors, cx.scope);
    Ok(Value::object(cx.scope.register(descriptors)))
}

pub fn has_own_property(cx: CallContext) -> Result<Value, Value> {
    let o = match cx.this.unpack() {
        ValueKind::Object(o) => o,
        _ => throw!(
            cx.scope,
            TypeError,
            "Object.prototype.hasOwnProperty called on non-object"
        ),
    };

    let key = cx.args.first().unwrap_or_undefined();
    let key = PropertyKey::from_value(cx.scope, key)?;
    let desc = o.get_property_descriptor(key, cx.scope).root_err(cx.scope)?;
    Ok(Value::boolean(desc.is_some()))
}

pub fn define_property(cx: CallContext) -> Result<Value, Value> {
    let object = match cx.args.first().unpack() {
        Some(ValueKind::Object(o)) => o,
        _ => throw!(
            cx.scope,
            TypeError,
            "Object.prototype.hasOwnProperty called on non-object"
        ),
    };

    let property = match cx.args.get(1) {
        Some(other) => {
            if let ValueKind::Symbol(sym) = other.unpack() {
                sym.to_key(cx.scope)
            } else {
                // TODO: we should just do this in PropertyKey directly
                other.to_js_string(cx.scope)?.to_key(cx.scope)
            }
        }
        _ => throw!(cx.scope, TypeError, "Property must be a string or symbol"),
    };
    let descriptor = match cx.args.get(2).unpack() {
        Some(ValueKind::Object(o)) => o,
        _ => throw!(cx.scope, TypeError, "Property descriptor must be an object"),
    };

    let value = PropertyValue::from_descriptor_value(cx.scope, Value::object(descriptor))?;

    object.set_property(property, value, cx.scope)?;

    Ok(Value::object(object))
}

pub fn define_properties(cx: CallContext) -> Result<Value, Value> {
    let object = match cx.args.first().unpack() {
        Some(ValueKind::Object(o)) => o,
        _ => throw!(
            cx.scope,
            TypeError,
            "Object.prototype.hasOwnProperty called on non-object"
        ),
    };

    let properties = cx.args.get(1).unwrap_or_undefined();
    for key in properties.own_keys(cx.scope)? {
        let key = key.to_js_string(cx.scope)?;
        let descriptor = properties.get_property(key.to_key(cx.scope), cx.scope).root(cx.scope)?;
        let descriptor = PropertyValue::from_descriptor_value(cx.scope, descriptor)?;
        object.set_property(key.to_key(cx.scope), descriptor, cx.scope)?;
    }

    Ok(Value::object(object))
}

pub fn assign(cx: CallContext) -> Result<Value, Value> {
    let mut args = cx.args.into_iter();
    let to = args.next().unwrap_or_undefined().to_object(cx.scope)?;
    for source in args {
        let source = source.to_object(cx.scope)?;
        for key in source.own_keys(cx.scope)? {
            let key = PropertyKey::from_value(cx.scope, key)?;
            let desc = source.get_own_property(key, cx.scope).root(cx.scope)?;
            to.set_property(key, PropertyValue::static_default(desc), cx.scope)?;
        }
    }
    Ok(Value::object(to))
}

pub fn entries(cx: CallContext) -> Result<Value, Value> {
    let mut entries = Vec::new();
    let obj = cx.args.first().unwrap_or_undefined().to_object(cx.scope)?;
    for key in obj.own_keys(cx.scope)? {
        let key = PropertyKey::from_value(cx.scope, key)?;
        let value = obj.get_own_property(key, cx.scope).root(cx.scope)?;
        let entry = Array::from_vec(
            vec![
                PropertyValue::static_default(key.to_value(cx.scope)),
                PropertyValue::static_default(value),
            ],
            cx.scope,
        );
        entries.push(PropertyValue::static_default(Value::object(cx.scope.register(entry))));
    }

    let entries = Array::from_vec(entries, cx.scope);
    Ok(Value::object(cx.scope.register(entries)))
}

pub fn get_prototype_of(cx: CallContext) -> Result<Value, Value> {
    let obj = cx.args.first().unwrap_or_undefined().to_object(cx.scope)?;
    obj.get_prototype(cx.scope)
}

pub fn set_prototype_of(cx: CallContext) -> Result<Value, Value> {
    let obj = cx.args.first().unwrap_or_undefined().to_object(cx.scope)?;
    let target = cx.args.get(1).unwrap_or_undefined();
    obj.set_prototype(target, cx.scope)?;
    Ok(Value::object(obj))
}

pub fn is_prototype_of(cx: CallContext) -> Result<Value, Value> {
    let target_proto = Value::object(cx.this.to_object(cx.scope)?);
    let this_proto = cx.args.first().unwrap_or_undefined();
    if this_proto.type_of(cx.scope) != Typeof::Object {
        return Ok(Value::boolean(false));
    }

    Ok(Value::boolean(
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
    let desc = obj.get_own_property_descriptor(prop, cx.scope).root_err(cx.scope)?;
    Ok(Value::boolean(desc.is_some_and(|val| {
        val.descriptor.contains(PropertyDataDescriptor::ENUMERABLE)
    })))
}

pub fn freeze(cx: CallContext) -> Result<Value, Value> {
    let arg = cx.args.first().unwrap_or_undefined();
    if let ValueKind::Object(o) = arg.unpack() {
        o.set_integrity_level(IntegrityLevel::Frozen, cx.scope)?;
        Ok(Value::object(o))
    } else {
        Ok(arg)
    }
}

pub fn seal(cx: CallContext) -> Result<Value, Value> {
    let arg = cx.args.first().unwrap_or_undefined();
    if let ValueKind::Object(o) = arg.unpack() {
        o.set_integrity_level(IntegrityLevel::Sealed, cx.scope)?;
        Ok(Value::object(o))
    } else {
        Ok(arg)
    }
}
