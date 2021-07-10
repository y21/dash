use super::Value;
use std::cell::RefCell;
use std::rc::Rc;

/// A JavaScript Array
#[derive(Debug, Clone)]
pub struct Array {
    /// The elements of this array
    pub elements: Vec<Rc<RefCell<Value>>>,
}

impl Array {
    /// Creates a new JavaScript array
    pub fn new(elements: Vec<Rc<RefCell<Value>>>) -> Self {
        Self { elements }
    }

    /// Returns the value at a given index and clones it
    pub fn at(&self, idx: impl Into<usize>) -> Option<Rc<RefCell<Value>>> {
        self.elements.get(idx.into()).cloned()
    }
}
