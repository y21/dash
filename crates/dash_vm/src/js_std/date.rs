use crate::throw;
use crate::value::date::Date;
use crate::value::function::native::CallContext;
use crate::value::root_ext::RootErrExt;
use crate::value::{Unpack, Value};

pub fn time_millis(cx: &mut CallContext) -> Result<u64, Value> {
    let callback = match cx.scope.params().time_millis_callback {
        Some(c) => c,
        None => throw!(&mut cx.scope, Error, "Failed to get the current time"),
    };

    callback(cx.scope).root_err(cx.scope)
}

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let date = Date::current(cx.scope)?;
    Ok(cx.scope.register(date).into())
}

pub fn now(mut cx: CallContext) -> Result<Value, Value> {
    let time = time_millis(&mut cx)?;
    Ok(Value::number(time as f64))
}

pub fn get_time(cx: CallContext) -> Result<Value, Value> {
    let Some(this) = cx
        .this
        .unpack()
        .downcast_ref::<Date>(cx.scope)
        .map(|date| date.timestamp)
    else {
        throw!(cx.scope, Error, "Incompatible receiver to Date.prototype.getTime")
    };
    Ok(Value::number(this as f64))
}
