use std::cell::RefCell;
use std::rc::Rc;

use crate::vm::value::function::CallResult;
use crate::vm::value::{function::CallContext, Value};

/// Implements isNaN
///
/// https://tc39.es/ecma262/multipage/global-object.html#sec-isnan-number
pub fn is_nan(ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    let value = match ctx.args.first() {
        Some(v) => v,
        None => return Ok(CallResult::Ready(ctx.vm.create_js_value(true).into())),
    };

    let value = value.borrow().as_number();

    Ok(CallResult::Ready(
        ctx.vm.create_js_value(value.is_nan()).into(),
    ))
}
