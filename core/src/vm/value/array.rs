use super::Value;

/// A JavaScript Array
#[derive(Debug, Clone)]
pub struct Array {
    /// The elements of this array
    pub elements: Vec<Value>,
}

impl Array {
    /// Creates a new JavaScript array
    pub fn new(elements: Vec<Value>) -> Self {
        Self { elements }
    }

    /// Returns the value at a given index and clones it
    pub fn at(&self, idx: impl Into<usize>) -> Option<Value> {
        self.elements.get(idx.into()).cloned()
    }
}
