use std::rc::Rc;
use std::{borrow::Cow, cell::RefCell};

use crate::vm::value::{function::CallContext, Value, ValueKind};

pub fn is_nan(value: CallContext) -> Rc<RefCell<Value>> {
    let value = match value.args.first() {
        Some(v) => v,
        None => return Value::from(true).into(),
    };

    let value = value.borrow().as_number();

    Value::from(value.is_nan()).into()
}
