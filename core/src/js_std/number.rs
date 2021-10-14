use super::todo;
use crate::gc::Handle;
use crate::vm::value::{function::CallContext, Value};

/// The number constructor
///
/// https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-number-constructor
pub fn number_constructor(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo("Number", ctx.vm)
}
