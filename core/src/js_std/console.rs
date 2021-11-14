use crate::vm::value::{
    function::{CallContext, NativeFunctionCallbackResult},
    Value, ValueKind,
};

/// Implements console.log
///
/// This is not part of the JS standard and may get removed at some point
pub fn log(ctx: CallContext) -> NativeFunctionCallbackResult {
    for value_cell in ctx.arguments() {
        let value_string = value_cell.inspect(ctx.vm, 0);

        println!("{}", &*value_string);
    }

    Ok(Value::new(ValueKind::Undefined))
}
