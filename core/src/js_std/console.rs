use std::{cell::RefCell, rc::Rc};

use crate::vm::value::{
    function::{CallContext, CallResult},
    Value, ValueKind,
};

pub fn log(value: CallContext) -> Result<CallResult, Rc<RefCell<Value>>> {
    for value_cell in value.arguments() {
        let value_cell_ref = value_cell.borrow();
        let value_string = value_cell_ref.inspect(0);

        println!("{}", &*value_string);
    }

    Ok(CallResult::Ready(Value::new(ValueKind::Undefined).into()))
}
