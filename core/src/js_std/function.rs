use crate::gc::Handle;
use crate::vm::value::function::CallResult;
use crate::vm::value::{function::CallContext, Value};

/// The function constructor
///
/// https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-function-constructor
pub fn function_constructor(_args: CallContext) -> Result<CallResult, Handle<Value>> {
    todo!()
}
