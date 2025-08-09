use crate::localscope::LocalScope;
use crate::throw;
use crate::value::function::native::CallContext;
use crate::value::ops::conversions::ValueConversion;
use crate::value::root_ext::RootErrExt;
use crate::value::{Value, ValueContext};

pub fn abs(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    // 1. Let n be ? ToNumber(x).
    let n = cx.args.first().unwrap_or_undefined().to_number(scope)?;
    // 2. If n is NaN, return NaN.
    // 3. If n is -0𝔽, return +0𝔽.
    // 4. If n is -∞𝔽, return +∞𝔽.
    // 5. If n < +0𝔽, return -n.
    // 6. Return n.
    Ok(Value::number(n.abs()))
}

pub fn acos(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    // 1. Let n be ? ToNumber(x).
    let n = cx.args.first().unwrap_or_undefined().to_number(scope)?;
    // 2. If n is NaN, n > 1𝔽, or n < -1𝔽, return NaN.
    // 3. If n is 1𝔽, return +0𝔽.
    // 4. Return an implementation-approximated Number value representing the result of the inverse cosine of ℝ(n).
    Ok(Value::number(n.acos()))
}

pub fn acosh(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    // 1. Let n be ? ToNumber(x).
    let n = cx.args.first().unwrap_or_undefined().to_number(scope)?;
    // 2. If n is NaN or n is +∞𝔽, return n.
    // 3. If n is 1𝔽, return +0𝔽.
    // 4. If n < 1𝔽, return NaN.
    // 5. Return an implementation-approximated Number value representing the result of the inverse hyperbolic cosine of ℝ(n).
    Ok(Value::number(n.acosh()))
}

pub fn asin(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    // 1. Let n be ? ToNumber(x).
    let n = cx.args.first().unwrap_or_undefined().to_number(scope)?;
    // 2. If n is NaN, n is +0𝔽, or n is -0𝔽, return n.
    // 3. If n > 1𝔽 or n < -1𝔽, return NaN.
    // 4. Return an implementation-approximated Number value representing the result of the inverse sine of ℝ(n).
    Ok(Value::number(n.asin()))
}

pub fn asinh(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    // 1. Let n be ? ToNumber(x).
    let n = cx.args.first().unwrap_or_undefined().to_number(scope)?;
    // 2. If n is NaN, n is +0𝔽, n is -0𝔽, n is +∞𝔽, or n is -∞𝔽, return n.
    // 3. Return an implementation-approximated Number value representing the result of the inverse hyperbolic sine of ℝ(n).
    Ok(Value::number(n.asinh()))
}

pub fn atan(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    // 1. Let n be ? ToNumber(x).
    let n = cx.args.first().unwrap_or_undefined().to_number(scope)?;
    // 2. If n is NaN, n is +0𝔽, or n is -0𝔽, return n.
    // 3. If n is +∞𝔽, return an implementation-approximated Number value representing π / 2.
    // 4. If n is -∞𝔽, return an implementation-approximated Number value representing -π / 2.
    // 5. Return an implementation-approximated Number value representing the result of the inverse tangent of ℝ(n).
    Ok(Value::number(n.atan()))
}

pub fn atanh(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    // 1. Let n be ? ToNumber(x).
    let n = cx.args.first().unwrap_or_undefined().to_number(scope)?;
    // 2. If n is NaN, n is +0𝔽, or n is -0𝔽, return n.
    // 3. If n > 1𝔽 or n < -1𝔽, return NaN.
    // 4. If n is 1𝔽, return +∞𝔽.
    // 5. If n is -1𝔽, return -∞𝔽.
    // 6. Return an implementation-approximated Number value representing the result of the inverse hyperbolic tangent of ℝ(n).
    Ok(Value::number(n.atanh()))
}

pub fn atan2(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    // 1. Let ny be ? ToNumber(y).
    let ny = cx.args.first().unwrap_or_undefined().to_number(scope)?;
    // 2. Let nx be ? ToNumber(x).
    let nx = cx.args.get(1).unwrap_or_undefined().to_number(scope)?;
    // ... steps are a little too long to add here ...
    Ok(Value::number(ny.atan2(nx)))
}

pub fn cbrt(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    // 1. Let n be ? ToNumber(x).
    let n = cx.args.first().unwrap_or_undefined().to_number(scope)?;
    // 2. If n is NaN, n is +0𝔽, n is -0𝔽, n is +∞𝔽, or n is -∞𝔽, return n.
    // 3. Return an implementation-approximated Number value representing the result of the cube root of ℝ(n).
    Ok(Value::number(n.cbrt()))
}

pub fn ceil(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    // 1. Let n be ? ToNumber(x).
    let n = cx.args.first().unwrap_or_undefined().to_number(scope)?;
    // 2. If n is NaN, n is +0𝔽, n is -0𝔽, n is +∞𝔽, or n is -∞𝔽, return n.
    // 3. If n < +0𝔽 and n > -1𝔽, return -0𝔽.
    // 4. If n is an integral Number, return n.
    // 5. Return the smallest (closest to -∞) integral Number value that is not less than n.
    Ok(Value::number(n.ceil()))
}

pub fn clz32(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let n = cx.args.first().unwrap_or_undefined().to_number(scope)? as u32;
    Ok(Value::number(n.leading_zeros() as f64))
}

