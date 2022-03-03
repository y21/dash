use std::fmt::{self, Debug};

use crate::{
    gc::trace::Trace,
    vm::{frame::Frame, local::LocalScope},
};

use self::{
    native::{CallContext, NativeFunction},
    user::UserFunction,
};

use super::object::Object;

pub mod native;
pub mod user;

pub enum FunctionKind {
    Native(NativeFunction),
    User(UserFunction),
}

impl fmt::Debug for FunctionKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Native(_) => f.write_str("NativeFunction"),
            Self::User(_) => f.write_str("UserFunction"),
        }
    }
}

#[derive(Debug)]
pub struct Function {
    name: Option<String>,
    kind: FunctionKind,
}

impl Function {
    pub fn new(name: Option<String>, kind: FunctionKind) -> Self {
        Self { name, kind }
    }
}

unsafe impl Trace for Function {
    fn trace(&self) {}
}

impl Object for Function {
    fn get_property(&self, sc: &mut LocalScope, key: &str) -> Result<super::Value, super::Value> {
        todo!()
    }

    fn set_property(
        &self,
        sc: &mut LocalScope,
        key: &str,
        value: super::Value,
    ) -> Result<super::Value, super::Value> {
        todo!()
    }

    fn apply(
        &self,
        scope: &mut LocalScope,
        this: super::Value,
        args: Vec<super::Value>,
    ) -> Result<super::Value, super::Value> {
        match &self.kind {
            FunctionKind::Native(native) => {
                let cx = CallContext { args, scope, this };
                let result = native(cx);
                result
            }
            FunctionKind::User(uf) => {
                let sp = scope.stack.len();

                let argc = std::cmp::min(uf.params(), args.len());

                scope.stack.extend(args.into_iter().rev().take(argc));

                let mut frame = Frame::from(uf);
                frame.sp = sp;

                scope.vm.execute_frame(frame)
            }
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
