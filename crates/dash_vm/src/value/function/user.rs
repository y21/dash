use std::rc::Rc;

use dash_middle::compiler::constant::Function;
use dash_proc_macro::Trace;

use crate::dispatch::HandleResult;
use crate::frame::Frame;
use crate::gc::handle::Handle;
use crate::localscope::LocalScope;
use crate::value::ExternalValue;
use crate::value::Value;

use super::extend_stack_from_args;

#[derive(Debug, Clone, Trace)]
pub struct UserFunction {
    inner: Rc<Function>,
    externals: Rc<[Handle<ExternalValue>]>,
}

impl UserFunction {
    pub fn new(inner: Rc<Function>, externals: Rc<[Handle<ExternalValue>]>) -> Self {
        Self { inner, externals }
    }

    pub fn externals(&self) -> &Rc<[Handle<ExternalValue>]> {
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

        extend_stack_from_args(args, self.inner.params, scope, self.inner.rest_local.is_some());

        let mut frame = Frame::from_function(Some(this), self, is_constructor_call, false);
        frame.set_sp(sp);

        match scope.execute_frame(frame) {
            Ok(v) => Ok(v),
            Err(err) => Err(err.root(scope)),
        }
    }
}
