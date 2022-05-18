use crate::vm::value::arraybuffer::ArrayBuffer;
use crate::vm::value::function::native::CallContext;
use crate::vm::value::ops::abstractions::conversions::ValueConversion;
use crate::vm::value::Value;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let length = match cx.args.first() {
        Some(length) => length.to_number(cx.scope)? as usize,
        None => 0,
    };

    let buf = ArrayBuffer::with_capacity(cx.scope, length);
    Ok(cx.scope.register(buf).into())
}
