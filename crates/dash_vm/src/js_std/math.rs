use crate::throw;
use crate::value::function::native::CallContext;
use crate::value::ops::abstractions::conversions::ValueConversion;
use crate::value::Value;
use crate::value::ValueContext;

pub fn abs(cx: CallContext) -> Result<Value, Value> {
    // 1. Let n be ? ToNumber(x).
    let n = cx.args.get(0).unwrap_or_undefined().to_number(cx.scope)?;
    // 2. If n is NaN, return NaN.
    // 3. If n is -0𝔽, return +0𝔽.
    // 4. If n is -∞𝔽, return +∞𝔽.
    // 5. If n < +0𝔽, return -n.
    // 6. Return n.
    Ok(Value::Number(n.abs()))
}

pub fn acos(cx: CallContext) -> Result<Value, Value> {
    // 1. Let n be ? ToNumber(x).
    let n = cx.args.get(0).unwrap_or_undefined().to_number(cx.scope)?;
    // 2. If n is NaN, n > 1𝔽, or n < -1𝔽, return NaN.
    // 3. If n is 1𝔽, return +0𝔽.
    // 4. Return an implementation-approximated Number value representing the result of the inverse cosine of ℝ(n).
    Ok(Value::Number(n.acos()))
}

pub fn acosh(cx: CallContext) -> Result<Value, Value> {
    // 1. Let n be ? ToNumber(x).
    let n = cx.args.get(0).unwrap_or_undefined().to_number(cx.scope)?;
    // 2. If n is NaN or n is +∞𝔽, return n.
    // 3. If n is 1𝔽, return +0𝔽.
    // 4. If n < 1𝔽, return NaN.
    // 5. Return an implementation-approximated Number value representing the result of the inverse hyperbolic cosine of ℝ(n).
    Ok(Value::Number(n.acosh()))
}

pub fn asin(cx: CallContext) -> Result<Value, Value> {
    // 1. Let n be ? ToNumber(x).
    let n = cx.args.get(0).unwrap_or_undefined().to_number(cx.scope)?;
    // 2. If n is NaN, n is +0𝔽, or n is -0𝔽, return n.
    // 3. If n > 1𝔽 or n < -1𝔽, return NaN.
    // 4. Return an implementation-approximated Number value representing the result of the inverse sine of ℝ(n).
    Ok(Value::Number(n.asin()))
}

pub fn asinh(cx: CallContext) -> Result<Value, Value> {
    // 1. Let n be ? ToNumber(x).
    let n = cx.args.get(0).unwrap_or_undefined().to_number(cx.scope)?;
    // 2. If n is NaN, n is +0𝔽, n is -0𝔽, n is +∞𝔽, or n is -∞𝔽, return n.
    // 3. Return an implementation-approximated Number value representing the result of the inverse hyperbolic sine of ℝ(n).
    Ok(Value::Number(n.asinh()))
}

pub fn atan(cx: CallContext) -> Result<Value, Value> {
    // 1. Let n be ? ToNumber(x).
    let n = cx.args.get(0).unwrap_or_undefined().to_number(cx.scope)?;
    // 2. If n is NaN, n is +0𝔽, or n is -0𝔽, return n.
    // 3. If n is +∞𝔽, return an implementation-approximated Number value representing π / 2.
    // 4. If n is -∞𝔽, return an implementation-approximated Number value representing -π / 2.
    // 5. Return an implementation-approximated Number value representing the result of the inverse tangent of ℝ(n).
    Ok(Value::Number(n.atan()))
}

pub fn atanh(cx: CallContext) -> Result<Value, Value> {
    // 1. Let n be ? ToNumber(x).
    let n = cx.args.get(0).unwrap_or_undefined().to_number(cx.scope)?;
    // 2. If n is NaN, n is +0𝔽, or n is -0𝔽, return n.
    // 3. If n > 1𝔽 or n < -1𝔽, return NaN.
    // 4. If n is 1𝔽, return +∞𝔽.
    // 5. If n is -1𝔽, return -∞𝔽.
    // 6. Return an implementation-approximated Number value representing the result of the inverse hyperbolic tangent of ℝ(n).
    Ok(Value::Number(n.atanh()))
}

pub fn atan2(cx: CallContext) -> Result<Value, Value> {
    // 1. Let ny be ? ToNumber(y).
    let ny = cx.args.get(0).unwrap_or_undefined().to_number(cx.scope)?;
    // 2. Let nx be ? ToNumber(x).
    let nx = cx.args.get(1).unwrap_or_undefined().to_number(cx.scope)?;
    // ... steps are a little too long to add here ...
    Ok(Value::Number(ny.atan2(nx)))
}

pub fn cbrt(cx: CallContext) -> Result<Value, Value> {
    // 1. Let n be ? ToNumber(x).
    let n = cx.args.get(0).unwrap_or_undefined().to_number(cx.scope)?;
    // 2. If n is NaN, n is +0𝔽, n is -0𝔽, n is +∞𝔽, or n is -∞𝔽, return n.
    // 3. Return an implementation-approximated Number value representing the result of the cube root of ℝ(n).
    Ok(Value::Number(n.cbrt()))
}

pub fn ceil(cx: CallContext) -> Result<Value, Value> {
    // 1. Let n be ? ToNumber(x).
    let n = cx.args.get(0).unwrap_or_undefined().to_number(cx.scope)?;
    // 2. If n is NaN, n is +0𝔽, n is -0𝔽, n is +∞𝔽, or n is -∞𝔽, return n.
    // 3. If n < +0𝔽 and n > -1𝔽, return -0𝔽.
    // 4. If n is an integral Number, return n.
    // 5. Return the smallest (closest to -∞) integral Number value that is not less than n.
    Ok(Value::Number(n.ceil()))
}

