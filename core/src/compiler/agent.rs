use crate::vm::{instruction::Instruction, value::Value};

pub enum ImportResult {
    Value(Value),
    Bytecode(Vec<Instruction>), // TODO: Box<[Instruction]>?
}

pub trait Agent {
    fn import(&mut self, module_name: &[u8]) -> Option<ImportResult>;
}

impl Agent for () {
    fn import(&mut self, _: &[u8]) -> Option<ImportResult> {
        None
    }
}
