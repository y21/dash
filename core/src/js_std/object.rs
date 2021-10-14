use std::borrow::Cow;

use super::todo;
use crate::{
    gc::Handle,
    vm::value::{array::Array, function::CallContext, PropertyKey, Value, ValueKind},
};

/// The object constructor
///
/// https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object-constructor
pub fn object_constructor(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo("Object", ctx.vm)
}

/// Implements Object.defineProperty
///
/// https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object.defineproperty
pub fn define_property(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    let mut arguments = ctx.arguments();

    let obj_cell = arguments.next().unwrap();
    let mut obj = unsafe { obj_cell.borrow_mut_unbounded() };
    let prop_cell = arguments.next().unwrap();
    let prop = unsafe { prop_cell.borrow_unbounded() };
    let prop_str = prop.to_property_key(Handle::clone(obj_cell));
    let descriptor_cell = arguments.next().unwrap();

    let value =
        Value::get_property(ctx.vm, descriptor_cell, &PropertyKey::from("value"), None).unwrap();
    obj.set_property(prop_str, value);

    Ok(Handle::clone(&obj_cell))
}

/// Implements Object.getOwnPropertyNames
///
/// https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object.getownpropertynames
pub fn get_own_property_names(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    let obj_cell = ctx.args.first().unwrap();
    let obj = unsafe { obj_cell.borrow_unbounded() };

    let mut keys = Vec::with_capacity(obj.fields.len());
    for key in obj.fields.keys() {
        // We only want to collect string keys, not symbols
        if let PropertyKey::String(s) = key {
            keys.push(ctx.vm.create_js_value(s.to_string()).into_handle(ctx.vm));
        }
    }

    Ok(ctx.vm.create_js_value(Array::new(keys)).into_handle(ctx.vm))
}

/// Implements Object.getOwnPropertySymbols
///
/// https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object.getownpropertysymbols
pub fn get_own_property_symbols(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    let obj_cell = ctx.args.first().unwrap();
    let obj = unsafe { obj_cell.borrow_unbounded() };

    let mut keys = Vec::with_capacity(obj.fields.len());
    for key in obj.fields.keys() {
        if let PropertyKey::Symbol(symbol) = key {
            keys.push(Handle::clone(symbol));
        }
    }

    Ok(ctx.vm.create_js_value(Array::new(keys)).into_handle(ctx.vm))
}

/// Implements Object.getPrototypeOf
///
/// https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object.getprototypeof
pub fn get_prototype_of(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    let obj_cell = ctx.args.first().unwrap();
    let obj = unsafe { obj_cell.borrow_unbounded() };
    Ok(obj
        .proto
        .clone()
        .unwrap_or_else(|| Value::new(ValueKind::Null).into_handle(ctx.vm)))
}

/// Implements Object.prototype.toString
///
/// https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object.prototype.tostring
pub fn to_string(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    let this_cell = ctx
        .receiver
        .as_ref()
        .cloned()
        .unwrap_or_else(|| Value::new(ValueKind::Undefined).into_handle(ctx.vm));

    let this_ref = unsafe { this_cell.borrow_unbounded() };

    let s = match &this_ref.kind {
        // 1. If the this value is undefined, return "[object Undefined]".
        ValueKind::Undefined => Cow::Borrowed("[object Undefined]"),
        // 2. If the this value is null, return "[object Null]".
        ValueKind::Null => Cow::Borrowed("[object Null]"),
        // 3. Let O be ! ToObject(this value).
        _ => {
            if let Some(constructor_cell) = this_ref.constructor.as_ref() {
                let constructor = unsafe { constructor_cell.borrow_unbounded() };
                let constructor_func = constructor.as_function().unwrap();
                Cow::Owned(format!(
                    "[object {}]",
                    constructor_func.name().unwrap_or("Function")
                ))
            } else {
                Cow::Borrowed("Undefined")
            }
        }
    };

    Ok(ctx.vm.create_js_value(s.to_string()).into_handle(ctx.vm))
}