pub fn clz32(cx: CallContext) -> Result<Value, Value> {
    let n = cx.args.get(0).unwrap_or_undefined().to_number(cx.scope)? as u32;
    Ok(Value::Number(n.leading_zeros() as f64))
}

pub fn cos(cx: CallContext) -> Result<Value, Value> {
    // 1. Let n be ? ToNumber(x).
    let n = cx.args.get(0).unwrap_or_undefined().to_number(cx.scope)?;
    // 2. If n is NaN, n is +∞𝔽, or n is -∞𝔽, return NaN.
    // 3. If n is +0𝔽 or n is -0𝔽, return 1𝔽.
    // 4. Return an implementation-approximated Number value representing the result of the cosine of ℝ(n).
    Ok(Value::Number(n.cos()))
}

pub fn cosh(cx: CallContext) -> Result<Value, Value> {
    // 1. Let n be ? ToNumber(x).
    let n = cx.args.get(0).unwrap_or_undefined().to_number(cx.scope)?;
    // 2. If n is NaN, return NaN.
    // 3. If n is +∞𝔽 or n is -∞𝔽, return +∞𝔽.
    // 4. If n is +0𝔽 or n is -0𝔽, return 1𝔽.
    // 5. Return an implementation-approximated Number value representing the result of the hyperbolic cosine of ℝ(n).
    Ok(Value::Number(n.cosh()))
}

pub fn exp(cx: CallContext) -> Result<Value, Value> {
    // 1. Let n be ? ToNumber(x).
    let n = cx.args.get(0).unwrap_or_undefined().to_number(cx.scope)?;
    // 2. If n is NaN or n is +∞𝔽, return n.
    // 3. If n is +0𝔽 or n is -0𝔽, return 1𝔽.
    // 4. If n is -∞𝔽, return +0𝔽.
    // 5. Return an implementation-approximated Number value representing the result of the exponential function of ℝ(n).
    Ok(Value::Number(n.exp()))
}

pub fn expm1(cx: CallContext) -> Result<Value, Value> {
    // 1. Let n be ? ToNumber(x).
    let n = cx.args.get(0).unwrap_or_undefined().to_number(cx.scope)?;
    // 2. If n is NaN, n is +0𝔽, n is -0𝔽, or n is +∞𝔽, return n.
    // 3. If n is -∞𝔽, return -1𝔽.
    // 4. Return an implementation-approximated Number value representing the result of subtracting 1 from the exponential function of ℝ(n).
    Ok(Value::Number(n.exp_m1()))
}

pub fn log(cx: CallContext) -> Result<Value, Value> {
    let n = cx.args.get(0).unwrap_or_undefined().to_number(cx.scope)?;
    Ok(Value::Number(n.ln()))
}

pub fn log1p(cx: CallContext) -> Result<Value, Value> {
    let n = cx.args.get(0).unwrap_or_undefined().to_number(cx.scope)?;
    Ok(Value::Number(n.ln_1p()))
}

pub fn log10(cx: CallContext) -> Result<Value, Value> {
    let n = cx.args.get(0).unwrap_or_undefined().to_number(cx.scope)?;
    Ok(Value::Number(n.log10()))
}

pub fn log2(cx: CallContext) -> Result<Value, Value> {
    let n = cx.args.get(0).unwrap_or_undefined().to_number(cx.scope)?;
    Ok(Value::Number(n.log2()))
}

pub fn round(cx: CallContext) -> Result<Value, Value> {
    let n = cx.args.get(0).unwrap_or_undefined().to_number(cx.scope)?;
    Ok(Value::Number(n.round()))
}

pub fn sin(cx: CallContext) -> Result<Value, Value> {
    let n = cx.args.get(0).unwrap_or_undefined().to_number(cx.scope)?;
    Ok(Value::Number(n.sin()))
}

pub fn sinh(cx: CallContext) -> Result<Value, Value> {
    let n = cx.args.get(0).unwrap_or_undefined().to_number(cx.scope)?;
    Ok(Value::Number(n.sinh()))
}

pub fn sqrt(cx: CallContext) -> Result<Value, Value> {
    let n = cx.args.get(0).unwrap_or_undefined().to_number(cx.scope)?;
    Ok(Value::Number(n.sqrt()))
}

pub fn tan(cx: CallContext) -> Result<Value, Value> {
    let n = cx.args.get(0).unwrap_or_undefined().to_number(cx.scope)?;
    Ok(Value::Number(n.tan()))
}

pub fn tanh(cx: CallContext) -> Result<Value, Value> {
    let n = cx.args.get(0).unwrap_or_undefined().to_number(cx.scope)?;
    Ok(Value::Number(n.tanh()))
}

pub fn trunc(cx: CallContext) -> Result<Value, Value> {
    let n = cx.args.get(0).unwrap_or_undefined().to_number(cx.scope)?;
    Ok(Value::Number(n.trunc()))
}

pub fn floor(cx: CallContext) -> Result<Value, Value> {
    // 1. Let n be ? ToNumber(x).
    let n = cx.args.get(0).unwrap_or_undefined().to_number(cx.scope)?;

    // 2. If n is NaN, n is +0𝔽, n is -0𝔽, n is +∞𝔽, or n is -∞𝔽, return n.
    if n.is_nan() || n.is_infinite() || n == 0f64 {
        return Ok(Value::Number(0f64));
    }

    Ok(Value::Number(n.floor()))
}

pub fn random(mut cx: CallContext) -> Result<Value, Value> {
    let num = match cx.scope.params().math_random_callback() {
        Some(cb) => cb(&mut cx.scope)?,
        None => throw!(cx.scope, "Math.random is disabled for this context"),
    };

    Ok(Value::Number(num))
}
