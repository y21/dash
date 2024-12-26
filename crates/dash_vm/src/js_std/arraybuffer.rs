use crate::throw;
use crate::value::arraybuffer::ArrayBuffer;
use crate::value::function::native::CallContext;
use crate::value::object::NamedObject;
use crate::value::ops::conversions::ValueConversion;
use crate::value::{Unpack, Value};

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let length = match cx.args.first() {
        Some(length) => length.to_number(cx.scope)? as usize,
        None => 0,
    };
    if let Some(new_target) = cx.new_target {
        let buf = ArrayBuffer::with_capacity(length, NamedObject::instance_for_new_target(new_target, cx.scope)?);
        Ok(cx.scope.register(buf).into())
    } else {
        throw!(cx.scope, TypeError, "ArrayBuffer constructor requires new")
    }
}

pub fn byte_length(cx: CallContext) -> Result<Value, Value> {
    let this = cx.this.unpack();
    let Some(this) = this.downcast_ref::<ArrayBuffer>(cx.scope) else {
        throw!(
            cx.scope,
            TypeError,
            "ArrayBuffer.prototype.byteLength called on non-ArrayBuffer"
        )
    };
    Ok(Value::number(this.len() as f64))
}
