
#[derive(Debug)]
pub struct Trace {
    pub(crate) start: usize,
    pub(crate) end: usize,
    /// A vector of conditional jumps, i.e. diverging control flow.
    /// The index is the # of the jump and the bool represents whether the jump is taken.
    pub(crate) conditional_jumps: Vec<bool>
}

impl Trace {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end, conditional_jumps: Vec::new() }
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
