use std::collections::HashMap;

use dash_middle::compiler::constant::Constant;
use dash_middle::compiler::constant::Function;
use dash_middle::compiler::instruction as inst;
use indexmap::IndexMap;

#[derive(Debug)]
pub struct Trace {
    pub(crate) origin: *const Function,
    pub(crate) start: usize,
    pub(crate) end: usize,
    /// A vector of conditional jumps, i.e. diverging control flow.
    /// The index is the # of the jump and the bool represents whether the jump is taken.
    ///
    /// Note for later: can change to HashSet<usize, bool> where usize is the IP if a trace
    /// is composed of multiple possible paths
    pub(crate) conditional_jumps: Vec<bool>,
}

impl Trace {
    pub fn new(origin: *const Function, start: usize, end: usize) -> Self {
        Self {
            origin,
            start,
            end,
            conditional_jumps: Vec::new(),
        }
    }

    pub fn get_conditional_jump(&self, id: usize) -> bool {
        self.conditional_jumps[id]
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

    pub fn origin(&self) -> *const Function {
        self.origin
    }
}
