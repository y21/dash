use std::borrow::Cow;

use crate::{
    gc::Handle,
    vm::{
        value::{function::CallContext, object::Object, Value},
        VM,
    },
};

/// Creates a JS error given a string
pub fn create_error<S: Into<String>>(message: S, vm: &VM) -> Handle<Value> {
    let mut error = Value::from(Object::Ordinary);
    error.update_internal_properties(&vm.statics.error_proto, &vm.statics.error_ctor);

    let message_str: String = message.into();

    let stack = vm.generate_stack_trace(Some(&message_str));

    error.set_property(
        "message".into(),
        vm.create_js_value(message_str).into_handle(vm),
    );

    error.set_property("stack".into(), vm.create_js_value(stack).into_handle(vm));

    error.into_handle(vm)
}

/// The error constructor
///
/// https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-error-constructor
pub fn error_constructor(value: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    let message_cell = value.args.first();
    let message_cell_ref = message_cell.map(|c| unsafe { c.borrow_unbounded() });
    let message = message_cell_ref
        .as_deref()
        .map(|v| &**v)
        .map(Value::to_string)
        .unwrap_or(Cow::Borrowed(""));

    Ok(create_error(message.into_owned(), value.vm))
}
