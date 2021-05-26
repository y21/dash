use std::{collections::HashSet, rc::Rc};

use super::HashWeak;

#[derive(Debug, Clone)]
pub struct WeakSet<T>(pub HashSet<HashWeak<T>>);

impl<T> WeakSet<T> {
    pub fn new() -> Self {
        Self(HashSet::new())
    }

    pub fn has(&self, rc: &Rc<T>) -> bool {
        let weak = Rc::downgrade(rc);
        let hashweak = HashWeak(weak);
        self.0.contains(&hashweak)
    }

    pub fn add(&mut self, rc: &Rc<T>) {
        let weak = Rc::downgrade(rc);
        let hashweak = HashWeak(weak);
        self.0.insert(hashweak);
    }

    pub fn delete(&mut self, rc: &Rc<T>) -> bool {
        let weak = Rc::downgrade(rc);
        let hashweak = HashWeak(weak);
        self.0.remove(&hashweak)
    }
}

impl<T> From<HashSet<HashWeak<T>>> for WeakSet<T> {
    fn from(s: HashSet<HashWeak<T>>) -> Self {
        Self(s)
    }
}
