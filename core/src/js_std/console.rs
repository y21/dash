use std::{borrow::Cow, cell::RefCell, rc::Rc};

use crate::vm::value::{function::CallContext, Value, ValueKind};

pub fn log(value: CallContext) -> Rc<RefCell<Value>> {
    let value_cell = value.args.first().map(|c| c.borrow());
    let value_string = value_cell
        .as_deref()
        .map(Value::to_string)
        .unwrap_or(Cow::Borrowed("undefined"));

    println!("{}", &*value_string);

    Rc::new(RefCell::new(Value::new(ValueKind::Undefined)))
}
