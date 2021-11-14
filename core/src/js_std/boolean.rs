use super::todo;
use crate::vm::value::function::CallContext;
use crate::vm::value::function::NativeFunctionCallbackResult;

/// The boolean constructor
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-boolean-constructor
pub fn boolean_constructor(ctx: CallContext) -> NativeFunctionCallbackResult {
    todo("Boolean", ctx.vm)
}
