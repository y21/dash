use dash_middle::interner::sym;
use dash_proc_macro::Trace;

use crate::localscope::LocalScope;
use crate::{delegate, extract};

use super::Value;
use super::object::{NamedObject, Object, ObjectMap, PropertyValue};
use super::propertykey::ToPropertyKey;

#[derive(Debug, Clone, Trace)]
pub struct Arguments {
    object: NamedObject,
}

impl Arguments {
    pub fn new(vm: &mut LocalScope, args: impl IntoIterator<IntoIter = impl ExactSizeIterator<Item = Value>>) -> Self {
        let args = args.into_iter();
        let len = args.len();
        let mut args = args
            .enumerate()
            .map(|(idx, v)| (idx.to_key(vm), PropertyValue::static_non_enumerable(v)))
            .collect::<ObjectMap<_, _>>();
        args.insert(
            sym::length.to_key(vm),
            PropertyValue::static_default(Value::number(len as f64)),
        );

        Self {
            object: NamedObject::null_with_values(args),
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

    extract!(self);
}
