use std::cell::RefCell;
use std::collections::HashSet;

use dash_proc_macro::Trace;

use crate::delegate;
use crate::Vm;

use super::object::NamedObject;
use super::object::Object;
use super::Value;

#[derive(Debug, Trace)]
pub struct Set {
    inner: RefCell<HashSet<Value>>,
    obj: NamedObject,
}

impl Set {
    pub fn new(vm: &mut Vm) -> Self {
        let prototype = vm.statics.set_prototype.clone();
        let ctor = vm.statics.set_constructor.clone();
        Self::with_obj(NamedObject::with_prototype_and_constructor(prototype, ctor))
    }

    pub fn with_obj(obj: NamedObject) -> Self {
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
        as_any,
        apply,
        own_keys
    );
}
