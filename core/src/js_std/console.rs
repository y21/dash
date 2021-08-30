use crate::{
    gc::Handle,
    vm::value::{function::CallContext, Value, ValueKind},
};

/// Implements console.log
///
/// This is not part of the JS standard and may get removed at some point
pub fn log(value: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    for value_cell in value.arguments() {
        let value_cell_ref = unsafe { value_cell.borrow_unbounded() };
        let value_string = value_cell_ref.inspect(0);

        println!("{}", &*value_string);
    }

    Ok(Value::new(ValueKind::Undefined).into_handle(value.vm))
}
