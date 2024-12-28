use dash_proc_macro::Trace;

use crate::{delegate, extract};

use super::Value;
use super::object::{NamedObject, Object};
use super::set::Set;

#[derive(Debug, Trace)]
pub struct WeakSet {
    // for now
    set: Set,
}

impl WeakSet {
    pub fn with_obj(object: NamedObject) -> Self {
        Self {
            set: Set::with_obj(object),
        }
    }

    pub fn null() -> Self {
        Self::with_obj(NamedObject::null())
    }

    pub fn add(&self, key: Value) {
        self.set.add(key);
    }

    pub fn delete(&self, key: &Value) -> bool {
        self.set.delete(key)
    }

    pub fn has(&self, key: &Value) -> bool {
        self.set.has(key)
    }
}

impl Object for WeakSet {
    delegate!(
        set,
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
