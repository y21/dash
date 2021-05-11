use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::value::Value;

#[derive(Debug)]
pub struct Environment(HashMap<Box<str>, Rc<RefCell<Value>>>);

impl Environment {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get_var(&self, k: impl AsRef<str>) -> Option<Rc<RefCell<Value>>> {
        self.0.get(k.as_ref()).cloned()
    }

    pub fn set_var(&mut self, k: impl Into<Box<str>>, v: Rc<RefCell<Value>>) {
        self.0.insert(k.into(), v);
    }
}
