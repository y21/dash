use std::{cell::RefCell, rc::Rc};

use crate::vm::value::{function::CallContext, Value, ValueKind};

pub fn log(value: CallContext) -> Result<Rc<RefCell<Value>>, Rc<RefCell<Value>>> {
    for value_cell in value.arguments() {
        let value_cell_ref = value_cell.borrow();
        let value_string = value_cell_ref.inspect();

        println!("{}", &*value_string);
    }

    Ok(Value::new(ValueKind::Undefined).into())
}
