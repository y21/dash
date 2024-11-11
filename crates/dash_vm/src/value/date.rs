use dash_proc_macro::Trace;

use crate::localscope::LocalScope;
use crate::{delegate, extract, throw};

use super::object::{NamedObject, Object};
use super::root_ext::RootErrExt;
use super::Value;

#[derive(Debug, Trace)]
pub struct Date {
    pub timestamp: u64,
    object: NamedObject,
}

impl Date {
    pub fn current(sc: &mut LocalScope<'_>) -> Result<Self, Value> {
        let Some(cb) = sc.params.time_millis_callback else {
            throw!(sc, Error, "failed to get time")
        };
        let timestamp = cb(sc).root_err(sc)?;

        Ok(Self {
            timestamp,
            object: NamedObject::with_prototype_and_constructor(sc.statics.date_prototype, sc.statics.date_ctor),
        })
    }
}

impl Object for Date {
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
