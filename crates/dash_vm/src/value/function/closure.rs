use dash_proc_macro::Trace;

use crate::dispatch::HandleResult;
use crate::gc::ObjectId;
use crate::localscope::LocalScope;
use crate::value::Unrooted;
use crate::value::object::This;

use super::args::CallArgs;
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
        args: CallArgs,
        new_target: Option<ObjectId>,
    ) -> Result<Unrooted, Unrooted> {
        let ret = self.fun.handle_function_call(scope, self.this, args, new_target)?;

        Ok(match ret {
            HandleResult::Return(v) => v,
            HandleResult::Yield(_) | HandleResult::Await(_) => unreachable!("closure cannot yield or await"),
        })
    }
}
