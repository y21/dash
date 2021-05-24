use std::rc::Rc;
use std::{borrow::Cow, cell::RefCell};

use crate::vm::value::{function::CallContext, Value, ValueKind};

pub fn is_nan(value: CallContext) -> Rc<RefCell<Value>> {
    let value = match value.args.first() {
        Some(v) => v,
        None => return Rc::new(RefCell::new(Value::new(ValueKind::Bool(true)))),
    };

    let value = value.borrow().as_number();

    Rc::new(RefCell::new(Value::new(ValueKind::Bool(value.is_nan()))))
}
