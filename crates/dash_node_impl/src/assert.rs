use dash_vm::localscope::LocalScope;
use dash_vm::throw;
use dash_vm::value::function::native::{CallContext, register_native_fn};
use dash_vm::value::ops::conversions::ValueConversion;
use dash_vm::value::{ExceptionContext, Value};

use crate::state::state_mut;
use crate::symbols::NodeSymbols;

pub fn init_module(sc: &mut LocalScope<'_>) -> Result<Value, Value> {
    let NodeSymbols { assert: assert_sym, .. } = state_mut(sc).sym;
    let js_assert = register_native_fn(sc, assert_sym, js_assert);
    Ok(Value::object(js_assert))
}

fn js_assert(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let value = cx.args.first().or_type_err(scope, "Missing value to assert")?;
    let message = cx.args.get(1);

    // TODO: throw AssertionError
    if !value.is_truthy(scope) {
        match message {
            Some(message) => {
                let message = message.to_js_string(scope)?.res(scope).to_owned();
                throw!(scope, Error, "Assertion failed: {}", message)
            }
            None => throw!(scope, Error, "Assertion failed"),
        }
    }

    Ok(Value::undefined())
}
