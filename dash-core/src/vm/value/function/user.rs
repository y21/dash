use std::rc::Rc;

use crate::compiler::constant::Constant;

pub struct UserFunction {
    buffer: Rc<[u8]>,
    constants: Rc<[Constant]>,
    externals: Box<[u16]>,
    locals: usize,
    params: usize,
}

impl UserFunction {
    pub fn new(
        buffer: Rc<[u8]>,
        constants: Rc<[Constant]>,
        externals: Box<[u16]>,
        locals: usize,
        params: usize,
    ) -> Self {
        Self {
            buffer,
            constants,
            externals,
            locals,
            params,
        }
    }

    pub fn externals(&self) -> &[u16] {
        &self.externals
    }

    pub fn buffer(&self) -> &Rc<[u8]> {
        &self.buffer
    }

    pub fn constants(&self) -> &Rc<[Constant]> {
        &self.constants
    }

    pub fn locals(&self) -> usize {
        self.locals
    }

    pub fn params(&self) -> usize {
        self.params
    }
}
