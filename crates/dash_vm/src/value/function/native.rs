use dash_middle::interner::Symbol;

use crate::gc::ObjectId;
use crate::localscope::LocalScope;
use crate::value::Value;

use super::args::CallArgs;
use super::{Function, FunctionKind};

// TODO: return Unrooted?
pub type NativeFunction = fn(cx: CallContext) -> Result<Value, Value>;

pub fn register_native_fn(
    sc: &mut LocalScope<'_>,
    name: Symbol,
    constructable: bool,
    function: NativeFunction,
) -> ObjectId {
    let fun = Function::new(
        sc,
        Some(name.into()),
        FunctionKind::Native {
            function,
            constructable,
        },
    );
    let fun = sc.register(fun);
    fun.extract::<Function>(sc)
        .expect("registered native function must be a Function")
        .set_self_object_id(fun);
    fun
}

#[derive(Debug)]
pub struct CallContext<'s, 'c> {
    pub args: CallArgs,
    pub scope: &'c mut LocalScope<'s>,
    pub this: Value,
    pub new_target: Option<ObjectId>,
}
