use crate::compiler::constant::Constant;

#[derive(Debug, Clone)]
pub struct Frame {
    pub ip: usize,
    pub local_count: usize,
    pub constants: Box<[Constant]>,
    pub buffer: Box<[u8]>,
}
