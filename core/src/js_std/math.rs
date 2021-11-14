use super::error;
use crate::vm::value::function::{CallContext, NativeFunctionCallbackResult};

/// Implements Math.abs
///
/// https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-math.abs
pub fn abs(ctx: CallContext) -> NativeFunctionCallbackResult {
    let num = ctx
        .args
        .first()
        .map(|v| v.as_number(ctx.vm))
        .unwrap_or(f64::NAN);

    Ok(num.abs().into())
}

/// Implements Math.ceil
///
/// https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-ceil.abs
pub fn ceil(ctx: CallContext) -> NativeFunctionCallbackResult {
    let num = ctx
        .args
        .first()
        .map(|v| v.as_number(ctx.vm))
        .unwrap_or(f64::NAN);

    Ok(num.ceil().into())
}

/// Implements Math.floor
///
/// https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-floor.abs
pub fn floor(ctx: CallContext) -> NativeFunctionCallbackResult {
    let num = ctx
        .args
        .first()
        .map(|v| v.as_number(ctx.vm))
        .unwrap_or(f64::NAN);

    Ok(num.floor().into())
}

/// Implements Math.max
///
/// https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-math.max
pub fn max(ctx: CallContext) -> NativeFunctionCallbackResult {
    let mut arguments = ctx.arguments();
    let mut max = match arguments.next().cloned() {
        Some(value) => value.as_number(ctx.vm),
        None => return Ok((-f64::INFINITY).into()),
    };

    for arg in arguments {
        let arg = arg.as_number(ctx.vm);
        if arg > max {
            max = arg;
        }
    }

    Ok(max.into())
}

/// Implements Math.min
///
/// https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-math.min
pub fn min(ctx: CallContext) -> NativeFunctionCallbackResult {
    let mut arguments = ctx.arguments();
    let mut min = match arguments.next().cloned() {
        Some(value) => value.as_number(ctx.vm),
        None => return Ok(f64::INFINITY.into()),
    };

    for arg in arguments {
        let arg = arg.as_number(ctx.vm);
        if arg < min {
            min = arg;
        }
    }

    Ok(min.into())
}

/// Implements Math.pow
///
/// https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-math.pow
pub fn pow(ctx: CallContext) -> NativeFunctionCallbackResult {
    let mut args = ctx.arguments().map(|n| n.as_number(ctx.vm));

    let lhs = args.next().unwrap_or(f64::NAN);
    let rhs = args.next().unwrap_or(f64::NAN);

    Ok(lhs.powf(rhs).into())
}

/// Implements Math.random
///
/// https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-math.random
pub fn random(ctx: CallContext) -> NativeFunctionCallbackResult {
    let maybe_random = ctx.vm.agent.random();

    match maybe_random {
        Some(rand) => Ok(rand.into()),
        None => Err(error::create_error("Random number generation failed", ctx.vm).into()),
    }
}
