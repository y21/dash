use std::collections::HashMap;

use dash_middle::compiler::constant::Function;
use indexmap::IndexMap;

#[derive(Debug)]
pub struct Trace {
    pub(crate) origin: *const Function,
    pub(crate) start: usize,
    pub(crate) end: usize,
    /// A vector of conditional jumps, i.e. diverging control flow.
    /// The index is the # of the jump and the bool represents whether the jump is taken.
    pub(crate) conditional_jumps: Vec<bool>,

    pub(crate) locals: IndexMap<u16, i64>,
    pub(crate) constants: HashMap<u16, i64>
}

impl Trace {
    pub fn new(origin: *const Function, start: usize, end: usize) -> Self {
        Self {
            origin,
            start,
            end,
            conditional_jumps: Vec::new(),
            locals: IndexMap::new(),
            constants: HashMap::new()
        }
    }

    pub fn record_local(&mut self, index: u16, value: i64) {
        self.locals.insert(index, value);
    }

    pub fn record_constant(&mut self, index: u16, value: i64) {
        self.constants.insert(index, value);
    }

    pub fn record_conditional_jump(&mut self, taken: bool) {
        self.conditional_jumps.push(taken);
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }
}
