use dash_proc_macro::Trace;

use crate::gc::ObjectId;
use crate::localscope::LocalScope;
use crate::value::object::{NamedObject, Object, PropertyKey};
use crate::value::promise::{wrap_promise, Promise};
use crate::value::root_ext::RootErrExt;
use crate::value::{Root, Typeof, Unpack, Unrooted, Value, ValueContext};
use crate::{delegate, throw, PromiseAction, Vm};
use dash_middle::interner::sym;

use super::generator::{GeneratorFunction, GeneratorIterator, GeneratorState};
use super::user::UserFunction;

#[derive(Debug, Trace)]
pub struct AsyncFunction {
    /// The properties of generator functions are very similar to async functions, so we can build upon generators
    inner: GeneratorFunction,
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
        this: Value,
        args: Vec<Value>,
        is_constructor_call: bool,
    ) -> Result<Value, Unrooted> {
        let generator_iter = self
            .inner
            .handle_function_call(scope, callee, this, args, is_constructor_call)?;

        let result = scope
            .statics
            .generator_iterator_next
            .clone()
            .apply(scope, generator_iter.clone(), Vec::new())
            .root(scope)
            .and_then(|result| {
                result
                    .get_property(scope, PropertyKey::String(sym::value.into()))
                    .root(scope)
            });

        match &result {
            Ok(value) => {
                let is_done = match generator_iter.unpack().downcast_ref::<GeneratorIterator>(scope) {
                    Some(it) => matches!(*it.state().borrow(), GeneratorState::Finished),
                    None => throw!(scope, TypeError, "Incompatible receiver"),
                };

                if is_done {
                    // Promise in resolved state
                    let promise = wrap_promise(scope, value.clone());
                    Ok(promise)
                } else {
                    // Promise in pending state
                    let final_promise = Promise::new(scope);
                    let final_promise = scope.register(final_promise);
                    let then_task = ThenTask::new(scope, generator_iter.clone(), final_promise.clone());
                    let then_task = scope.register(then_task);

                    let promise = Value::object(final_promise);

                    scope
                        .statics
                        .promise_then
                        .clone()
                        .apply(
                            scope,
                            match result {
                                Ok(value) => value,
                                Err(value) => value,
                            },
                            vec![Value::object(then_task)],
                        )
                        .root_err(scope)?;

                    Ok(promise)
                }
            }
            Err(value) => {
                let promise = wrap_promise(scope, value.clone());
                Err(promise.into())
            }
        }
    }

    pub fn inner(&self) -> &GeneratorFunction {
        &self.inner
    }
}

/// A callable object that is passed to `.then()` on awaited promises.
/// Calling this will drive the async function to the next await or return point.
#[derive(Debug, Trace)]
pub struct ThenTask {
    /// The inner generator iterator of the async function
    generator_iter: Value,
    final_promise: ObjectId,
    obj: NamedObject,
}

impl ThenTask {
    pub fn new(vm: &Vm, generator_iter: Value, final_promise: ObjectId) -> Self {
        Self {
            generator_iter,
            obj: NamedObject::new(vm),
            final_promise,
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
        scope: &mut crate::localscope::LocalScope,
        _callee: ObjectId,
        _this: Value,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        let promise_value = args.first().unwrap_or_undefined();

        // Call GeneratorIterator.prototype.next on the generator of async function
        // TODO: this probably wont work because when it gets to an await point, the generator doesnt know how to handle it
        let value = scope
            .statics
            .generator_iterator_next
            .clone()
            .apply(scope, self.generator_iter.clone(), vec![promise_value])
            .root(scope)
            .and_then(|result| {
                result
                    .get_property(scope, PropertyKey::String(sym::value.into()))
                    .root(scope)
            });

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
                        self.final_promise.as_any(scope).downcast_ref::<Promise>().unwrap(),
                        vec![value],
                    );
                } else {
                    let then_task = ThenTask::new(scope, self.generator_iter.clone(), self.final_promise.clone());
                    let then_task = scope.register(then_task);
                    let value = wrap_promise(scope, value);

                    scope
                        .statics
                        .promise_then
                        .clone()
                        .apply(scope, value, vec![Value::object(then_task)])?;
                }
            }
            Err(value) => {
                // Promise in rejected state
                scope.drive_promise(
                    PromiseAction::Reject,
                    self.final_promise.as_any(scope).downcast_ref::<Promise>().unwrap(),
                    vec![value],
                );
            }
        }

        Ok(Value::undefined().into())
    }

    fn type_of(&self, _: &Vm) -> Typeof {
        Typeof::Function
    }
}
