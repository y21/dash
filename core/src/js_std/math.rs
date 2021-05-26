use std::{cell::RefCell, rc::Rc};

use crate::vm::value::{function::CallContext, Value, ValueKind};

pub fn abs(value: CallContext) -> Rc<RefCell<Value>> {
    let num = value
        .args
        .first()
        .map(|c| c.borrow().as_number())
        .unwrap_or(f64::NAN);

    Value::new(ValueKind::Number(num.abs())).into()
}

pub fn ceil(value: CallContext) -> Rc<RefCell<Value>> {
    let num = value
        .args
        .first()
        .map(|c| c.borrow().as_number())
        .unwrap_or(f64::NAN);

    Value::new(ValueKind::Number(num.ceil())).into()
}

pub fn floor(value: CallContext) -> Rc<RefCell<Value>> {
    let num = value
        .args
        .first()
        .map(|c| c.borrow().as_number())
        .unwrap_or(f64::NAN);

    Value::new(ValueKind::Number(num.floor())).into()
}

pub fn max(value: CallContext) -> Rc<RefCell<Value>> {
    let mut arguments = value.arguments();
    let mut max = arguments.next().cloned().unwrap();
    let mut max_num = max.borrow().as_number();

    for arg_cell in arguments {
        let arg = arg_cell.borrow().as_number();
        if arg > max_num {
            max_num = arg;
            max = arg_cell.clone();
        }
    }

    max
}

pub fn min(value: CallContext) -> Rc<RefCell<Value>> {
    let mut arguments = value.arguments();
    let mut max = arguments.next().cloned().unwrap();
    let mut max_num = max.borrow().as_number();

    for arg_cell in arguments {
        let arg = arg_cell.borrow().as_number();
        if arg < max_num {
            max_num = arg;
            max = arg_cell.clone();
        }
    }

    max
}

pub fn pow(value: CallContext) -> Rc<RefCell<Value>> {
    let mut args = value.arguments();

    let lhs = args
        .next()
        .map(|n| n.borrow().as_number())
        .unwrap_or(f64::NAN);

    let rhs = args
        .next()
        .map(|n| n.borrow().as_number())
        .unwrap_or(f64::NAN);

    let result = lhs.powf(rhs);

    Value::new(ValueKind::Number(result)).into()
}
