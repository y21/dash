use std::{cell::RefCell, rc::Rc};

use crate::vm::value::{function::CallContext, object::Object, Value, ValueKind};

pub fn push(value: CallContext) -> Rc<RefCell<Value>> {
    let this_cell = value.receiver.unwrap();
    // TODO: assert typeof this == array

    let mut this = this_cell.borrow_mut();
    let this_arr = match this.as_object_mut() {
        Some(Object::Array(a)) => a,
        _ => unreachable!(),
    };

    for value in value.args.into_iter().rev() {
        this_arr.elements.push(value);
    }

    Rc::new(RefCell::new(Value::new(ValueKind::Number(
        this_arr.elements.len() as f64,
    ))))
}
