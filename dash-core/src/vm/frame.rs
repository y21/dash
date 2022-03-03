use std::rc::Rc;

use crate::compiler::constant::Constant;
use crate::compiler::CompileResult;

use super::value::function::user::UserFunction;

#[derive(Debug, Clone)]
pub struct Frame {
    pub ip: usize,
    pub local_count: usize,
    pub constants: Rc<[Constant]>,
    pub buffer: Rc<[u8]>,
    pub sp: usize,
}

impl From<CompileResult> for Frame {
    fn from(compiled: CompileResult) -> Self {
        Frame {
            ip: 0,
            local_count: compiled.locals,
            constants: compiled.cp.into_vec().into(),
            buffer: compiled.instructions.into(),
            sp: 0,
        }
    }
}

impl From<&UserFunction> for Frame {
    fn from(uf: &UserFunction) -> Self {
        Frame {
            ip: 0,
            sp: 0,
            buffer: uf.buffer().clone(),
            constants: uf.constants().clone(),
            local_count: uf.locals(),
        }
    }
}
