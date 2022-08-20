use crate::dispatch::HandleResult;
use crate::local::LocalScope;
use crate::throw;
use crate::value::promise::Promise;
use crate::value::promise::PromiseState;
use crate::value::Value;

use super::generator::GeneratorFunction;
use super::user::UserFunction;

#[derive(Debug)]
pub struct AsyncFunction {
    // The properties of generator functions are very similar to async functions, so we can build upon those
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
        this: Value,
        args: Vec<Value>,
        is_constructor_call: bool,
    ) -> Result<Value, Value> {
        let result = self
            .inner
            .function
            .handle_function_call(scope, this, args, is_constructor_call);

        let promise = Promise::new(scope);
        let mut promise_state = promise.state().borrow_mut();

        match result {
            Ok(HandleResult::Return(value)) => {
                // Create Promise in resolved state.
                *promise_state = PromiseState::Resolved(value);
            }
            Ok(HandleResult::Await(..)) => {
                // Create Promise in pending state.
                todo!("Handle pending promise")
            }
            Ok(HandleResult::Yield(..)) => {
                throw!(scope, "Cannot `yield` in async function")
            }
            Err(err) => {
                // Create Promise in rejected state.
                *promise_state = PromiseState::Rejected(err);
            }
        }

        drop(promise_state);
        Ok(Value::Object(scope.register(promise)))
    }
}
