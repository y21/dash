use dash_middle::parser::error::IntoFormattableErrors;

use crate::eval::EvalError;
use crate::localscope::LocalScope;
use crate::throw;
use crate::value::function::native::CallContext;
use crate::value::ops::conversions::ValueConversion;
use crate::value::{Root, Unpack, Value, ValueContext, ValueKind};

pub fn is_nan(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    // 1. Let num be ? ToNumber(number).
    let num = cx.args.first().unwrap_or_undefined().to_number(scope)?;
    // 2. If num is NaN, return true.
    // 3. Otherwise, return false.
    Ok(Value::boolean(num.is_nan()))
}

pub fn eval(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let source = {
        let value = cx.args.first().unwrap_or_undefined();
        match value.unpack() {
            ValueKind::String(s) => s.res(scope).to_owned(),
            _ => return Ok(value),
        }
    };

    match scope.eval(&source, Default::default()) {
        Ok(v) => Ok(v.root(scope)),
        Err(EvalError::Exception(ex)) => Err(ex.root(scope)),
        Err(EvalError::Middle(err)) => throw!(scope, SyntaxError, "{}", err.formattable(&source, true)),
    }
}

pub fn log(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    for arg in cx.args {
        let tstr = arg.to_js_string(scope)?;
        println!("{} ", tstr.res(scope));
    }

    Ok(Value::undefined())
}

pub fn is_finite(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    // 1. Let num be ? ToNumber(number).
    let num = cx.args.first().unwrap_or_undefined().to_number(scope)?;
    // 2. If num is NaN, +∞, or -∞, return false.
    // 3. Otherwise, return true.
    Ok(Value::boolean(num.is_finite()))
}

pub fn parse_float(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    // 1. Let inputString be ? ToString(string).
    let input_string = cx.args.first().unwrap_or_undefined().to_js_string(scope)?;
    // 2. Let trimmedString be ! TrimString(inputString, start).
    let trimmed_string = input_string.res(scope).trim();

    // TODO: follow spec
    let num = Value::number(trimmed_string.parse().unwrap_or(f64::NAN));

    Ok(num)
}

pub fn parse_int(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let input_string = cx.args.first().unwrap_or_undefined().to_js_string(scope)?;
    let radix = cx
        .args
        .get(1)
        .cloned()
        .map(|v| v.to_number(scope))
        .transpose()?
        .map(|r| r as u32)
        .unwrap_or(10);

    let trimmed_string = input_string.res(scope).trim();

    // TODO: follow spec
    let num = Value::number(
        i32::from_str_radix(trimmed_string, radix)
            .map(|n| n as f64)
            .unwrap_or(f64::NAN),
    );

    Ok(num)
}
