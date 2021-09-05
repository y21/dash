use crate::{gc::Handle, vm::{VM, abstractions::conversions::to_string, value::{Value, ValueKind, object::{ExoticObject, Object}}}};

const MAX: f64 = 9007199254740991f64;

/// Implements the abstract operation LengthOfArrayLike
///
// https://tc39.es/ecma262/multipage/abstract-operations.html#sec-lengthofarraylike
pub fn length_of_array_like(vm: &VM, obj: &Handle<Value>) -> Result<f64, Handle<Value>> {
    // ? Get(obj, "length")
    let len_prop_cell = Value::get_property(vm, obj, "length", None);
    let len_prop = len_prop_cell
        .as_ref()
        .map(|x| unsafe { x.borrow_unbounded() });

    // ? ToLength(prop)
    let len = to_length(len_prop.as_ref().map(|c| &***c))?;

    // Return
    Ok(len)
}

/// Implements the abstract operation ToLength
///
/// https://tc39.es/ecma262/multipage/abstract-operations.html#sec-tolength
pub fn to_length(argument: Option<&Value>) -> Result<f64, Handle<Value>> {
    // 1. Let len be ? ToIntegerOrInfinity(argument).
    let len = to_integer_or_infinity(argument)?;

    // 2. If len ≤ 0, return +0𝔽.
    if len <= 0f64 {
        return Ok(0f64);
    }

    // 3. Return 𝔽(min(len, 2^53 - 1)).
    Ok(len.min(MAX))
}

/// Implements the abstract operation ToIntegerOrInfinity
///
// https://tc39.es/ecma262/multipage/abstract-operations.html#sec-tointegerorinfinity
pub fn to_integer_or_infinity(argument: Option<&Value>) -> Result<f64, Handle<Value>> {
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

/// Implements the abstract operation ToNumber
///
// https://tc39.es/ecma262/multipage/abstract-operations.html#sec-tonumber
pub fn to_number(argument: Option<&Value>) -> Result<f64, Handle<Value>> {
    match argument.as_ref().map(|a| &a.kind) {
        Some(ValueKind::Undefined) => Ok(f64::NAN),
        Some(ValueKind::Null) => Ok(0f64),
        Some(ValueKind::Bool(b)) => Ok(*b as u8 as f64),
        Some(ValueKind::Number(n)) => Ok(*n),
        Some(ValueKind::Object(o)) => match &**o {
            Object::Exotic(ExoticObject::String(s)) => Ok(to_number_from_string(s)),
            _ => todo!(),
        },
        None => Ok(f64::NAN),
    }
}

/// Implements the abstract operation ToNumberFromString
///
// https://tc39.es/ecma262/multipage/abstract-operations.html#sec-tonumber-applied-to-the-string-type
pub fn to_number_from_string(argument: &str) -> f64 {
    argument.parse::<f64>().unwrap_or_else(|_| f64::NAN)
}

/// Implements indexOf
pub fn index_of(
    vm: &mut VM,
    hay: Option<&Handle<Value>>,
    needle: Option<&Handle<Value>>,
    position: Option<&Handle<Value>>,
) -> Result<f64, Handle<Value>> {
    // Let S be ? ToString(O).
    let this = to_string(vm, hay)?;
    let this_ref = unsafe { this.borrow_unbounded() };
    let this_s = this_ref.as_string().unwrap();

    // 5. Let searchStr be ? ToString(searchString).
    let search_str = to_string(vm, needle)?;
    let search_str_ref = unsafe { search_str.borrow_unbounded() };
    let search_s = search_str_ref.as_string().unwrap();

    // 6. Let pos be ? ToIntegerOrInfinity(position).
    let pos = position.as_ref().map(|x| unsafe { x.borrow_unbounded() });
    let pos = to_integer_or_infinity(pos.as_ref().map(|x| &***x))? as usize;

    // 8. Let len be the length of S.
    let len = this_s.len();

    // 9. Let start be the result of clamping pos between 0 and len.
    let start = pos.clamp(0, len);

    let sub = String::from_utf8_lossy(&this_s.as_bytes()[start..len]);

    let idx = sub.find(search_s).map(|x| x as f64).unwrap_or(-1f64);

    Ok(idx)
}
