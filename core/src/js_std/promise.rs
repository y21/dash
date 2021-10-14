use super::todo;
use crate::{
    gc::Handle,
    vm::value::{
        function::CallContext,
        object::Object,
        promise::{Promise, PromiseState},
        Value,
    },
};

/// The promise constructor
///
/// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-promise-constructor
pub fn promise_constructor(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo("Promise", ctx.vm)
}

/// Implements Promise.resolve
///
/// https://tc39.es/ecma262/multipage/control-abstraction-objects.html#sec-promise.resolve
pub fn resolve(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    let value_cell = Value::unwrap_or_undefined(ctx.args.first().cloned(), ctx.vm);
    let is_promise = unsafe { value_cell.borrow_unbounded() }
        .as_object()
        .and_then(Object::as_promise)
        .is_some();

    if is_promise {
        // Do not nest promises
        Ok(value_cell)
    } else {
        let promise = Promise::new(PromiseState::Resolved(value_cell));
        Ok(ctx.vm.create_js_value(promise).into_handle(ctx.vm))
    }
}

/// Implements Promise.reject
///
/// https://tc39.es/ecma262/multipage/control-abstraction-objects.html#sec-promise.reject
pub fn reject(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    let value_cell = Value::unwrap_or_undefined(ctx.args.first().cloned(), ctx.vm);

    let promise = Promise::new(PromiseState::Rejected(value_cell));
    Ok(ctx.vm.create_js_value(promise).into_handle(ctx.vm))
}
