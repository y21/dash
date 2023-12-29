use std::cell::RefCell;

use ahash::HashMap;
use dash_proc_macro::Trace;

use crate::{delegate, Vm};

use super::object::{NamedObject, Object};
use super::Value;

#[derive(Debug, Trace)]
pub struct Map {
    inner: RefCell<HashMap<Value, Value>>,
    obj: NamedObject,
}

impl Map {
    pub fn new(vm: &Vm) -> Self {
        let prototype: crate::gc::handle::Handle = vm.statics.map_prototype.clone();
        let ctor = vm.statics.map_constructor.clone();
        Self::with_obj(NamedObject::with_prototype_and_constructor(prototype, ctor))
    }

    pub fn with_obj(obj: NamedObject) -> Self {
        Self {
            inner: RefCell::new(HashMap::default()),
            obj,
        }
    }

    pub fn set(&self, key: Value, value: Value) {
        self.inner.borrow_mut().insert(key, value);
    }

    pub fn has(&self, item: &Value) -> bool {
        self.inner.borrow().contains_key(item)
    }

    pub fn get(&self, item: &Value) -> Option<Value> {
        self.inner.borrow().get(item).cloned()
    }

    pub fn delete(&self, item: &Value) -> bool {
        self.inner.borrow_mut().remove(item).is_some()
    }

    pub fn clear(&self) {
        self.inner.borrow_mut().clear();
    }

    pub fn size(&self) -> usize {
        self.inner.borrow().len()
    }
}

impl Object for Map {
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
