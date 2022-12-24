use std::collections::HashMap;

use dash_middle::compiler::constant::Constant;
use dash_middle::compiler::constant::Function;
use dash_middle::compiler::instruction as inst;
use indexmap::IndexMap;

use super::assembler::Assembler;
use super::assembler::JitCacheKey;
use super::assembler::JitResult;
use super::value::Value;

#[derive(Debug)]
pub struct Trace {
    /// Whether this trace records a side exit
    pub(crate) side_exit: bool,
    pub(crate) origin: *const Function,
    pub(crate) start: usize,
    pub(crate) end: usize,
    /// A vector of conditional jumps, i.e. diverging control flow.
    /// The index is the # of the jump and the bool represents whether the jump is taken.
    ///
    /// Note for later: can change to HashSet<usize, bool> where usize is the IP if a trace
    /// is composed of multiple possible paths
    pub(crate) conditional_jumps: Vec<bool>,

    pub(crate) locals: IndexMap<u16, Value>,
    pub(crate) constants: HashMap<u16, Value>,
}

impl Trace {
    pub fn new(origin: *const Function, start: usize, end: usize, side_exit: bool) -> Self {
        Self {
            side_exit,
            origin,
            start,
            end,
            conditional_jumps: Vec::new(),
            locals: IndexMap::new(),
            constants: HashMap::new(),
        }
    }

    pub fn get_conditional_jump(&self, id: usize) -> bool {
        self.conditional_jumps[id]
    }

    pub fn record_local(&mut self, index: u16, value: Value) {
        self.locals.insert(index, value);
    }

    pub fn record_constant(&mut self, index: u16, value: Value) {
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

    pub fn side_exit(&self) -> bool {
        self.side_exit
    }
}
