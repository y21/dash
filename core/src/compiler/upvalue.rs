/// A compile time upvalue
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Upvalue {
    /// Whether this upvalue is a local
    pub local: bool,
    /// The stack index of this upvalue
    pub idx: usize,
}

impl Upvalue {
    /// Creates a new upvalue
    pub fn new(local: bool, idx: usize) -> Self {
        Self { local, idx }
    }
}
