use std::fmt::{self, Debug};

use crate::{
    gc::trace::Trace,
    vm::{local::LocalScope, Vm},
};

use self::native::{CallContext, NativeFunction};

use super::object::Object;

pub mod native;

pub enum FunctionKind {
    Native(NativeFunction),
    User,
}

impl fmt::Debug for FunctionKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Native(_) => f.write_str("NativeFunction"),
            Self::User => f.write_str("UserFunction"),
        }
    }
}

#[derive(Debug)]
pub struct Function {
    name: String,
    kind: FunctionKind,
}

impl Function {
    pub fn new(name: String, kind: FunctionKind) -> Self {
        Self { name, kind }
    }
}

unsafe impl Trace for Function {
    fn trace(&self) {}
}

impl Object for Function {
    fn get_property(&self, vm: &mut Vm, key: &str) -> Result<super::Value, super::Value> {
        todo!()
    }

    fn set_property(
        &self,
        vm: &mut Vm,
        key: &str,
        value: super::Value,
    ) -> Result<super::Value, super::Value> {
        todo!()
    }

    fn apply(
        &self,
        vm: &mut Vm,
        this: super::Value,
        args: Vec<super::Value>,
    ) -> Result<super::Value, super::Value> {
        match self.kind {
            FunctionKind::Native(native) => {
                let scope = LocalScope::new(vm);
                let cx = CallContext { args, scope };
                let result = native(cx);
                result
            }
            _ => unimplemented!(),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
