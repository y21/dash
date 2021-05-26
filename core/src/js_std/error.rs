use std::{borrow::Cow, cell::RefCell, rc::Rc};

use crate::vm::{
    value::{
        function::CallContext,
        object::{AnyObject, Object},
        Value,
    },
    VM,
};

pub enum MaybeRc<T> {
    Rc(Rc<RefCell<Value>>),
    Owned(T),
}

pub fn create_error(message: MaybeRc<&str>, vm: &VM) -> Rc<RefCell<Value>> {
    let mut error = Value::from(AnyObject {});

    let message_str = match message {
        MaybeRc::Rc(r) => r.borrow().to_string().to_string(),
        MaybeRc::Owned(r) => String::from(r),
    };

    let stack = vm.generate_stack_trace(Some(&message_str));

    error.set_property("message", Value::from(Object::String(message_str)).into());

    error.set_property("stack", Value::from(Object::String(stack)).into());

    error.into()
}

pub fn error_constructor(value: CallContext) -> Rc<RefCell<Value>> {
    let message_cell = value.args.first();
    let message_cell_ref = message_cell.map(|c| c.borrow());
    let message = message_cell_ref
        .as_deref()
        .map(Value::to_string)
        .unwrap_or(Cow::Borrowed(""));

    create_error(MaybeRc::Owned(&message), value.vm)
}
