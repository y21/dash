use crate::vm::value::function::native::CallContext;
use crate::vm::value::Value;

pub fn to_string(cx: CallContext) -> Result<Value, Value> {
    if let Value::Boolean(b) = cx.this {
        let s = b
            .then(|| cx.scope.statics().get_true())
            .unwrap_or_else(|| cx.scope.statics().get_false());

        Ok(Value::String(s))
    } else {
        todo!()
    }
}
