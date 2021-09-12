use crate::gc::Handle;

use super::Value;

/// The state of a generator execution
#[derive(Debug, Clone)]
pub enum GeneratorState {
    /// The generator has been fully consumed
    Finished,
    /// The generator is currently running
    Running {
        /// The current instruction pointer
        ip: usize,
        /// Function stack
        stack: Vec<Handle<Value>>,
    },
}

/// An iterator over a generator function
///
/// Captures stack state when generator function
/// is suspended, and restores it when resumed.
#[derive(Debug, Clone)]
pub struct GeneratorIterator {
    /// The generator function
    pub function: Handle<Value>,
    /// The state of the generator
    pub state: GeneratorState,
}

impl GeneratorIterator {
    /// Creates a new generator iterator given a generator value
    pub fn new(function: Handle<Value>, stack: Vec<Handle<Value>>) -> Self {
        Self {
            function,
            state: GeneratorState::Running { ip: 0, stack },
        }
    }

    pub(crate) fn mark(&self) {
        Value::mark(&self.function);

        match &self.state {
            GeneratorState::Finished => {}
            GeneratorState::Running { stack, .. } => {
                for handle in stack {
                    Value::mark(handle);
                }
            }
        }
    }
}
