use std::iter;
use std::rc::Rc;

use dash_middle::compiler::constant::Function;

use crate::dispatch::HandleResult;
use crate::frame::Frame;
use crate::gc::handle::Handle;
use crate::local::LocalScope;
use crate::value::array::Array;
use crate::value::object::Object;
use crate::value::object::PropertyValue;
use crate::value::Value;

#[derive(Debug, Clone)]
pub struct UserFunction {
    inner: Rc<Function>,
    externals: Rc<[Handle<dyn Object>]>,
}

impl UserFunction {
    pub fn new(inner: Rc<Function>, externals: Rc<[Handle<dyn Object>]>) -> Self {
        Self { inner, externals }
    }

    pub fn externals(&self) -> &Rc<[Handle<dyn Object>]> {
        &self.externals
    }

    pub fn inner(&self) -> &Rc<Function> {
        &self.inner
    }

    pub(crate) fn handle_function_call(
        &self,
        scope: &mut LocalScope,
        this: Value,
        args: Vec<Value>,
        is_constructor_call: bool,
    ) -> Result<HandleResult, Value> {
        let sp = scope.stack.len();

        // Insert at most [param_count] amount of provided arguments on the stack
        // In the compiler we allocate local space for every parameter
        let param_count = self.inner.params;
        scope.stack.extend(args.iter().take(param_count).cloned());

        // Insert undefined values for parameters without a value
        if param_count > args.len() {
            scope
                .stack
                .extend(iter::repeat(Value::undefined()).take(param_count - args.len()));
        }

        // Finally insert Value::Object([]) if this function uses the rest operator
        if self.inner.rest_local.is_some() {
            let args = args
                .get(param_count..)
                .map(|s| s.iter().cloned().map(PropertyValue::Static).collect())
                .unwrap_or_default();

            let array = Array::from_vec(scope, args);
            let array = scope.register(array);
            scope.stack.push(Value::Object(array));
        }

        let mut frame = Frame::from_function(Some(this), self, is_constructor_call);
        frame.set_sp(sp);

        scope.execute_frame(frame)
    }
}
