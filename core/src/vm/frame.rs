use super::instruction::Instruction;

#[derive(Debug)]
pub struct Frame {
    pub buffer: Box<[Instruction]>,
    /// Instruction pointer
    pub ip: usize,
    /// Stack pointer
    pub sp: usize,
}
