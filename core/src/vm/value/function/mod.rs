use std::fmt::Debug;

use self::native::{CallContext, NativeFunction};

use super::object::Object;

pub mod native;

#[derive(Debug)]
pub enum FunctionKind {
    Native(NativeFunction),
    User,
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

impl Object for Function {
    fn get_property(&self, key: &str) -> Result<super::Value, super::Value> {
        todo!()
    }

    fn set_property(&self, key: &str, value: super::Value) -> Result<super::Value, super::Value> {
        todo!()
    }

    fn apply(
        &self,
        this: super::Value,
        args: Vec<super::Value>,
    ) -> Result<super::Value, super::Value> {
        match self.kind {
            FunctionKind::Native(native) => {
                let cx = CallContext { args };
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
