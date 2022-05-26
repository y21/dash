use crate::throw;
use crate::value::function::native::CallContext;
use crate::value::Value;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    throw!(cx.scope, "Dynamic code compilation is currently not supported")
}
