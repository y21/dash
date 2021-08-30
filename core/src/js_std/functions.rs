use crate::gc::Handle;
use crate::vm::value::{function::CallContext, Value};

/// Implements isNaN
///
/// https://tc39.es/ecma262/multipage/global-object.html#sec-isnan-number
pub fn is_nan(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    let value = match ctx.args.first() {
        Some(v) => v,
        None => return Ok(ctx.vm.create_js_value(true).into_handle(ctx.vm)),
    };

    let value = unsafe { value.borrow_unbounded() }.as_number();

    Ok(ctx.vm.create_js_value(value.is_nan()).into_handle(ctx.vm))
}
