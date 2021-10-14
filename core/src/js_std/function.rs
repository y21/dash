use super::todo;
use crate::gc::Handle;
use crate::vm::value::{function::CallContext, Value};

/// The function constructor
///
/// https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-function-constructor
pub fn function_constructor(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo("Function", ctx.vm)
}
