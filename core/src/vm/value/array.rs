use super::Value;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Array {
    pub elements: Vec<Rc<RefCell<Value>>>,
}

impl Array {
    pub fn new(elements: Vec<Rc<RefCell<Value>>>) -> Self {
        Self { elements }
    }

    pub fn at(&self, idx: impl Into<usize>) -> Option<Rc<RefCell<Value>>> {
        self.elements.get(idx.into()).cloned()
    }
}
