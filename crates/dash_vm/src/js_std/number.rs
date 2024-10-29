use crate::throw;
use crate::util::intern_f64;
use crate::value::function::native::CallContext;
use crate::value::object::Object;
use crate::value::ops::conversions::ValueConversion;
use crate::value::primitive::{Number, MAX_SAFE_INTEGER, MIN_SAFE_INTEGER};
use crate::value::{boxed, Unpack, Value, ValueContext, ValueKind};

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let value = cx.args.first().unwrap_or_undefined().to_number(cx.scope)?;
    if cx.is_constructor_call {
        let value = boxed::Number::new(cx.scope, value);
        Ok(Value::object(cx.scope.register(value)))
    } else {
        Ok(Value::number(value))
    }
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

    let Some(num) = cx
        .this
        .internal_slots(&cx.scope)
        .and_then(|slots| slots.number_value(cx.scope))
    else {
        throw!(
            cx.scope,
            TypeError,
            "Number.prototype.toString called on non-number value"
        )
    };

    let re = match radix {
        2 => cx.scope.intern(format!("{:b}", num as u64).as_ref()),
        10 => intern_f64(cx.scope, num),
        16 => cx.scope.intern(format!("{:x}", num as u64)),
        _ => throw!(cx.scope, RangeError, "Invalid radix: {}", radix),
    };

    Ok(Value::string(re.into()))
}

pub fn value_of(cx: CallContext) -> Result<Value, Value> {
    if let Some(num) = cx
        .this
        .internal_slots(&cx.scope)
        .and_then(|slots| slots.number_value(cx.scope))
    {
        Ok(Value::number(num))
    } else {
        throw!(
            cx.scope,
            TypeError,
            "Number.prototype.valueOf called on non-number value"
        )
    }
}

pub fn is_finite(cx: CallContext) -> Result<Value, Value> {
    let num = match cx.args.first().unpack() {
        Some(ValueKind::Number(Number(n))) => n,
        _ => return Ok(Value::boolean(false)),
    };

    Ok(Value::boolean(num.is_finite()))
}

pub fn is_nan(cx: CallContext) -> Result<Value, Value> {
    let num = match cx.args.first().unpack() {
        Some(ValueKind::Number(Number(n))) => n,
        _ => return Ok(Value::boolean(false)),
    };

    Ok(Value::boolean(num.is_nan()))
}

fn as_integer(f: f64) -> Option<i64> {
    (f.trunc() == f && f.is_finite()).then_some(f as i64)
}

pub fn is_safe_integer(cx: CallContext) -> Result<Value, Value> {
    let num = match cx.args.first().unpack() {
        Some(ValueKind::Number(Number(n))) => n,
        _ => return Ok(Value::boolean(false)),
    };
    let is_safe = as_integer(num).is_some_and(|n| (MIN_SAFE_INTEGER..=MAX_SAFE_INTEGER).contains(&n));

    Ok(Value::boolean(is_safe))
}

pub fn is_integer(cx: CallContext) -> Result<Value, Value> {
    let num = match cx.args.first().unpack() {
        Some(ValueKind::Number(Number(n))) => n,
        _ => return Ok(Value::boolean(false)),
    };
    Ok(Value::boolean(as_integer(num).is_some()))
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

    if decimals > 100 {
        throw!(
            cx.scope,
            RangeError,
            "toFixed() fractional digits must be between 0 and 100 inclusive"
        )
    };

    let re = format!("{num:.decimals$}");

    Ok(Value::string(cx.scope.intern(re.as_ref()).into()))
}
