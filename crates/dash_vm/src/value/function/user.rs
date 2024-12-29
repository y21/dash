use std::rc::Rc;

use dash_middle::compiler::constant::Function;
use dash_proc_macro::Trace;

use crate::dispatch::HandleResult;
use crate::frame::{Frame, This};
use crate::gc::ObjectId;
use crate::localscope::LocalScope;
use crate::value::arguments::Arguments;
use crate::value::{ExternalValue, Root, Value};

use super::args::CallArgs;
use super::extend_stack_from_args;

#[derive(Debug, Clone, Trace)]
pub struct UserFunction {
    inner: Rc<Function>,
    externals: Rc<[ExternalValue]>,
}

impl UserFunction {
    pub fn new(inner: Rc<Function>, externals: Rc<[ExternalValue]>) -> Self {
        Self { inner, externals }
    }

    pub fn externals(&self) -> &Rc<[ExternalValue]> {
        &self.externals
    }

    pub fn inner(&self) -> &Rc<Function> {
        &self.inner
    }

    pub(crate) fn handle_function_call(
        &self,
        scope: &mut LocalScope,
        this: This,
        args: CallArgs,
        new_target: Option<ObjectId>,
    ) -> Result<HandleResult, Value> {
        let sp = scope.stack.len();

        let mut arguments = None;
        if self.inner.references_arguments {
            let args = Arguments::new(scope, args.iter().cloned());
            let args = scope.register(args);
            arguments = Some(args);
        }

        extend_stack_from_args(args, self.inner.params, scope, self.inner.rest_local.is_some());

        let mut frame = Frame::from_function(this, self, new_target, false, arguments);
        frame.set_sp(sp);

        match scope.execute_frame(frame) {
            Ok(v) => Ok(v),
            Err(err) => Err(err.root(scope)),
        }
    }
}
