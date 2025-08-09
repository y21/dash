use crate::localscope::LocalScope;
use crate::throw;
use crate::value::Value;
use crate::value::date::Date;
use crate::value::function::native::CallContext;
use crate::value::object::OrdObject;
use crate::value::root_ext::RootErrExt;

use super::receiver_t;

pub fn time_millis(scope: &mut LocalScope<'_>) -> Result<u64, Value> {
    let callback = match scope.params().time_millis_callback {
        Some(c) => c,
        None => throw!(scope, Error, "Failed to get the current time"),
    };

    callback(scope).root_err(scope)
}

pub fn constructor(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    // FIXME: `new` has special behavior
    let new_target = cx.new_target.unwrap_or(scope.statics.date_ctor);
    let date = Date::new_with_object(OrdObject::instance_for_new_target(new_target, scope)?, scope)?;
    Ok(scope.register(date).into())
}

pub fn now(_: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let time = time_millis(scope)?;
    Ok(Value::number(time as f64))
}

pub fn get_time(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let this = receiver_t::<Date>(scope, &cx.this, "Date.prototype.getTime")?;
    Ok(Value::number(this.timestamp as f64))
}
