use dash_middle::interner::sym;
use dash_proc_macro::Trace;

use crate::localscope::LocalScope;
use crate::{delegate, extract};

use super::Value;
use super::object::{Object, OrdObject, PropertyValue};
use super::propertykey::ToPropertyKey;

#[derive(Debug, Trace)]
pub struct Arguments {
    object: OrdObject,
}

impl Arguments {
    pub fn new(
        scope: &mut LocalScope<'_>,
        args: impl IntoIterator<IntoIter = impl ExactSizeIterator<Item = Value>>,
    ) -> Self {
        let args = args.into_iter();
        let len = args.len();
        let object = OrdObject::null();

        for (idx, arg) in args.enumerate() {
            object
                .set_property(idx.to_key(scope), PropertyValue::static_non_enumerable(arg), scope)
                .unwrap();
        }
        object
            .set_property(
                sym::length.to_key(scope),
                PropertyValue::static_default(Value::number(len as f64)),
                scope,
            )
            .unwrap();

        Self { object }
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
