use crate::vm::value::{function::CallContext, Value, ValueKind};
use std::cell::RefCell;
use std::rc::Rc;

pub fn string_constructor(_args: CallContext) -> Result<Rc<RefCell<Value>>, Rc<RefCell<Value>>> {
    Ok(Value::new(ValueKind::Undefined).into())
}

pub fn index_of(_args: CallContext) -> Result<Rc<RefCell<Value>>, Rc<RefCell<Value>>> {
    Ok(Value::new(ValueKind::Undefined).into())
}
