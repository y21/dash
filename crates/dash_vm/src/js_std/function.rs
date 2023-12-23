use crate::throw;
use crate::value::function::bound::BoundFunction;
use crate::value::function::native::CallContext;
use crate::value::function::Function;
use crate::value::Root;
use crate::value::Typeof;
use crate::value::Value;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    throw!(cx.scope, Error, "Dynamic code compilation is currently not supported")
}

pub fn bind(cx: CallContext) -> Result<Value, Value> {
    let target_this = cx.args.first().cloned();
    let target_args = cx.args.get(1..).map(|s| s.to_vec());
    let target_callee = match cx.this {
        Value::Object(o) if matches!(o.type_of(), Typeof::Function) => o,
        _ => throw!(cx.scope, TypeError, "Bound value must be a function"),
    };

    let bf = BoundFunction::new(cx.scope, target_callee, target_this, target_args);
    Ok(Value::Object(cx.scope.register(bf)))
}

pub fn call(cx: CallContext) -> Result<Value, Value> {
    let target_this = cx.args.first().cloned();
    let target_args = cx.args.get(1..).map(|s| s.to_vec());
    let target_callee = match cx.this {
        Value::Object(o) if matches!(o.type_of(), Typeof::Function) => o,
        _ => throw!(cx.scope, TypeError, "Bound value must be a function"),
    };

    target_callee
        .apply(
            cx.scope,
            target_this.unwrap_or_else(Value::undefined),
            target_args.unwrap_or_default(),
        )
        .root(cx.scope)
}

pub fn to_string(cx: CallContext) -> Result<Value, Value> {
    let Some(this) = cx.this.downcast_ref::<Function>() else {
        throw!(cx.scope, TypeError, "Incompatible receiver");
    };
    Ok(Value::String(
        format!(
            "function {}() {{ [native code] }}",
            this.name().as_deref().unwrap_or(&cx.scope.statics.empty_str)
        )
        .into(),
    ))
}
