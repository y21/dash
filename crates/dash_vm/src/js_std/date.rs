use crate::throw;
use crate::value::Value;
use crate::value::date::Date;
use crate::value::function::native::CallContext;
use crate::value::object::OrdObject;
use crate::value::root_ext::RootErrExt;

use super::receiver_t;

pub fn time_millis(cx: &mut CallContext) -> Result<u64, Value> {
    let callback = match cx.scope.params().time_millis_callback {
        Some(c) => c,
        None => throw!(&mut cx.scope, Error, "Failed to get the current time"),
    };

    callback(cx.scope).root_err(cx.scope)
}

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    // FIXME: `new` has special behavior
    let new_target = cx.new_target.unwrap_or(cx.scope.statics.date_ctor);
    let date = Date::new_with_object(OrdObject::instance_for_new_target(new_target, cx.scope)?, cx.scope)?;
    Ok(cx.scope.register(date).into())
}

pub fn now(mut cx: CallContext) -> Result<Value, Value> {
    let time = time_millis(&mut cx)?;
    Ok(Value::number(time as f64))
}

pub fn get_time(cx: CallContext) -> Result<Value, Value> {
    let this = receiver_t::<Date>(cx.scope, &cx.this, "Date.prototype.getTime")?;
    Ok(Value::number(this.timestamp as f64))
}
