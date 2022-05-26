use crate::throw;
use crate::vm::value::function::native::CallContext;
use crate::vm::value::Value;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    throw!(cx.scope, "Dynamic code compilation is currently not supported")
}
