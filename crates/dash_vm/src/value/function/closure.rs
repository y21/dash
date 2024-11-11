use dash_proc_macro::Trace;

use crate::dispatch::HandleResult;
use crate::frame::This;
use crate::localscope::LocalScope;
use crate::value::{Unrooted, Value};

use super::user::UserFunction;

#[derive(Trace, Debug)]
pub struct Closure {
    pub fun: UserFunction,
    pub this: This,
}

impl Closure {
    pub(crate) fn handle_function_call(
        &self,
        scope: &mut LocalScope,
        _this: This,
        args: Vec<Value>,
        is_constructor_call: bool,
    ) -> Result<Unrooted, Unrooted> {
        let ret = self
            .fun
            .handle_function_call(scope, self.this, args, is_constructor_call)?;

        Ok(match ret {
            HandleResult::Return(v) => v,
            HandleResult::Yield(_) | HandleResult::Await(_) => unreachable!("closure cannot yield or await"),
        })
    }
}