pub fn cos(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    // 1. Let n be ? ToNumber(x).
    let n = cx.args.first().unwrap_or_undefined().to_number(scope)?;
    // 2. If n is NaN, n is +∞𝔽, or n is -∞𝔽, return NaN.
    // 3. If n is +0𝔽 or n is -0𝔽, return 1𝔽.
    // 4. Return an implementation-approximated Number value representing the result of the cosine of ℝ(n).
    Ok(Value::number(n.cos()))
}

pub fn cosh(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    // 1. Let n be ? ToNumber(x).
    let n = cx.args.first().unwrap_or_undefined().to_number(scope)?;
    // 2. If n is NaN, return NaN.
    // 3. If n is +∞𝔽 or n is -∞𝔽, return +∞𝔽.
    // 4. If n is +0𝔽 or n is -0𝔽, return 1𝔽.
    // 5. Return an implementation-approximated Number value representing the result of the hyperbolic cosine of ℝ(n).
    Ok(Value::number(n.cosh()))
}

pub fn exp(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    // 1. Let n be ? ToNumber(x).
    let n = cx.args.first().unwrap_or_undefined().to_number(scope)?;
    // 2. If n is NaN or n is +∞𝔽, return n.
    // 3. If n is +0𝔽 or n is -0𝔽, return 1𝔽.
    // 4. If n is -∞𝔽, return +0𝔽.
    // 5. Return an implementation-approximated Number value representing the result of the exponential function of ℝ(n).
    Ok(Value::number(n.exp()))
}

pub fn expm1(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    // 1. Let n be ? ToNumber(x).
    let n = cx.args.first().unwrap_or_undefined().to_number(scope)?;
    // 2. If n is NaN, n is +0𝔽, n is -0𝔽, or n is +∞𝔽, return n.
    // 3. If n is -∞𝔽, return -1𝔽.
    // 4. Return an implementation-approximated Number value representing the result of subtracting 1 from the exponential function of ℝ(n).
    Ok(Value::number(n.exp_m1()))
}

pub fn log(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let n = cx.args.first().unwrap_or_undefined().to_number(scope)?;
    Ok(Value::number(n.ln()))
}

pub fn log1p(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let n = cx.args.first().unwrap_or_undefined().to_number(scope)?;
    Ok(Value::number(n.ln_1p()))
}

pub fn log10(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let n = cx.args.first().unwrap_or_undefined().to_number(scope)?;
    Ok(Value::number(n.log10()))
}

pub fn log2(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let n = cx.args.first().unwrap_or_undefined().to_number(scope)?;
    Ok(Value::number(n.log2()))
}

pub fn round(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let n = cx.args.first().unwrap_or_undefined().to_number(scope)?;
    Ok(Value::number(n.round()))
}

pub fn sin(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let n = cx.args.first().unwrap_or_undefined().to_number(scope)?;
    Ok(Value::number(n.sin()))
}

pub fn sinh(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let n = cx.args.first().unwrap_or_undefined().to_number(scope)?;
    Ok(Value::number(n.sinh()))
}

pub fn sqrt(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let n = cx.args.first().unwrap_or_undefined().to_number(scope)?;
    Ok(Value::number(n.sqrt()))
}

pub fn tan(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let n = cx.args.first().unwrap_or_undefined().to_number(scope)?;
    Ok(Value::number(n.tan()))
}

pub fn tanh(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let n = cx.args.first().unwrap_or_undefined().to_number(scope)?;
    Ok(Value::number(n.tanh()))
}

pub fn trunc(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let n = cx.args.first().unwrap_or_undefined().to_number(scope)?;
    Ok(Value::number(n.trunc()))
}

pub fn floor(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    // 1. Let n be ? ToNumber(x).
    let n = cx.args.first().unwrap_or_undefined().to_number(scope)?;

    // 2. If n is NaN, n is +0𝔽, n is -0𝔽, n is +∞𝔽, or n is -∞𝔽, return n.
    if n.is_nan() || n.is_infinite() || n == 0f64 {
        return Ok(Value::number(0f64));
    }

    Ok(Value::number(n.floor()))
}

pub fn random(_: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let num = match scope.params().math_random_callback {
        Some(cb) => cb(scope).root_err(scope)?,
        None => throw!(scope, Error, "Math.random is disabled for this context"),
    };

    Ok(Value::number(num))
}

pub fn pow(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let base = cx.args.first().unwrap_or_undefined().to_number(scope)?;
    let exponent = cx.args.get(1).unwrap_or_undefined().to_number(scope)?;
    Ok(Value::number(base.powf(exponent)))
}

pub fn max(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let mut max = -f64::INFINITY;

    for arg in cx.args.iter() {
        let n = arg.to_number(scope)?;
        if n.is_nan() {
            return Ok(Value::number(f64::NAN));
        }

        if n > max {
            max = n;
        }
    }

    Ok(Value::number(max))
}

pub fn min(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let mut min = f64::INFINITY;

    for arg in cx.args.iter() {
        let n = arg.to_number(scope)?;
        if n.is_nan() {
            return Ok(Value::number(f64::NAN));
        }

        if n < min {
            min = n;
        }
    }

    Ok(Value::number(min))
}
