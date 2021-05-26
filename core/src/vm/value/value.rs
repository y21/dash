use core::fmt::Debug;
use std::{
    cell::RefCell,
    collections::HashMap,
    hash::{Hash, Hasher},
    rc::{Rc, Weak},
};

use super::ValueKind;

#[derive(Debug, Clone)]
pub struct HashRc<T>(pub Rc<T>);

#[derive(Debug, Clone)]
pub struct HashWeak<T>(pub Weak<T>);

impl<T> Hash for HashRc<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.0).hash(state)
    }
}
impl<T> PartialEq for HashRc<T> {
    fn eq(&self, other: &HashRc<T>) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl<T> Eq for HashRc<T> {}

impl<T> Hash for HashWeak<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Weak::as_ptr(&self.0).hash(state)
    }
}
impl<T> PartialEq for HashWeak<T> {
    fn eq(&self, other: &HashWeak<T>) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}
impl<T> Eq for HashWeak<T> {}

#[derive(Debug, Clone)]
pub struct Value {
    pub kind: ValueKind,
    pub fields: HashMap<Box<str>, Rc<RefCell<Value>>>,
    pub constructor: Option<Rc<RefCell<Value>>>,
}

impl Value {
    pub fn new(kind: ValueKind) -> Self {
        Self {
            kind,
            fields: HashMap::new(),
            constructor: None,
        }
    }
}

impl Value {
    pub fn try_into_inner(value: Rc<RefCell<Self>>) -> Option<Self> {
        Some(Rc::try_unwrap(value).ok()?.into_inner())
    }

    pub fn unwrap_or_undefined(o: Option<Rc<RefCell<Self>>>) -> Rc<RefCell<Self>> {
        o.unwrap_or_else(|| Value::new(ValueKind::Undefined).into())
    }

    pub fn get_property(value_cell: &Rc<RefCell<Value>>, k: &str) -> Option<Rc<RefCell<Value>>> {
        let value = value_cell.borrow();
        let k = k.into();

        if value.fields.len() > 0 {
            // We only need to go the "slow" path and look up the given key in a HashMap if there are entries
            if let Some(entry) = value.fields.get(k) {
                return Some(entry.clone());
            }
        }

        match &value.kind {
            ValueKind::Object(o) => o.get_property(value_cell, k),
            _ => None,
        }
    }

    pub fn set_property(&mut self, k: impl Into<Box<str>>, v: Rc<RefCell<Value>>) {
        self.fields.insert(k.into(), v);
    }
}
