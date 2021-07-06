use std::{cell::RefCell, rc::Rc};

use crate::vm::{
    value::{object::Object, Value, ValueKind},
    VM,
};

const MAX: f64 = 9007199254740991f64;

// https://tc39.es/ecma262/multipage/abstract-operations.html#sec-lengthofarraylike
pub fn length_of_array_like(vm: &VM, obj: &Rc<RefCell<Value>>) -> Result<f64, Rc<RefCell<Value>>> {
    // ? Get(obj, "length")
    let len_prop_cell = Value::get_property(vm, obj, "length", None);
    let len_prop = len_prop_cell.as_ref().map(|x| x.borrow());

    // ? ToLength(prop)
    let len = to_length(len_prop.as_deref())?;

    // Return
    Ok(len)
}

pub fn to_length(argument: Option<&Value>) -> Result<f64, Rc<RefCell<Value>>> {
    // 1. Let len be ? ToIntegerOrInfinity(argument).
    let len = to_integer_or_infinity(argument)?;

    // 2. If len ≤ 0, return +0𝔽.
    if len <= 0f64 {
        return Ok(0f64);
    }

    // 3. Return 𝔽(min(len, 2^53 - 1)).
    Ok(len.min(MAX))
}

// https://tc39.es/ecma262/multipage/abstract-operations.html#sec-tointegerorinfinity
pub fn to_integer_or_infinity(argument: Option<&Value>) -> Result<f64, Rc<RefCell<Value>>> {
    // 1. Let number be ? ToNumber(argument).
    let number = to_number(argument)?;

    // 2. If number is NaN, +0𝔽, or -0𝔽, return 0.
    if number.is_nan() || number == 0f64 {
        return Ok(0f64);
    }

    // 3. If number is +∞𝔽, return +∞.
    // 4. If number is -∞𝔽, return -∞.
    if number.is_infinite() {
        return Ok(number);
    }

    // 5. Let integer be floor(abs(ℝ(number))).
    let mut integer = number.abs().floor();

    // 6. If number < +0𝔽, set integer to -integer.
    if number < 0f64 {
        integer = -integer;
    }

    // 7. Return integer.
    Ok(integer)
}

// https://tc39.es/ecma262/multipage/abstract-operations.html#sec-tonumber
pub fn to_number(argument: Option<&Value>) -> Result<f64, Rc<RefCell<Value>>> {
    match argument.as_ref().map(|a| &a.kind) {
        Some(ValueKind::Undefined) => Ok(f64::NAN),
        Some(ValueKind::Null) => Ok(0f64),
        Some(ValueKind::Bool(b)) => Ok(*b as u8 as f64),
        Some(ValueKind::Number(n)) => Ok(*n),
        Some(ValueKind::Object(o)) => match &**o {
            Object::String(s) => Ok(to_number_from_string(s)),
            _ => todo!(),
        },
        None => Ok(f64::NAN),
        _ => todo!(),
    }
}

// https://tc39.es/ecma262/multipage/abstract-operations.html#sec-tonumber-applied-to-the-string-type
pub fn to_number_from_string(argument: &str) -> f64 {
    argument.parse::<f64>().unwrap_or_else(|_| f64::NAN)
}
