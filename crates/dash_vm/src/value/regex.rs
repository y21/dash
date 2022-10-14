use std::rc::Rc;

use dash_regex::node::Node;

use crate::delegate;
use crate::gc::trace::Trace;
use crate::Vm;

use super::object::NamedObject;
use super::object::Object;

#[derive(Debug)]
struct RegExpInner {
    nodes: Vec<Node>,
    source: Rc<str>,
}

#[derive(Debug)]
pub struct RegExp {
    inner: Option<RegExpInner>,
    object: NamedObject,
}

impl RegExp {
    pub fn new(nodes: Vec<Node>, source: Rc<str>, vm: &mut Vm) -> Self {
        let proto = vm.statics.regexp_prototype.clone();
        let ctor = vm.statics.regexp_ctor.clone();

        Self {
            inner: Some(RegExpInner { nodes, source }),
            object: NamedObject::with_prototype_and_constructor(proto, ctor),
        }
    }

    pub fn empty() -> Self {
        Self {
            inner: None,
            object: NamedObject::null(),
        }
    }

    pub fn inner(&self) -> Option<(&[Node], &str)> {
        self.inner
            .as_ref()
            .map(|inner| (inner.nodes.as_slice(), inner.source.as_ref()))
    }
}

unsafe impl Trace for RegExp {
    fn trace(&self) {
        self.object.trace();
    }
}

impl Object for RegExp {
    delegate!(
        object,
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
