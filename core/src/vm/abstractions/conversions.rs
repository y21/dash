use std::{borrow::Cow, cell::RefCell, rc::Rc};

use crate::{
    js_std,
    vm::{
        value::{function::CallResult, object::Object, Value, ValueKind},
        VM,
    },
};

// https://tc39.es/ecma262/multipage/abstract-operations.html#sec-tostring
pub fn to_string(
    vm: &VM,
    argument_cell: Option<&Rc<RefCell<Value>>>,
) -> Result<CallResult, Rc<RefCell<Value>>> {
    let argument = argument_cell.map(|x| x.borrow());
    let ready_string = match argument.as_ref().map(|x| &x.kind) {
        Some(ValueKind::Undefined) | None => Cow::Borrowed("undefined"),
        Some(ValueKind::Null) => Cow::Borrowed("null"),
        Some(ValueKind::Bool(b)) => Cow::Borrowed(if *b { "true" } else { "false" }),
        Some(ValueKind::Number(n)) => number_to_string(*n),
        Some(ValueKind::Object(o)) => match &**o {
            Object::String(s) => Cow::Borrowed(&**s),
            _ => {
                // 1. Let primValue be ? ToPrimitive(argument, string).
                let prim_value = to_primitive(
                    vm,
                    argument.as_ref().unwrap(),
                    argument_cell.unwrap(),
                    Some("string"),
                )?;

                // 2. Return ? ToString(primValue).
                return Ok(match prim_value {
                    CallResult::Ready(r) => return to_string(vm, Some(&r)),
                    CallResult::UserFunction(func, args) => CallResult::UserFunction(func, args),
                });
            }
        },
        Some(ValueKind::Constant(_)) => unreachable!(),
    };

    Ok(CallResult::Ready(
        vm.create_js_value(String::from(ready_string)).into(),
    ))
}

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

// https://tc39.es/ecma262/multipage/abstract-operations.html#sec-toprimitive
pub fn to_primitive(
    vm: &VM,
    input: &Value,
    input_cell: &Rc<RefCell<Value>>,
    preferred_type: Option<&str>,
) -> Result<CallResult, Rc<RefCell<Value>>> {
    // 2. If Type(input) is Object, then
    if input.as_object().is_some() {
        // Let exoticToPrim be ? GetMethod(input, @@toPrimitive).
        // ^ todo: we do not have symbols yet

        // c. If preferredType is not present, let preferredType be number.
        let preferred_type = preferred_type.unwrap_or("number");

        // d. Return ? OrdinaryToPrimitive(input, preferredType).
        return ordinary_to_primitive(vm, input_cell, preferred_type);
    }

    Ok(CallResult::Ready(Rc::clone(input_cell)))
}

// TODO: use enum for hint?
pub fn ordinary_to_primitive(
    vm: &VM,
    obj: &Rc<RefCell<Value>>,
    hint: &str,
) -> Result<CallResult, Rc<RefCell<Value>>> {
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
            let method_ref = method.borrow();

            // b. If IsCallable(method) is true, then
            if method_ref.as_function().is_some() {
                return Ok(CallResult::UserFunction(Rc::clone(&method), Vec::new()));
            }
        }
    }

    Err(js_std::error::create_error(
        "Cannot convert to primitive value".into(),
        vm,
    ))
}
