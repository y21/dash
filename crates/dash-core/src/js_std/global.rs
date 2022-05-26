use crate::vm::value::{
    function::native::CallContext, ops::abstractions::conversions::ValueConversion, Value, ValueContext,
};

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
    let num = trimmed_string
        .parse::<f64>()
        .map(Value::Number)
        .context(cx.scope, "Failed to parse float")?;

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
    let num = i32::from_str_radix(trimmed_string, radix)
        .map(|n| Value::Number(n as f64))
        .context(cx.scope, "Failed to parse number")?;

    Ok(num)
}
