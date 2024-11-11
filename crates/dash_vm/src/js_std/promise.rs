use dash_proc_macro::Trace;

use crate::gc::ObjectId;
use crate::value::function::bound::BoundFunction;
use crate::value::function::native::CallContext;
use crate::value::object::{NamedObject, Object, PropertyKey};
use crate::value::promise::{Promise, PromiseRejecter, PromiseResolver, PromiseState};
use crate::value::root_ext::RootErrExt;
use crate::value::{Root, Typeof, Unpack, Unrooted, Value, ValueContext, ValueKind};
use crate::{delegate, extract, throw, Vm};
use dash_middle::interner::sym;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let initiator = match cx.args.first() {
        Some(v) if matches!(v.type_of(cx.scope), Typeof::Function) => v,
        _ => throw!(cx.scope, TypeError, "Promise callback must be a function"),
    };

    let promise = {
        let p = Promise::new(cx.scope);
        cx.scope.register(p)
    };

    let (resolve, reject) = {
        let r1 = PromiseResolver::new(cx.scope, promise);
        let r2 = PromiseRejecter::new(cx.scope, promise);
        (cx.scope.register(r1), cx.scope.register(r2))
    };

    initiator
        .apply(
            cx.scope,
            Value::undefined(),
            vec![Value::object(resolve), Value::object(reject)],
        )
        .root_err(cx.scope)?;

    Ok(Value::object(cx.scope.register(promise)))
}

pub fn resolve(cx: CallContext) -> Result<Value, Value> {
    let value = cx.args.first().unwrap_or_undefined();
    // TODO: do not wrap thenable values in another promise
    let promise = Promise::resolved(cx.scope, value);
    Ok(Value::object(cx.scope.register(promise)))
}

pub fn reject(cx: CallContext) -> Result<Value, Value> {
    let value = cx.args.first().unwrap_or_undefined();
    let promise = Promise::resolved(cx.scope, value);
    Ok(Value::object(cx.scope.register(promise)))
}

pub fn then(cx: CallContext) -> Result<Value, Value> {
    let this = cx.this.unpack();
    let promise = match this.downcast_ref::<Promise>(cx.scope) {
        Some(promise) => promise,
        None => throw!(cx.scope, TypeError, "Receiver must be a promise"),
    };

    let handler = match cx.args.first().unpack() {
        Some(ValueKind::Object(obj)) if matches!(obj.type_of(cx.scope), Typeof::Function) => obj,
        _ => throw!(cx.scope, TypeError, "Promise handler must be a function"),
    };

    let mut state = promise.state().borrow_mut();

    let then_promise = {
        let p = Promise::new(cx.scope);
        cx.scope.register(p)
    };
    let resolver = {
        let p = PromiseResolver::new(cx.scope, then_promise);
        cx.scope.register(p)
    };
    let then_task = {
        let t = ThenTask::new(cx.scope, then_promise, handler, resolver);
        cx.scope.register(t)
    };

    match &mut *state {
        PromiseState::Pending { resolve, .. } => resolve.push(then_task),
        PromiseState::Resolved(value) => {
            let bf = BoundFunction::new(cx.scope, then_task, None, Some(vec![*value]));
            let bf = cx.scope.register(bf);
            cx.scope.add_async_task(bf);
        }
        PromiseState::Rejected(..) => {}
    }

    Ok(Value::object(then_promise))
}

// TODO: Promise.prototype.catch

#[derive(Debug, Trace)]
struct ThenTask {
    // TODO: make a type like CastHandle<Promise> that implements Deref by downcasting
    then_promise: ObjectId,
    handler: ObjectId,
    resolver: ObjectId,
    obj: NamedObject,
}

impl ThenTask {
    pub fn new(vm: &Vm, then_promise: ObjectId, handler: ObjectId, resolver: ObjectId) -> Self {
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
        own_keys
    );

    fn apply(
        &self,
        scope: &mut crate::localscope::LocalScope,
        _callee: ObjectId,
        _this: Value,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        let resolved = args.first().unwrap_or_undefined();
        let ret = self
            .handler
            .apply(scope, Value::undefined(), vec![resolved])
            .root(scope)?;

        let ret_then = ret
            .get_property(scope, PropertyKey::String(sym::then.into()))?
            .root(scope);

        match ret_then.unpack() {
            ValueKind::Undefined(..) => {
                // Not a promise. Call resolver(value)
                let bf = BoundFunction::new(scope, self.resolver, None, Some(vec![ret]));
                let bf = scope.register(bf);
                scope.add_async_task(bf);
            }
            _ => {
                // Is a promise. Call value.then(resolver)
                ret_then.apply(scope, ret, vec![Value::object(self.resolver)])?;
            }
        }

        Ok(Value::undefined().into())
    }

    extract!(self);
}
