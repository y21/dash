use crate::compiler::constant::Constant;

pub struct Frame {
    pub ip: usize,
    pub constants: Box<[Constant]>,
    pub buffer: Box<[u8]>,
}
