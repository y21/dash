use super::todo;
use crate::vm::value::function::CallContext;
use crate::vm::value::function::NativeFunctionCallbackResult;

/// The number constructor
///
/// https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-number-constructor
pub fn number_constructor(ctx: CallContext) -> NativeFunctionCallbackResult {
    todo("Number", ctx.vm)
}
