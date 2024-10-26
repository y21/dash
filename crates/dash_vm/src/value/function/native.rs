use dash_middle::interner::Symbol;

use crate::gc::ObjectId;
use crate::localscope::LocalScope;
use crate::value::Value;

use super::{Function, FunctionKind};

// TODO: return Unrooted?
pub type NativeFunction = fn(cx: CallContext) -> Result<Value, Value>;

pub fn register_native_fn(sc: &mut LocalScope<'_>, name: Symbol, fun: NativeFunction) -> ObjectId {
    let fun = Function::new(sc, Some(name.into()), FunctionKind::Native(fun));
    sc.register(fun)
}

#[derive(Debug)]
pub struct CallContext<'s, 'c> {
    pub args: Vec<Value>,
    pub scope: &'c mut LocalScope<'s>,
    pub this: Value,
    pub is_constructor_call: bool,
}

impl<'s, 'c> CallContext<'s, 'c> {
    pub fn constructor(args: Vec<Value>, scope: &'c mut LocalScope<'s>, this: Value) -> Self {
        Self {
            args,
            scope,
            this,
            is_constructor_call: true,
        }
    }

    pub fn call(args: Vec<Value>, scope: &'c mut LocalScope<'s>, this: Value) -> Self {
        Self {
            args,
            scope,
            this,
            is_constructor_call: false,
        }
    }
}
