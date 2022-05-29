use crate::local::LocalScope;
use crate::value::Value;

pub type NativeFunction = fn(cx: CallContext) -> Result<Value, Value>;

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
