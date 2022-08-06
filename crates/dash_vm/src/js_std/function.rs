use crate::throw;
use crate::value::function::bound::BoundFunction;
use crate::value::function::native::CallContext;
use crate::value::Typeof;
use crate::value::Value;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    throw!(cx.scope, "Dynamic code compilation is currently not supported")
}

pub fn bind(cx: CallContext) -> Result<Value, Value> {
    let target_this = cx.args.first().cloned();
    let target_args = cx.args.get(1..).map(|s| s.to_vec());
    let target_callee = match cx.this {
        Value::Object(o) if matches!(o.type_of(), Typeof::Function) => o,
        _ => throw!(cx.scope, "Bound value must be a function"),
    };

    let bf = BoundFunction::new(cx.scope, target_callee, target_this, target_args);
    Ok(Value::Object(cx.scope.register(bf)))
}
