use std::borrow::Cow;

use crate::{
    gc::Handle,
    js_std,
    vm::{
        value::{
            object::{ExoticObject, Object},
            Value, ValueKind,
        },
        VM,
    },
};

/// Implements the abstract operation ToString
///
// https://tc39.es/ecma262/multipage/abstract-operations.html#sec-tostring
pub fn to_string(
    vm: &mut VM,
    argument_cell: Option<&Handle<Value>>,
) -> Result<Handle<Value>, Handle<Value>> {
    let argument = argument_cell.map(|x| unsafe { x.borrow_unbounded() });
    let ready_string = match argument.as_ref().map(|x| &x.kind) {
        Some(ValueKind::Undefined) | None => Cow::Borrowed("undefined"),
        Some(ValueKind::Null) => Cow::Borrowed("null"),
        Some(ValueKind::Bool(b)) => Cow::Borrowed(if *b { "true" } else { "false" }),
        Some(ValueKind::Number(n)) => number_to_string(*n),
        Some(ValueKind::Object(o)) => match &**o {
            Object::Exotic(ExoticObject::String(s)) => Cow::Borrowed(&**s),
            _ => {
                // 1. Let primValue be ? ToPrimitive(argument, string).
                let prim_value = to_primitive(
                    vm,
                    argument.as_ref().unwrap(),
                    argument_cell.unwrap(),
                    Some("string"),
                )?;

                // 2. Return ? ToString(primValue).
                return to_string(vm, Some(&prim_value));
            }
        },
    };

    Ok(vm
        .create_js_value(String::from(ready_string))
        .into_handle(vm))
}

/// Implements the abstract operation Number::ToString
///
// https://tc39.es/ecma262/multipage/ecmascript-data-types-and-values.html#sec-numeric-types-number-tostring
pub fn number_to_string(x: f64) -> Cow<'static, str> {
    if x.is_nan() {
        Cow::Borrowed("NaN")
    } else if x.is_infinite() {
        Cow::Borrowed("Infinity")
    } else {
        Cow::Owned(x.to_string())
    }
}

/// Implements the abstract operation ToPrimitive
///
// https://tc39.es/ecma262/multipage/abstract-operations.html#sec-toprimitive
pub fn to_primitive(
    vm: &mut VM,
    input: &Value,
    input_cell: &Handle<Value>,
    preferred_type: Option<&str>,
) -> Result<Handle<Value>, Handle<Value>> {
    // 2. If Type(input) is Object, then
    if input.as_object().is_some() {
        // Let exoticToPrim be ? GetMethod(input, @@toPrimitive).
        // ^ todo: we do not have symbols yet

        // c. If preferredType is not present, let preferredType be number.
        let preferred_type = preferred_type.unwrap_or("number");

        // d. Return ? OrdinaryToPrimitive(input, preferredType).
        return ordinary_to_primitive(vm, input_cell, preferred_type);
    }

    Ok(Handle::clone(input_cell))
}

/// Implements the abstract operation OrdinaryToPrimitive
///
/// https://tc39.es/ecma262/multipage/abstract-operations.html#sec-ordinarytoprimitive
pub fn ordinary_to_primitive(
    vm: &mut VM,
    obj: &Handle<Value>,
    hint: &str,
) -> Result<Handle<Value>, Handle<Value>> {
    // 3. If hint is string, then
    let method_names = if hint == "string" {
        // a. Let methodNames be « "toString", "valueOf" ».
        ["toString", "valueOf"]
    } else {
        // 4. Else,
        // a. Let methodNames be « "valueOf", "toString" ».
        ["valueOf", "toString"]
    };

    // 5. For each element name of methodNames, do
    for name in method_names {
        // a. Let method be ? Get(O, name).
        if let Some(method) = Value::get_property(vm, obj, name, None) {
            let method_ref = unsafe { method.borrow_unbounded() };

            // b. If IsCallable(method) is true, then
            if method_ref.as_function().is_some() {
                return Value::call(&method, Vec::new(), vm);
            }
        }
    }

    Err(js_std::error::create_error(
        "Cannot convert to primitive value".into(),
        vm,
    ))
}
