use std::cell::Cell;

use dash_proc_macro::Trace;
use dash_regex::Regex;

use crate::gc::trace::{Trace, TraceCtxt};
use crate::{Vm, delegate, extract};

use super::object::{NamedObject, Object};
use super::string::JsString;

#[derive(Debug)]
pub struct RegExpInner {
    pub regex: Regex,
    pub source: JsString,
    pub last_index: Cell<usize>,
}

unsafe impl Trace for RegExpInner {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        let Self {
            regex: _,
            source,
            last_index: _,
        } = self;
        source.trace(cx);
    }
}

#[derive(Debug, Trace)]
pub struct RegExp {
    inner: Option<RegExpInner>,
    object: NamedObject,
}

impl RegExp {
    pub fn new(regex: Regex, source: JsString, vm: &Vm) -> Self {
        Self {
            inner: Some(RegExpInner {
                regex,
                source,
                last_index: Cell::new(0),
            }),
            object: NamedObject::with_prototype_and_constructor(vm.statics.regexp_prototype, vm.statics.regexp_ctor),
        }
    }

    pub fn with_obj(regex: Regex, source: JsString, object: NamedObject) -> Self {
        Self {
            inner: Some(RegExpInner {
                regex,
                source,
                last_index: Cell::new(0),
            }),
            object,
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
        apply,
        own_keys
    );

    extract!(self);
}
