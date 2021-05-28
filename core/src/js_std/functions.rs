use std::cell::RefCell;
use std::rc::Rc;

use crate::vm::value::{function::CallContext, Value};

pub fn is_nan(value: CallContext) -> Result<Rc<RefCell<Value>>, Rc<RefCell<Value>>> {
    let value = match value.args.first() {
        Some(v) => v,
        None => return Ok(Value::from(true).into()),
    };

    let value = value.borrow().as_number();

    Ok(Value::from(value.is_nan()).into())
}
