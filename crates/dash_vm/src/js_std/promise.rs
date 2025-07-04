use dash_proc_macro::Trace;

use crate::gc::ObjectId;
use crate::localscope::LocalScope;
use crate::value::function::args::CallArgs;
use crate::value::function::bound::BoundFunction;
use crate::value::function::native::CallContext;
use crate::value::object::{Object, OrdObject, This};
use crate::value::promise::{Promise, PromiseRejecter, PromiseResolver, PromiseState};
use crate::value::propertykey::ToPropertyKey;
use crate::value::root_ext::RootErrExt;
use crate::value::{Root, Typeof, Unpack, Unrooted, Value, ValueContext, ValueKind};
use crate::{Vm, delegate, extract, throw};
use dash_middle::interner::sym;

use super::receiver_t;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let initiator = match cx.args.first() {
        Some(v) if matches!(v.type_of(cx.scope), Typeof::Function) => v,
        _ => throw!(cx.scope, TypeError, "Promise callback must be a function"),
    };

    let Some(new_target) = cx.new_target else {
        throw!(cx.scope, TypeError, "Promise constructor requires new")
    };

    let promise = Promise::with_obj(OrdObject::instance_for_new_target(new_target, cx.scope)?);
    let promise = cx.scope.register(promise);

    let (resolve, reject) = {
        let r1 = PromiseResolver::new(cx.scope, promise);
        let r2 = PromiseRejecter::new(cx.scope, promise);
        (cx.scope.register(r1), cx.scope.register(r2))
    };

    initiator
        .apply(
            This::default(),
            [Value::object(resolve), Value::object(reject)].into(),
            cx.scope,
        )
        .root_err(cx.scope)?;

    Ok(Value::object(cx.scope.register(promise)))
}

pub fn resolve(cx: CallContext) -> Result<Value, Value> {
    let value = cx.args.first().unwrap_or_undefined();
    if value.extract::<Promise>(cx.scope).is_some() {
        Ok(value)
    } else {
        let promise = Promise::resolved(cx.scope, value);
        Ok(Value::object(cx.scope.register(promise)))
    }
}

pub fn reject(cx: CallContext) -> Result<Value, Value> {
    let value = cx.args.first().unwrap_or_undefined();
    if value.extract::<Promise>(cx.scope).is_some() {
        Ok(value)
    } else {
        let promise = Promise::rejected(cx.scope, value);
        Ok(Value::object(promise))
    }
}

pub fn then(cx: CallContext) -> Result<Value, Value> {
    let promise = receiver_t::<Promise>(cx.scope, &cx.this, "Promise.prototype.then")?;

    let fulfill_handler = match cx.args.first().unpack() {
        Some(ValueKind::Object(obj)) if matches!(obj.type_of(cx.scope), Typeof::Function) => Some(obj),
        _ => None,
    };
    let reject_handler = match cx.args.get(1).unpack() {
        Some(ValueKind::Object(obj)) if matches!(obj.type_of(cx.scope), Typeof::Function) => Some(obj),
        _ => None,
    };

    let mut state = promise.state().borrow_mut();

    let then_promise = cx.scope.mk_promise();
    let resolver = cx.scope.register(PromiseResolver::new(cx.scope, then_promise));
    let fulfill_handler = fulfill_handler.map(|handler| {
        cx.scope
            .register(ThenTask::new(cx.scope, then_promise, handler, resolver))
    });
    let reject_handler = reject_handler.map(|handler| {
        cx.scope
            .register(ThenTask::new(cx.scope, then_promise, handler, resolver))
    });

    match &mut *state {
        PromiseState::Pending { resolve, reject } => {
            if let Some(handler) = fulfill_handler {
                resolve.push(handler);
            }
            if let Some(handler) = reject_handler {
                reject.push(handler);
            }
        }
        PromiseState::Resolved(value) => {
            if let Some(handler) = fulfill_handler {
                let bf = BoundFunction::new(cx.scope, handler, None, [*value].into());
                let bf = cx.scope.register(bf);
                cx.scope.add_async_task(bf);
            }
        }
        PromiseState::Rejected { value, caught } => {
            if let Some(handler) = reject_handler {
                *caught = true;
                let bf = BoundFunction::new(cx.scope, handler, None, [*value].into());
                let bf = cx.scope.register(bf);
                cx.scope.add_async_task(bf);
            }
        }
    }

    Ok(Value::object(then_promise))
}

#[derive(Debug, Trace)]
struct ThenTask {
    then_promise: ObjectId,
    handler: ObjectId,
    resolver: ObjectId,
    obj: OrdObject,
}

impl ThenTask {
    pub fn new(vm: &Vm, then_promise: ObjectId, handler: ObjectId, resolver: ObjectId) -> Self {
        Self {
            then_promise,
            handler,
            resolver,
            obj: OrdObject::new(vm),
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
        _callee: ObjectId,
        _this: This,
        args: CallArgs,
        scope: &mut LocalScope<'_>,
    ) -> Result<Unrooted, Unrooted> {
        let resolved = args.first().unwrap_or_undefined();
        let ret = self
            .handler
            .apply(This::default(), [resolved].into(), scope)
            .root(scope)?;

        let ret_then = ret
            .into_option()
            .map(|ret| ret.get_property(sym::then.to_key(scope), scope))
            .transpose()?
            .root(scope)
            .unwrap_or_undefined();

        match ret_then.unpack() {
            ValueKind::Undefined(..) => {
                // Not a promise. Call resolver(value)
                let bf = BoundFunction::new(scope, self.resolver, None, [ret].into());
                let bf = scope.register(bf);
                scope.add_async_task(bf);
            }
            _ => {
                // Is a promise. Call value.then(resolver)
                ret_then.apply(This::bound(ret), [Value::object(self.resolver)].into(), scope)?;
            }
        }

        Ok(Value::undefined().into())
    }

    extract!(self);
}
