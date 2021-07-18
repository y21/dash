use std::borrow::Cow;

use crate::{
    gc::Handle,
    vm::{
        value::{
            function::{CallContext, CallResult},
            object::AnyObject,
            Value,
        },
        VM,
    },
};

/// A value that is either reference counted or owned
pub enum MaybeRc<T> {
    /// Reference counted JS value
    Rc(Handle<Value>),
    /// Owned T
    Owned(T),
}

impl<'a> From<&'a str> for MaybeRc<&'a str> {
    fn from(s: &'a str) -> Self {
        Self::Owned(s)
    }
}

/// Creates a JS error given a string
pub fn create_error(message: MaybeRc<&str>, vm: &VM) -> Handle<Value> {
    let mut error = Value::from(AnyObject {});
    error.update_internal_properties(&vm.statics.error_proto, &vm.statics.error_ctor);

    let message_str = match message {
        MaybeRc::Rc(r) => unsafe { r.borrow_unbounded() }.to_string().to_string(),
        MaybeRc::Owned(r) => String::from(r),
    };

    let stack = vm.generate_stack_trace(Some(&message_str));

    error.set_property("message", vm.create_js_value(message_str).into_handle(vm));

    error.set_property("stack", vm.create_js_value(stack).into_handle(vm));

    error.into_handle(vm)
}

/// The error constructor
///
/// https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-error-constructor
pub fn error_constructor(value: CallContext) -> Result<CallResult, Handle<Value>> {
    let message_cell = value.args.first();
    let message_cell_ref = message_cell.map(|c| unsafe { c.borrow_unbounded() });
    let message = message_cell_ref
        .as_deref()
        .map(|v| &**v)
        .map(Value::to_string)
        .unwrap_or(Cow::Borrowed(""));

    Ok(CallResult::Ready(create_error(
        MaybeRc::Owned(&message),
        value.vm,
    )))
}
