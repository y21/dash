use std::{cell::RefCell, rc::Rc};

use crate::vm::value::{CallContext, Value, ValueKind};

pub fn pow(value: CallContext) -> Rc<RefCell<Value>> {
    let mut args = value.args.iter().rev();

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
