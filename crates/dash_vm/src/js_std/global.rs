use crate::value::function::native::CallContext;
use crate::value::ops::conversions::ValueConversion;
use crate::value::{Value, ValueContext};

#[rustfmt::skip]
pub fn is_nan(cx: CallContext) -> Result<Value, Value> {
    // 1. Let num be ? ToNumber(number).
    let num = cx.args.first().unwrap_or_undefined().to_number(cx.scope)?;
    // 2. If num is NaN, return true.
    // 3. Otherwise, return false.
    Ok(Value::Boolean(num.is_nan()))
}

pub fn log(cx: CallContext) -> Result<Value, Value> {
    for arg in cx.args {
        let tstr = arg.to_string(cx.scope)?;
        println!("{tstr} ");
    }

    Ok(Value::undefined())
}

pub fn is_finite(cx: CallContext) -> Result<Value, Value> {
    // 1. Let num be ? ToNumber(number).
    let num = cx.args.first().unwrap_or_undefined().to_number(cx.scope)?;
    // 2. If num is NaN, +∞, or -∞, return false.
    // 3. Otherwise, return true.
    Ok(Value::Boolean(num.is_finite()))
}

pub fn parse_float(cx: CallContext) -> Result<Value, Value> {
    // 1. Let inputString be ? ToString(string).
    let input_string = cx.args.first().unwrap_or_undefined().to_string(cx.scope)?;
    // 2. Let trimmedString be ! TrimString(inputString, start).
    let trimmed_string = input_string.trim();

    // TODO: follow spec
    let num = Value::number(trimmed_string.parse().unwrap_or(f64::NAN));

    Ok(num)
}

pub fn parse_int(cx: CallContext) -> Result<Value, Value> {
    let input_string = cx.args.first().unwrap_or_undefined().to_string(cx.scope)?;
    let trimmed_string = input_string.trim();
    let radix = cx
        .args
        .get(1)
        .cloned()
        .map(|v| v.to_number(cx.scope))
        .transpose()?
        .map(|r| r as u32)
        .unwrap_or(10);

    // TODO: follow spec
    let num = Value::number(
        i32::from_str_radix(trimmed_string, radix)
            .map(|n| n as f64)
            .unwrap_or(f64::NAN),
    );

    Ok(num)
}
