#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Upvalue {
    pub local: bool,
    pub idx: usize,
}

impl Upvalue {
    pub fn new(local: bool, idx: usize) -> Self {
        Self { local, idx }
    }
}
