use dash_proc_macro::Trace;

use crate::delegate;
use crate::gc::handle::Handle;
use crate::local::LocalScope;
use crate::value::object::NamedObject;
use crate::value::object::Object;
use crate::value::object::PropertyKey;
use crate::value::promise::wrap_promise;
use crate::value::promise::Promise;
use crate::value::Typeof;
use crate::value::Value;
use crate::value::ValueContext;
use crate::PromiseAction;
use crate::Vm;

use super::generator::as_generator;
use super::generator::GeneratorFunction;
use super::generator::GeneratorState;
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
        callee: Handle<dyn Object>,
        this: Value,
        args: Vec<Value>,
        is_constructor_call: bool,
    ) -> Result<Value, Value> {
        let generator_iter = self
            .inner
            .handle_function_call(scope, callee, this, args, is_constructor_call)?;

        let result = scope
            .statics
            .generator_iterator_next
            .clone()
            .apply(scope, generator_iter.clone(), Vec::new())
            .and_then(|result| result.get_property(scope, PropertyKey::String("value".into())));

        match &result {
            Ok(value) => {
                let is_done = as_generator(scope, &generator_iter)
                    .map(|gen| matches!(&*gen.state().borrow(), GeneratorState::Finished))?;

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

                    let promise = Value::Object(final_promise);

                    scope.statics.promise_then.clone().apply(
                        scope,
                        match result {
                            Ok(value) => value,
                            Err(value) => value,
                        },
                        vec![Value::Object(then_task)],
                    )?;

                    Ok(promise)
                }
            }
            Err(value) => {
                let promise = wrap_promise(scope, value.clone());
                Err(promise)
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
    final_promise: Handle<dyn Object>,
    obj: NamedObject,
}

impl ThenTask {
    pub fn new(vm: &mut Vm, generator_iter: Value, final_promise: Handle<dyn Object>) -> Self {
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
        scope: &mut crate::local::LocalScope,
        _callee: Handle<dyn Object>,
        _this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        let promise_value = args.first().unwrap_or_undefined();

        // Call GeneratorIterator.prototype.next on the generator of async function
        // TODO: this probably wont work because when it gets to an await point, the generator doesnt know how to handle it
        let value = scope
            .statics
            .generator_iterator_next
            .clone()
            .apply(scope, self.generator_iter.clone(), vec![promise_value])
            .and_then(|result| result.get_property(scope, PropertyKey::String("value".into())));

        // - Repeat what we are doing above.
        // Check if generator iterator is done:
        //   - If yes, we can resolve the inner promise.
        //   - If no, promisify value and attach then handler
        // If value is an error, we can reject promise.

        match value {
            Ok(value) => {
                let is_done = as_generator(scope, &self.generator_iter)
                    .map(|gen| matches!(&*gen.state().borrow(), GeneratorState::Finished))?;

                if is_done {
                    // Promise in resolved state
                    // TODO: value might be a promise
                    scope.drive_promise(
                        PromiseAction::Resolve,
                        self.final_promise.as_any().downcast_ref::<Promise>().unwrap(),
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
                        .apply(scope, value, vec![Value::Object(then_task)])?;
                }
            }
            Err(value) => {
                // Promise in rejected state
                scope.drive_promise(
                    PromiseAction::Reject,
                    self.final_promise.as_any().downcast_ref::<Promise>().unwrap(),
                    vec![value],
                );
            }
        }

        Ok(Value::undefined())
    }

    fn type_of(&self) -> Typeof {
        Typeof::Function
    }
}
