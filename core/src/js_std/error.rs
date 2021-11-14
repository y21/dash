use std::borrow::Cow;

use crate::{
    gc::Handle,
    vm::{
        value::{
            function::{CallContext, NativeFunctionCallbackResult},
            object::{Object, ObjectKind},
        },
        VM,
    },
};

/// Creates a JS error given a string
pub fn create_error<S: Into<String>>(message: S, vm: &VM) -> Handle<Object> {
    let mut error = Object::new(ObjectKind::Ordinary);
    error.update_internal_properties(&vm.statics.error_proto, &vm.statics.error_ctor);

    let message_str: String = message.into();

    let stack = vm.generate_stack_trace(Some(&message_str));

    error.set_property("message", vm.register_object(Object::from(message_str)));
    error.set_property("stack", vm.register_object(Object::from(stack)));

    error.into_handle(vm)
}

/// The error constructor
///
/// https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-error-constructor
pub fn error_constructor(ctx: CallContext) -> NativeFunctionCallbackResult {
    let message_cell = ctx.args.first();
    // let message_cell_ref = message_cell.map(|c| unsafe { c.borrow_unbounded() });
    let message = message_cell
        .map(|v| v.to_string(ctx.vm))
        .unwrap_or(Cow::Borrowed(""));

    Ok(create_error(message.into_owned(), ctx.vm).into())
}
