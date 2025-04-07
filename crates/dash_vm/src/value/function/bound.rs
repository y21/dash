use dash_proc_macro::Trace;

use crate::frame::This;
use crate::gc::ObjectId;
use crate::localscope::LocalScope;
use crate::value::object::{OrdObject, Object};
use crate::value::{Typeof, Unrooted, Value};
use crate::{Vm, delegate, extract};

use super::args::CallArgs;

#[derive(Debug, Trace)]
pub struct BoundFunction {
    callee: ObjectId,
    this: Option<Value>,
    args: CallArgs,
    obj: OrdObject,
}

impl BoundFunction {
    pub fn new(vm: &Vm, callee: ObjectId, this: Option<Value>, args: CallArgs) -> Self {
        Self {
            callee,
            this,
            args,
            obj: OrdObject::new(vm),
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
        _callee: ObjectId,
        this: This,
        args: CallArgs,
        scope: &mut LocalScope,
    ) -> Result<Unrooted, Unrooted> {
        let target_this = self.this.map_or(this, This::Bound);

        let mut target_args = self.args.clone();
        target_args.extend(args);

        self.callee.apply(target_this, target_args, scope)
    }

    fn type_of(&self, _: &Vm) -> Typeof {
        Typeof::Function
    }

    extract!(self);
}
