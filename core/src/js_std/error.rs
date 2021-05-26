use std::{borrow::Cow, cell::RefCell, rc::Rc};

use crate::vm::value::{
    function::{CallContext, FunctionKind},
    object::{AnyObject, Object},
    Value, ValueKind,
};

pub fn error_constructor(value: CallContext) -> Rc<RefCell<Value>> {
    // Get reference to message
    let message_cell = value.args.first();
    let message_cell_ref = message_cell.map(|c| c.borrow());
    let message = message_cell_ref
        .as_deref()
        .and_then(Value::as_string_lossy)
        .unwrap_or(Cow::Borrowed(""));

    // Create error object
    let mut error = Value::new(ValueKind::Object(Box::new(Object::Any(AnyObject {}))));
    // Add message property
    error.set_property("message", message_cell.unwrap().clone());

    // Create stack string
    let mut stack = format!("Error: {}\n", message);

    // Iterate over frames and add it to the stack string
    for frame in value.vm.frames.as_array_bottom() {
        let frame = unsafe { &*frame.as_ptr() };
        stack.push_str("  at ");

        // Get reference to function
        let func = frame.func.borrow();
        let func_name = func
            .as_function()
            .and_then(FunctionKind::as_closure)
            .and_then(|c| c.func.name.as_ref());

        // Add function name to string (or <anonymous> if it's an anonymous function)
        stack.push_str(func_name.map(|x| &**x).unwrap_or("<anonymous>"));
        stack.push('\n');
    }

    // Add stack property
    error.set_property(
        "stack",
        Value::new(ValueKind::Object(Box::new(Object::String(stack)))).into(),
    );

    // Return constructed error object
    error.into()
}
