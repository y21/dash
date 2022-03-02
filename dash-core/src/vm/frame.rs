use std::rc::Rc;

use crate::compiler::constant::Constant;

#[derive(Debug, Clone)]
pub struct Frame {
    pub ip: usize,
    pub local_count: usize,
    pub constants: Rc<[Constant]>,
    pub buffer: Rc<[u8]>,
}
