use dash_middle::interner::Symbol;

use crate::gc::ObjectId;
use crate::localscope::LocalScope;
use crate::value::Value;

use super::args::CallArgs;
use super::{Function, FunctionKind};

// TODO: return Unrooted?
pub type NativeFunction = fn(cx: CallContext) -> Result<Value, Value>;

pub fn register_native_fn(sc: &mut LocalScope<'_>, name: Symbol, fun: NativeFunction) -> ObjectId {
    let fun = Function::new(sc, Some(name.into()), FunctionKind::Native(fun));
    sc.register(fun)
}

#[derive(Debug)]
pub struct CallContext<'s, 'c> {
    pub args: CallArgs,
    pub scope: &'c mut LocalScope<'s>,
    pub this: Value,
    pub new_target: Option<ObjectId>,
}
