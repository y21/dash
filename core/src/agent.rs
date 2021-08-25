use crate::{
    compiler::instruction::Instruction,
    vm::{instruction::Constant, value::Value},
};

/// The result of a successful import resolve
pub enum ImportResult {
    /// This import resolves to a JavaScript value
    Value(Value),
    /// This import resolves to bytecode
    Bytecode(Vec<Instruction>, Vec<Constant>),
}

/// Embedder specific methods.
///
/// Embedders of this implementation may want to choose behavior when something occurs and handle it differently.
/// For example, embedders can choose to control what happens when an `import` statement is reached.
/// A regular runtime may want to let users import another file.
pub trait Agent {
    /// A method that is called when the compiler resolves an import statement
    fn import(&mut self, _module_name: &[u8]) -> Option<ImportResult> {
        None
    }
    /// A method that is called at runtime when Math.random() is called
    fn random(&mut self) -> Option<f64> {
        None
    }
    /// A method that is called at runtime when a `debugger` statement is reached
    fn debugger(&mut self) {}
}

impl Agent for () {}
