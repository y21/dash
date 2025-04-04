use dash_proc_macro::Trace;

use crate::frame::This;
use crate::gc::ObjectId;
use crate::localscope::LocalScope;
use crate::value::object::{NamedObject, Object};
use crate::value::promise::{Promise, PromiseState, wrap_resolved_promise};
use crate::value::propertykey::ToPropertyKey;
use crate::value::root_ext::RootErrExt;
use crate::value::{Root, Typeof, Unpack, Unrooted, Value, ValueContext};
use crate::{PromiseAction, Vm, delegate, extract, throw};
use dash_middle::interner::sym;

use super::args::CallArgs;
use super::bound::BoundFunction;
use super::generator::{GeneratorFunction, GeneratorIterator, GeneratorState};
use super::user::UserFunction;

#[derive(Debug, Trace)]
pub struct AsyncFunction {
    /// The properties of generator functions are very similar to async functions, so we can build upon generators
    pub inner: GeneratorFunction,
}

impl AsyncFunction {
    pub fn new(fun: UserFunction) -> Self {
        Self {
            inner: GeneratorFunction::new(fun),
        }
    }

    pub(crate) fn handle_function_call(
        &self,
        scope: &mut LocalScope,
        callee: ObjectId,
        this: This,
        args: CallArgs,
        new_target: Option<ObjectId>,
    ) -> Result<Value, Unrooted> {
        let generator_iter = self.inner.handle_function_call(scope, callee, this, args, new_target)?;

        let result = scope
            .statics
            .generator_iterator_next
            .clone()
            .apply(This::Bound(generator_iter), CallArgs::empty(), scope)
            .root(scope)
            .and_then(|result| result.get_property(sym::value.to_key(scope), scope).root(scope));

        match result {
            Ok(value) => {
                let is_done = match generator_iter.unpack().downcast_ref::<GeneratorIterator>(scope) {
                    Some(it) => matches!(*it.state().borrow(), GeneratorState::Finished),
                    None => throw!(scope, TypeError, "Incompatible receiver"),
                };

                if is_done {
                    // Promise in resolved state
                    let promise = wrap_resolved_promise(scope, value);
                    Ok(promise)
                } else {
                    // Promise in pending state
                    let final_promise = scope.register(Promise::new(scope));
                    let promise = Value::object(final_promise);
                    let (resolve, reject) = ThenTask::resolve_reject_pair(scope, generator_iter, final_promise);

                    scope
                        .statics
                        .promise_then
                        .clone()
                        .apply(
                            This::Bound(match result {
                                Ok(value) => value,
                                Err(value) => value,
                            }),
                            [Value::object(resolve), Value::object(reject)].into(),
                            scope,
                        )
                        .root_err(scope)?;

                    Ok(promise)
                }
            }
            Err(value) => Err(value.into()),
        }
    }
}

/// A callable object that is passed to `.then()` on awaited promises.
/// Calling this will drive the async function to the next await or return point, either by resolving or rejecting it.
#[derive(Debug, Trace)]
pub struct ThenTask {
    /// The inner generator iterator of the async function
    generator_iter: Value,
    final_promise: ObjectId,
    obj: NamedObject,
    action: PromiseAction,
}

impl ThenTask {
    pub fn new(vm: &Vm, generator_iter: Value, final_promise: ObjectId, action: PromiseAction) -> Self {
        Self {
            generator_iter,
            obj: NamedObject::new(vm),
            final_promise,
            action,
        }
    }
    pub fn resolve_reject_pair(
        scope: &mut LocalScope<'_>,
        generator_iter: Value,
        final_promise: ObjectId,
    ) -> (ObjectId, ObjectId) {
        let resolve = scope.register(Self::new(scope, generator_iter, final_promise, PromiseAction::Resolve));
        let reject = scope.register(Self::new(scope, generator_iter, final_promise, PromiseAction::Reject));
        (resolve, reject)
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
        scope: &mut LocalScope,
    ) -> Result<Unrooted, Unrooted> {
        let promise_value = args.first().unwrap_or_undefined();

        // Call GeneratorIterator.prototype.(next|throw) on the generator of async function
        let progress_fn = match self.action {
            PromiseAction::Resolve => scope.statics.generator_iterator_next,
            PromiseAction::Reject => scope.statics.generator_iterator_throw,
        };
        // TODO: this probably wont work because when it gets to an await point, the generator doesnt know how to handle it
        let value = progress_fn
            .apply(This::Bound(self.generator_iter), [promise_value].into(), scope)
            .root(scope)
            .and_then(|result| result.get_property(sym::value.to_key(scope), scope).root(scope));

        // - Repeat what we are doing above.
        // Check if generator iterator is done:
        //   - If yes, we can resolve the inner promise.
        //   - If no, promisify value and attach then handler
        // If value is an error, we can reject promise.

        match value {
            Ok(value) => {
                let is_done = match self.generator_iter.unpack().downcast_ref::<GeneratorIterator>(scope) {
                    Some(it) => matches!(*it.state().borrow(), GeneratorState::Finished),
                    None => throw!(scope, TypeError, "Incompatible receiver"),
                };

                if is_done {
                    // Promise in resolved state
                    // TODO: value might be a promise
                    scope.drive_promise(
                        PromiseAction::Resolve,
                        self.final_promise.extract::<Promise>(scope).unwrap(),
                        self.final_promise,
                        [value].into(),
                    );
                } else {
                    let (resolver, rejecter) =
                        Self::resolve_reject_pair(scope, self.generator_iter, self.final_promise);
                    let value = wrap_resolved_promise(scope, value);

                    if let Some(promise) = value.extract::<Promise>(scope) {
                        match &mut *promise.state().borrow_mut() {
                            PromiseState::Pending { resolve, reject } => {
                                resolve.push(resolver);
                                reject.push(rejecter);
                            }
                            PromiseState::Resolved(value) => {
                                let bf = BoundFunction::new(scope, resolver, None, [*value].into());
                                let bf = scope.register(bf);
                                scope.add_async_task(bf);
                            }
                            PromiseState::Rejected { value, caught } => {
                                *caught = true;
                                let bf = BoundFunction::new(scope, rejecter, None, [*value].into());
                                let bf = scope.register(bf);
                                scope.add_async_task(bf);
                            }
                        }
                    }
                }
            }
            Err(value) => {
                // Promise in rejected state
                scope.drive_promise(
                    PromiseAction::Reject,
                    self.final_promise.extract::<Promise>(scope).unwrap(),
                    self.final_promise,
                    [value].into(),
                );
            }
        }

        Ok(Value::undefined().into())
    }

    fn type_of(&self, _: &Vm) -> Typeof {
        Typeof::Function
    }

    extract!(self);
}
