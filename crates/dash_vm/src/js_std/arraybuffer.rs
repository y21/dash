use crate::throw;
use crate::value::arraybuffer::ArrayBuffer;
use crate::value::function::native::CallContext;
use crate::value::ops::conversions::ValueConversion;
use crate::value::Value;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let length = match cx.args.first() {
        Some(length) => length.to_number(cx.scope)? as usize,
        None => 0,
    };

    let buf = ArrayBuffer::with_capacity(cx.scope, length);
    Ok(cx.scope.register(buf).into())
}

pub fn byte_length(cx: CallContext) -> Result<Value, Value> {
    let Some(this) = cx.this.downcast_ref::<ArrayBuffer>(&cx.scope) else {
        throw!(
            cx.scope,
            TypeError,
            "ArrayBuffer.prototype.byteLength called on non-ArrayBuffer"
        )
    };
    Ok(Value::number(this.len() as f64))
}
