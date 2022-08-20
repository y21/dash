use crate::local::LocalScope;
use crate::value::Value;

use super::generator::GeneratorFunction;

#[derive(Debug)]
pub struct AsyncFunction {
    // The properties of generator functions are very similar to async functions, so we can build upon those
    inner: GeneratorFunction,
}

impl AsyncFunction {
    pub(crate) fn handle_function_call(
        &self,
        scope: &mut LocalScope,
        this: Value,
        args: Vec<Value>,
        is_constructor_call: bool,
    ) -> Result<Value, Value> {
        self.inner
            .function
            .handle_function_call(scope, this, args, is_constructor_call)
    }
}
