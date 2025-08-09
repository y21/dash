use crate::localscope::LocalScope;
use crate::throw;
use crate::value::Value;
use crate::value::arraybuffer::ArrayBuffer;
use crate::value::function::native::CallContext;
use crate::value::object::OrdObject;
use crate::value::ops::conversions::ValueConversion;

use super::receiver_t;

pub fn constructor(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let length = match cx.args.first() {
        Some(length) => length.to_number(scope)? as usize,
        None => 0,
    };
    if let Some(new_target) = cx.new_target {
        let buf = ArrayBuffer::with_capacity(length, OrdObject::instance_for_new_target(new_target, scope)?);
        Ok(scope.register(buf).into())
    } else {
        throw!(scope, TypeError, "ArrayBuffer constructor requires new")
    }
}

pub fn byte_length(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let this = receiver_t::<ArrayBuffer>(scope, &cx.this, "ArrayBuffer.prototype.byteLength")?;
    Ok(Value::number(this.len() as f64))
}
