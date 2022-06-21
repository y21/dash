use std::rc::Rc;

use dash_middle::compiler::constant::Function;

use crate::gc::handle::Handle;
use crate::value::object::Object;

#[derive(Debug, Clone)]
pub struct UserFunction {
    inner: Rc<Function>,
    externals: Rc<[Handle<dyn Object>]>,
}

impl UserFunction {
    pub fn new(inner: Rc<Function>, externals: Rc<[Handle<dyn Object>]>) -> Self {
        Self { inner, externals }
    }

    pub fn externals(&self) -> &Rc<[Handle<dyn Object>]> {
        &self.externals
    }

    pub fn inner(&self) -> &Rc<Function> {
        &self.inner
    }
}
