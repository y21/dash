use dash_proc_macro::Trace;

use crate::delegate;
use crate::gc::handle::Handle;
use crate::throw;
use crate::value::function::bound::BoundFunction;
use crate::value::function::native::CallContext;
use crate::value::object::NamedObject;
use crate::value::object::Object;
use crate::value::object::PropertyKey;
use crate::value::promise::Promise;
use crate::value::promise::PromiseRejecter;
use crate::value::promise::PromiseResolver;
use crate::value::promise::PromiseState;
use crate::value::Typeof;
use crate::value::Value;
use crate::value::ValueContext;
use crate::Vm;

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

    let then_promise = {
        let p = Promise::new(cx.scope);
        cx.scope.register(p)
    };
    let resolver = {
        let p = PromiseResolver::new(cx.scope, then_promise.clone());
        cx.scope.register(p)
    };
    let then_task = {
        let t = ThenTask::new(cx.scope, then_promise.clone(), handler, resolver);
        cx.scope.register(t)
    };

    match &mut *state {
        PromiseState::Pending { resolve, .. } => resolve.push(then_task),
        PromiseState::Resolved(value) => {
            let bf = BoundFunction::new(cx.scope, then_task, None, Some(vec![value.clone()]));
            let bf = cx.scope.register(bf);
            cx.scope.add_async_task(bf);
        }
        PromiseState::Rejected(..) => {}
    }

    Ok(Value::Object(then_promise))
}

// TODO: Promise.prototype.catch

#[derive(Debug, Trace)]
struct ThenTask {
    // TODO: make a type like CastHandle<Promise> that implements Deref by downcasting
    then_promise: Handle<dyn Object>,
    handler: Handle<dyn Object>,
    resolver: Handle<dyn Object>,
    obj: NamedObject,
}

impl ThenTask {
    pub fn new(
        vm: &mut Vm,
        then_promise: Handle<dyn Object>,
        handler: Handle<dyn Object>,
        resolver: Handle<dyn Object>,
    ) -> Self {
        Self {
            then_promise,
            handler,
            resolver,
            obj: NamedObject::new(vm),
        }
    }
}

impl Object for ThenTask {
    delegate!(
        obj,
        get_own_property_descriptor,
        get_property,
        get_property_descriptor,
        set_property,
        delete_property,
        set_prototype,
        get_prototype,
        as_any,
        own_keys
    );

    fn apply(
        &self,
        scope: &mut crate::local::LocalScope,
        _callee: Handle<dyn Object>,
        _this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        let resolved = args.first().unwrap_or_undefined();
        let ret = self.handler.apply(scope, Value::undefined(), vec![resolved])?;

        let ret_then = ret.get_property(scope, PropertyKey::String("then".into()))?;

        match ret_then {
            Value::Undefined(..) => {
                // Not a promise. Call resolver(value)
                let bf = BoundFunction::new(scope, self.resolver.clone(), None, Some(vec![ret]));
                let bf = scope.register(bf);
                scope.add_async_task(bf);
            }
            _ => {
                // Is a promise. Call value.then(resolver)
                ret_then.apply(scope, ret, vec![Value::Object(self.resolver.clone())])?;
            }
        }

        Ok(Value::undefined())
    }
}
