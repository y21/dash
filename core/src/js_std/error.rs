use std::{borrow::Cow, cell::RefCell, rc::Rc};

use crate::vm::{
    value::{
        function::{CallContext, CallResult},
        Value,
    },
    VM,
};

pub enum MaybeRc<T> {
    Rc(Rc<RefCell<Value>>),
    Owned(T),
}

pub fn create_error(message: MaybeRc<&str>, vm: &VM) -> Rc<RefCell<Value>> {
    let mut error = vm.create_object();

    let message_str = match message {
        MaybeRc::Rc(r) => r.borrow().to_string().to_string(),
        MaybeRc::Owned(r) => String::from(r),
    };

    let stack = vm.generate_stack_trace(Some(&message_str));

    error.set_property("message", vm.create_js_value(message_str).into());

    error.set_property("stack", vm.create_js_value(stack).into());

    error.into()
}

pub fn error_constructor(value: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    let message_cell = value.args.first();
    let message_cell_ref = message_cell.map(|c| c.borrow());
    let message = message_cell_ref
        .as_deref()
        .map(Value::to_string)
        .unwrap_or(Cow::Borrowed(""));

    Ok(CallResult::Ready(create_error(
        MaybeRc::Owned(&message),
        value.vm,
    )))
}
