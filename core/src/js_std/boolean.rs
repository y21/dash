use crate::vm::value::function::CallResult;
use crate::vm::value::{function::CallContext, Value, ValueKind};
use std::cell::RefCell;
use std::rc::Rc;

/// The boolean constructor
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-boolean-constructor
pub fn boolean_constructor(_args: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    Ok(CallResult::Ready(Value::new(ValueKind::Undefined).into()))
}
