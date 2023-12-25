use std::cell::Cell;
use std::rc::Rc;

use dash_proc_macro::Trace;
use dash_regex::ParsedRegex;

use crate::{delegate, Vm};

use super::object::{NamedObject, Object};

#[derive(Debug)]
pub struct RegExpInner {
    pub regex: ParsedRegex,
    pub source: Rc<str>,
    // TODO: this should only exist if the `g` flag is set (we currently don't even have regex flags)
    pub last_index: Cell<usize>,
}

#[derive(Debug, Trace)]
pub struct RegExp {
    inner: Option<RegExpInner>,
    object: NamedObject,
}

impl RegExp {
    pub fn new(regex: ParsedRegex, source: Rc<str>, vm: &Vm) -> Self {
        let proto = vm.statics.regexp_prototype.clone();
        let ctor = vm.statics.regexp_ctor.clone();

        Self {
            inner: Some(RegExpInner {
                regex,
                source,
                last_index: Cell::new(0),
            }),
            object: NamedObject::with_prototype_and_constructor(proto, ctor),
        }
    }

    pub fn empty() -> Self {
        Self {
            inner: None,
            object: NamedObject::null(),
        }
    }

    pub fn inner(&self) -> Option<&RegExpInner> {
        self.inner.as_ref()
    }
}

impl Object for RegExp {
    delegate!(
        object,
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
