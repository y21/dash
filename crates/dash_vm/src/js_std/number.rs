use crate::throw;
use crate::util::format_f64;
use crate::value::function::native::CallContext;
use crate::value::ops::abstractions::conversions::ValueConversion;
use crate::value::primitive::Number;
use crate::value::primitive::MAX_SAFE_INTEGERF;
use crate::value::Value;
use crate::value::ValueContext;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let value = cx.args.get(0).unwrap_or_undefined().to_number(cx.scope)?;
    Ok(Value::number(value))
}

pub fn to_string(cx: CallContext) -> Result<Value, Value> {
    let radix = cx
        .args
        .first()
        .cloned()
        .map(|v| v.to_number(cx.scope))
        .transpose()?
        .map(|n| n as u8)
        .unwrap_or(10);

    let Value::Number(Number(num)) = cx.this else {
        throw!(cx.scope, TypeError, "Number.prototype.toString called on non-number")
    };

    let re = match radix {
        2 => format!("{:b}", num as u64),
        10 => format_f64(num),
        16 => format!("{:x}", num as u64),
        _ => throw!(cx.scope, RangeError, "Invalid radix: {}", radix),
    };

    Ok(Value::String(re.into()))
}

pub fn is_finite(cx: CallContext) -> Result<Value, Value> {
    let num = match cx.args.first() {
        Some(Value::Number(Number(n))) => n,
        _ => return Ok(Value::Boolean(false)),
    };

    Ok(Value::Boolean(num.is_finite()))
}

pub fn is_nan(cx: CallContext) -> Result<Value, Value> {
    let num = match cx.args.first() {
        Some(Value::Number(Number(n))) => n,
        _ => return Ok(Value::Boolean(false)),
    };

    Ok(Value::Boolean(num.is_nan()))
}

pub fn is_safe_integer(cx: CallContext) -> Result<Value, Value> {
    let num = match cx.args.first() {
        Some(Value::Number(Number(n))) => n,
        _ => return Ok(Value::Boolean(false)),
    };

    Ok(Value::Boolean(*num < MAX_SAFE_INTEGERF))
}

pub fn to_fixed(cx: CallContext) -> Result<Value, Value> {
    let num = cx.this.to_number(cx.scope)?;
    let decimals = cx
        .args
        .first()
        .cloned()
        .map(|v| v.to_number(cx.scope))
        .transpose()?
        .map(|n| n as usize)
        .unwrap_or(0);

    let re = format!("{num:.decimals$}");

    Ok(Value::String(re.into()))
}
