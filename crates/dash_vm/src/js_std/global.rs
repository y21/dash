use dash_middle::parser::error::IntoFormattableErrors;

use crate::eval::EvalError;
use crate::throw;
use crate::value::function::native::CallContext;
use crate::value::ops::conversions::ValueConversion;
use crate::value::{Root, Unpack, Value, ValueContext, ValueKind};

pub fn is_nan(cx: CallContext) -> Result<Value, Value> {
    // 1. Let num be ? ToNumber(number).
    let num = cx.args.first().unwrap_or_undefined().to_number(cx.scope)?;
    // 2. If num is NaN, return true.
    // 3. Otherwise, return false.
    Ok(Value::boolean(num.is_nan()))
}

pub fn eval(cx: CallContext) -> Result<Value, Value> {
    let source = {
        let value = cx.args.first().unwrap_or_undefined();
        match value.unpack() {
            ValueKind::String(s) => s.res(cx.scope).to_owned(),
            _ => return Ok(value),
        }
    };

    match cx.scope.eval(&source, Default::default()) {
        Ok(v) => Ok(v.root(cx.scope)),
        Err(EvalError::Exception(ex)) => Err(ex.root(cx.scope)),
        Err(EvalError::Middle(err)) => throw!(cx.scope, SyntaxError, "{}", err.formattable(&source, true)),
    }
}

pub fn log(cx: CallContext) -> Result<Value, Value> {
    for arg in cx.args {
        let tstr = arg.to_js_string(cx.scope)?;
        println!("{} ", tstr.res(cx.scope));
    }

    Ok(Value::undefined())
}

pub fn is_finite(cx: CallContext) -> Result<Value, Value> {
    // 1. Let num be ? ToNumber(number).
    let num = cx.args.first().unwrap_or_undefined().to_number(cx.scope)?;
    // 2. If num is NaN, +∞, or -∞, return false.
    // 3. Otherwise, return true.
    Ok(Value::boolean(num.is_finite()))
}

pub fn parse_float(cx: CallContext) -> Result<Value, Value> {
    // 1. Let inputString be ? ToString(string).
    let input_string = cx.args.first().unwrap_or_undefined().to_js_string(cx.scope)?;
    // 2. Let trimmedString be ! TrimString(inputString, start).
    let trimmed_string = input_string.res(cx.scope).trim();

    // TODO: follow spec
    let num = Value::number(trimmed_string.parse().unwrap_or(f64::NAN));

    Ok(num)
}

pub fn parse_int(cx: CallContext) -> Result<Value, Value> {
    let input_string = cx.args.first().unwrap_or_undefined().to_js_string(cx.scope)?;

    let mut radix = cx
        .args
        .get(1)
        .cloned()
        .map(|v| v.to_number(cx.scope))
        .transpose()?
        .map(|r| r as i32)
        .unwrap_or(10);

    let mut trimmed_string = input_string.res(cx.scope).trim();

    let mut sign = 1;

    if trimmed_string.starts_with('-') {
        sign = -1;
    }

    // If S is not empty and the first code unit of S is either
    // the code unit 0x002B (PLUS SIGN) or the code unit 0x002D (HYPHEN-MINUS),
    // set S to the substring of S from index 1.
    if trimmed_string.starts_with(&['+', '-']) {
        trimmed_string = &trimmed_string[1..];
    }

    let mut strip_prefix = true;

    if radix != 0 {
        if radix < 2 || radix > 36 {
            return Ok(Value::number(f64::NAN));
        }

        if radix != 16 {
            strip_prefix = false;
        }
    } else {
        radix = 10;
    }

    // If stripPrefix is true, then
    if strip_prefix {
        // If the length of S is at least 2 and the first two code units of S are either "0x" or "0X"
        if trimmed_string.len() >= 2 && (trimmed_string.starts_with("0x") || trimmed_string.starts_with("0X")) {
            trimmed_string = &trimmed_string[2..];
            radix = 16;
        }
    }

    let radix = radix as u32; // by here it cannot be negative..

    // If S contains a code unit that is not a radix-R digit,
    // let end be the index within S of the first such code unit;
    // otherwise let end be the length of S.
    let end = trimmed_string
        .find(|c: char| !c.is_digit(radix))
        .unwrap_or_else(|| trimmed_string.len());

    let z = &trimmed_string[0..end];

    if z.is_empty() {
        return Ok(Value::number(f64::NAN));
    }

    let output = i128::from_str_radix(z, radix).unwrap();

    Ok(Value::number((sign * output) as f64))
}
