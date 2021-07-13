use crate::gc::Handle;
use crate::vm::value::function::CallResult;
use crate::vm::value::{function::CallContext, Value};

/// The boolean constructor
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-boolean-constructor
pub fn boolean_constructor(_args: CallContext) -> Result<CallResult, Handle<Value>> {
    todo!()
}
