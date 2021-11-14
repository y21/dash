use crate::vm::value::function::CallContext;
use crate::vm::value::function::NativeFunctionCallbackResult;

/// Implements isNaN
///
/// https://tc39.es/ecma262/multipage/global-object.html#sec-isnan-number
pub fn is_nan(ctx: CallContext) -> NativeFunctionCallbackResult {
    let value = match ctx.args.first() {
        Some(v) => v,
        None => return Ok(true.into()),
    };

    Ok(value.as_number(ctx.vm).is_nan().into())
}
