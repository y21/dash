use super::todo;
use crate::vm::value::function::CallContext;
use crate::vm::value::function::NativeFunctionCallbackResult;

/// The function constructor
///
/// https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-function-constructor
pub fn function_constructor(ctx: CallContext) -> NativeFunctionCallbackResult {
    todo("Function", ctx.vm)
}
