use std::{
    any::Any,
    fmt::{self, Debug},
};

use crate::{
    gc::trace::Trace,
    vm::{frame::Frame, local::LocalScope, Vm},
};

use self::{
    native::{CallContext, NativeFunction},
    user::UserFunction,
};

use super::{
    object::{NamedObject, Object},
    Value,
};

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
    obj: NamedObject,
}

impl Function {
    pub fn new(vm: &mut Vm, name: Option<String>, kind: FunctionKind) -> Self {
        Self {
            name,
            kind,
            obj: NamedObject::new(vm),
        }
    }

    pub fn with_obj(name: Option<String>, kind: FunctionKind, obj: NamedObject) -> Self {
        Self { name, kind, obj }
    }
}

unsafe impl Trace for Function {
    fn trace(&self) {}
}

impl Object for Function {
    fn get_property(&self, sc: &mut LocalScope, key: &str) -> Result<super::Value, super::Value> {
        self.obj.get_property(sc, key)
    }

    fn set_property(
        &self,
        sc: &mut LocalScope,
        key: &str,
        value: super::Value,
    ) -> Result<(), super::Value> {
        self.obj.set_property(sc, key, value)
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

                let mut frame = Frame::from_function(uf, scope);
                frame.sp = sp;

                scope.vm.execute_frame(frame)
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: super::Value) -> Result<(), Value> {
        self.obj.set_prototype(sc, value)
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        self.obj.get_prototype(sc)
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        Ok(["length", "name"]
            .iter()
            .map(|&s| Value::String(s.into()))
            .collect())
    }
}
