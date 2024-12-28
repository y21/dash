use dash_proc_macro::Trace;

use crate::frame::This;
use crate::gc::ObjectId;
use crate::value::object::{NamedObject, Object};
use crate::value::{Typeof, Unrooted, Value};
use crate::{Vm, delegate, extract};

#[derive(Debug, Trace)]
pub struct BoundFunction {
    callee: ObjectId,
    this: Option<Value>,
    args: Option<Vec<Value>>,
    obj: NamedObject,
}

impl BoundFunction {
    pub fn new(vm: &Vm, callee: ObjectId, this: Option<Value>, args: Option<Vec<Value>>) -> Self {
        Self {
            callee,
            this,
            args,
            obj: NamedObject::new(vm),
        }
    }
}

impl Object for BoundFunction {
    delegate!(
        obj,
        get_own_property_descriptor,
        get_property,
        get_property_descriptor,
        set_property,
        delete_property,
        set_prototype,
        get_prototype,
        own_keys
    );

    fn apply(
        &self,
        scope: &mut crate::localscope::LocalScope,
        _callee: ObjectId,
        this: This,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        let target_this = self.this.map_or(this, This::Bound);

        let mut target_args = self.args.clone().unwrap_or_default();
        target_args.extend(args);

        self.callee.apply(scope, target_this, target_args)
    }

    fn type_of(&self, _: &Vm) -> Typeof {
        Typeof::Function
    }

    extract!(self);
}
