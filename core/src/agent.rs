use crate::vm::{instruction::Instruction, value::Value};

pub enum ImportResult {
    Value(Value),
    Bytecode(Vec<Instruction>), // TODO: Box<[Instruction]>?
}

pub trait Agent {
    /// A method that is called when the compiler resolves an import statement
    fn import(&mut self, module_name: &[u8]) -> Option<ImportResult>;
    /// A method that is called at runtime when Math.random() is called
    fn random(&mut self) -> Option<f64>;
}

impl Agent for () {
    fn import(&mut self, _: &[u8]) -> Option<ImportResult> {
        None
    }

    fn random(&mut self) -> Option<f64> {
        None
    }
}
