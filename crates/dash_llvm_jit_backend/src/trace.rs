use std::collections::HashMap;

use dash_middle::compiler::constant::Constant;
use dash_middle::compiler::constant::Function;
use dash_middle::compiler::instruction as inst;
use dash_typed_cfg::passes::bb_generation::ConditionalBranchAction;
use indexmap::IndexMap;

#[derive(Debug)]
pub struct Trace {
    pub(crate) is_subtrace: bool,
    pub(crate) origin: *const Function,
    pub(crate) start: usize,
    pub(crate) end: usize,
    /// A vector of conditional jumps, i.e. diverging control flow.
    /// The index is the # of the jump and the bool represents whether the jump is taken.
    ///
    /// Note for later: can change to HashSet<usize, bool> where usize is the IP if a trace
    /// is composed of multiple possible paths
    // pub(crate) conditional_jumps: Vec<bool>,
    pub(crate) conditional_jumps: HashMap<usize, ConditionalBranchAction>,
}

impl Trace {
    pub fn new(origin: *const Function, start: usize, end: usize, is_subtrace: bool) -> Self {
        Self {
            origin,
            start,
            end,
            conditional_jumps: HashMap::new(),
            is_subtrace,
        }
    }

    pub fn get_conditional_jump(&self, id: usize) -> Option<ConditionalBranchAction> {
        self.conditional_jumps.get(&id).copied()
    }

    pub fn record_conditional_jump(&mut self, id: usize, action: ConditionalBranchAction) {
        self.conditional_jumps.insert(id, action);
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

    pub fn set_subtrace(&mut self) {
        self.is_subtrace = true;
    }

    pub fn is_subtrace(&self) -> bool {
        self.is_subtrace
    }
}
