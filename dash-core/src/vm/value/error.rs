use crate::gc::trace::Trace;

use super::object::Object;

#[derive(Debug)]
pub struct Error {
    pub name: String,
    pub message: String,
}

impl Error {
    pub fn new<S: Into<String>>(message: S) -> Self {
        Self {
            name: String::from("Error"),
            message: message.into(),
        }
    }

    pub fn with_name<S1: Into<String>, S2: Into<String>>(name: S1, message: S2) -> Self {
        Self {
            name: name.into(),
            message: message.into(),
        }
    }
}

unsafe impl Trace for Error {
    fn trace(&self) {}
}

impl Object for Error {
    fn get_property(
        &self,
        sc: &mut crate::vm::local::LocalScope,
        key: &str,
    ) -> Result<super::Value, super::Value> {
        todo!()
    }

    fn set_property(
        &self,
        sc: &mut crate::vm::local::LocalScope,
        key: &str,
        value: super::Value,
    ) -> Result<super::Value, super::Value> {
        todo!()
    }

    fn apply<'s>(
        &self,
        scope: &mut crate::vm::local::LocalScope,
        this: super::Value,
        args: Vec<super::Value>,
    ) -> Result<super::Value, super::Value> {
        todo!()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        todo!()
    }
}
