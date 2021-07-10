use std::{cell::RefCell, rc::Rc};

use crate::vm::value::{
    function::{CallContext, CallResult},
    object::Object,
    promise::{Promise, PromiseState},
    Value, ValueKind,
};

/// The promise constructor
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-promise-constructor
pub fn promise_constructor(_args: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    Ok(CallResult::Ready(Value::new(ValueKind::Undefined).into()))
}

/// Implements Promise.resolve
///
/// https://tc39.es/ecma262/multipage/control-abstraction-objects.html#sec-promise.resolve
pub fn resolve(ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    let value_cell = Value::unwrap_or_undefined(ctx.args.first().cloned());
    let is_promise = value_cell
        .borrow()
        .as_object()
        .and_then(Object::as_promise)
        .is_some();

    if is_promise {
        // Do not nest promises
        Ok(CallResult::Ready(value_cell))
    } else {
        let promise = Promise::new(PromiseState::Resolved(value_cell));
        Ok(CallResult::Ready(ctx.vm.create_js_value(promise).into()))
    }
}

/// Implements Promise.reject
///
/// https://tc39.es/ecma262/multipage/control-abstraction-objects.html#sec-promise.reject
pub fn reject(ctx: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    let value_cell = Value::unwrap_or_undefined(ctx.args.first().cloned());

    let promise = Promise::new(PromiseState::Rejected(value_cell));
    Ok(CallResult::Ready(ctx.vm.create_js_value(promise).into()))
}
