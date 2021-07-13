use std::borrow::Cow;

use crate::{
    gc::Handle,
    vm::value::{
        array::Array,
        function::{CallContext, CallResult},
        Value, ValueKind,
    },
};

/// The object constructor
///
/// https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object-constructor
pub fn object_constructor(_args: CallContext) -> Result<CallResult, Handle<Value>> {
    todo!()
}

/// Implements Object.defineProperty
///
/// https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object.defineproperty
pub fn define_property(ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    let mut arguments = ctx.arguments();

    let obj_cell = arguments.next().unwrap();
    let mut obj = obj_cell.borrow_mut();
    let prop_cell = arguments.next().unwrap();
    let prop = prop_cell.borrow();
    let prop_str = prop.to_string();
    let descriptor_cell = arguments.next().unwrap();

    let value = Value::get_property(ctx.vm, descriptor_cell, "value", None).unwrap();
    obj.set_property(&*prop_str, value);

    Ok(CallResult::Ready(Handle::clone(&obj_cell)))
}

/// Implements Object.getOwnPropertyNames
///
/// https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object.getownpropertynames
pub fn get_own_property_names(ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    let obj_cell = ctx.args.first().unwrap();
    let obj = obj_cell.borrow();

    let mut keys = Vec::with_capacity(obj.fields.len());
    for key in obj.fields.keys() {
        let key: &str = &*key;
        keys.push(
            ctx.vm
                .create_js_value(String::from(key))
                .into_handle(ctx.vm),
        );
    }

    Ok(CallResult::Ready(
        ctx.vm.create_js_value(Array::new(keys)).into_handle(ctx.vm),
    ))
}

/// Implements Object.getPrototypeOf
///
/// https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object.getprototypeof
pub fn get_prototype_of(ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    let obj_cell = ctx.args.first().unwrap();
    let obj = obj_cell.borrow();
    Ok(CallResult::Ready(obj.proto.clone().unwrap_or_else(|| {
        Value::new(ValueKind::Null).into_handle(ctx.vm)
    })))
}

/// Implements Object.prototype.toString
///
/// https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object.prototype.tostring
pub fn to_string(ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    let this_cell = ctx
        .receiver
        .as_ref()
        .cloned()
        .unwrap_or_else(|| Value::new(ValueKind::Undefined).into_handle(ctx.vm));

    let this_ref = this_cell.borrow();

    let s = match &this_ref.kind {
        // 1. If the this value is undefined, return "[object Undefined]".
        ValueKind::Undefined => Cow::Borrowed("[object Undefined]"),
        // 2. If the this value is null, return "[object Null]".
        ValueKind::Null => Cow::Borrowed("[object Null]"),
        // 3. Let O be ! ToObject(this value).
        _ => {
            if let Some(constructor_cell) = this_ref.constructor.as_ref() {
                let constructor = constructor_cell.borrow();
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

    Ok(CallResult::Ready(
        ctx.vm.create_js_value(s.to_string()).into_handle(ctx.vm),
    ))
}
