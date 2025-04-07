use std::cell::RefCell;
use std::collections::HashSet;

use dash_proc_macro::Trace;

use crate::{Vm, delegate, extract};

use super::Value;
use super::object::{OrdObject, Object};

#[derive(Debug, Trace)]
pub struct Set {
    inner: RefCell<HashSet<Value>>,
    obj: OrdObject,
}

impl Set {
    pub fn new(vm: &Vm) -> Self {
        Self::with_obj(OrdObject::with_prototype_and_constructor(
            vm.statics.set_prototype,
            vm.statics.set_constructor,
        ))
    }

    pub fn with_obj(obj: OrdObject) -> Self {
        Self {
            inner: RefCell::new(HashSet::new()),
            obj,
        }
    }

    pub fn add(&self, item: Value) {
        self.inner.borrow_mut().insert(item);
    }

    pub fn has(&self, item: &Value) -> bool {
        self.inner.borrow().contains(item)
    }

    pub fn delete(&self, item: &Value) -> bool {
        self.inner.borrow_mut().remove(item)
    }

    pub fn clear(&self) {
        self.inner.borrow_mut().clear();
    }

    pub fn size(&self) -> usize {
        self.inner.borrow().len()
    }
}

impl Extend<Value> for Set {
    fn extend<T: IntoIterator<Item = Value>>(&mut self, iter: T) {
        self.inner.borrow_mut().extend(iter);
    }
}

impl Object for Set {
    delegate!(
        obj,
        get_own_property_descriptor,
        get_property,
        get_property_descriptor,
        set_property,
        delete_property,
        set_prototype,
        get_prototype,
        apply,
        own_keys
    );

    extract!(self);
}
