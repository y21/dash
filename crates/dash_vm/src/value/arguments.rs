use std::any::Any;

use dash_middle::interner::sym;
use dash_proc_macro::Trace;

use crate::delegate;
use crate::localscope::LocalScope;

use super::object::{NamedObject, Object, PropertyKey, PropertyValue};
use super::Value;

#[derive(Debug, Clone, Trace)]
pub struct Arguments {
    object: NamedObject,
}

impl Arguments {
    pub fn new(vm: &mut LocalScope, args: impl IntoIterator<IntoIter = impl ExactSizeIterator<Item = Value>>) -> Self {
        let args = args.into_iter();
        let len = args.len();

        Self {
            object: NamedObject::null_with_values(
                args.enumerate()
                    .map(|(i, v)| {
                        (
                            PropertyKey::String(vm.interner.intern_usize(i).into()),
                            PropertyValue::static_non_enumerable(v),
                        )
                    })
                    .chain([(
                        PropertyKey::String(sym::length.into()),
                        PropertyValue::static_default(Value::number(len as f64)),
                    )])
                    .collect(),
            ),
        }
    }
}

impl Object for Arguments {
    delegate!(
        object,
        get_own_property_descriptor,
        get_prototype,
        set_prototype,
        set_property,
        own_keys,
        delete_property,
        apply
    );

    fn as_any(&self) -> &dyn Any {
        self
    }
}
