use crate::throw;
use crate::value::function::bound::BoundFunction;
use crate::value::function::native::CallContext;
use crate::value::promise::Promise;
use crate::value::promise::PromiseRejecter;
use crate::value::promise::PromiseResolver;
use crate::value::promise::PromiseState;
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
    // TODO: do not wrap thenable values in another promise
    let promise = Promise::resolved(cx.scope, value);
    Ok(Value::Object(cx.scope.register(promise)))
}

pub fn reject(cx: CallContext) -> Result<Value, Value> {
    let value = cx.args.first().unwrap_or_undefined();
    let promise = Promise::resolved(cx.scope, value);
    Ok(Value::Object(cx.scope.register(promise)))
}

pub fn then(cx: CallContext) -> Result<Value, Value> {
    let promise = match cx.this {
        Value::Object(obj) => obj,
        _ => throw!(cx.scope, "Receiver must be a promise"),
    };

    let promise = match promise.as_any().downcast_ref::<Promise>() {
        Some(promise) => promise,
        None => throw!(cx.scope, "Receiver must be a promise"),
    };

    let handler = match cx.args.first() {
        Some(Value::Object(obj)) if matches!(obj.type_of(), Typeof::Function) => obj.clone(),
        _ => throw!(cx.scope, "Promise handler must be a function"),
    };

    let mut state = promise.state().borrow_mut();

    match &mut *state {
        PromiseState::Pending { resolve, .. } => resolve.push(handler),
        PromiseState::Resolved(value) => {
            let value = value.clone();
            let bf = BoundFunction::new(cx.scope, handler, None, Some(vec![value]));
            let bf = cx.scope.register(bf);
            cx.scope.add_async_task(bf);
        }
        PromiseState::Rejected(..) => {}
    }

    // TODO: return a new promise
    Ok(Value::undefined())
}

// TODO: Promise.prototype.catch
