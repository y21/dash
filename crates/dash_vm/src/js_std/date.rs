use crate::throw;
use crate::value::function::native::CallContext;
use crate::value::root_ext::RootErrExt;
use crate::value::Value;

pub fn time_millis(cx: &mut CallContext) -> Result<u64, Value> {
    let callback = match cx.scope.params().time_millis_callback {
        Some(c) => c,
        None => throw!(&mut cx.scope, Error, "Failed to get the current time"),
    };

    callback(cx.scope).root_err(cx.scope)
}

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    throw!(cx.scope, Error, "Date constructor is currently unimplemented")
}

pub fn now(mut cx: CallContext) -> Result<Value, Value> {
    let time = time_millis(&mut cx)?;
    Ok(Value::number(time as f64))
}
