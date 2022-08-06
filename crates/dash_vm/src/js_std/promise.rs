use crate::throw;
use crate::value::function::native::CallContext;
use crate::value::promise::Promise;
use crate::value::promise::PromiseRejecter;
use crate::value::promise::PromiseResolver;
use crate::value::Typeof;
use crate::value::Value;
use crate::value::ValueContext;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let initiator = match cx.args.first() {
        Some(v) if matches!(v.type_of(), Typeof::Function) => v,
        _ => throw!(cx.scope, "Promise callback must be a function"),
    };

    let promise = {
        let p = Promise::new(cx.scope);
        cx.scope.register(p)
    };

    let (resolve, reject) = {
        let r1 = PromiseResolver::new(cx.scope, promise.clone());
        let r2 = PromiseRejecter::new(cx.scope, promise.clone());
        (cx.scope.register(r1), cx.scope.register(r2))
    };

    initiator.apply(
        cx.scope,
        Value::undefined(),
        vec![Value::Object(resolve), Value::Object(reject)],
    )?;

    Ok(Value::Object(cx.scope.register(promise)))
}

pub fn resolve(cx: CallContext) -> Result<Value, Value> {
    let value = cx.args.first().unwrap_or_undefined();
    let promise = Promise::resolved(cx.scope, value);
    Ok(Value::Object(cx.scope.register(promise)))
}

pub fn reject(cx: CallContext) -> Result<Value, Value> {
    let value = cx.args.first().unwrap_or_undefined();
    let promise = Promise::resolved(cx.scope, value);
    Ok(Value::Object(cx.scope.register(promise)))
}
