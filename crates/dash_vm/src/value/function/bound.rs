use std::any::Any;

use dash_proc_macro::Trace;

use crate::gc::handle::Handle;
use crate::value::object::{NamedObject, Object};
use crate::value::{Typeof, Unrooted, Value};
use crate::{delegate, Vm};

#[derive(Debug, Trace)]
pub struct BoundFunction {
    callee: Handle,
    this: Option<Value>,
    args: Option<Vec<Value>>,
    obj: NamedObject,
}

impl BoundFunction {
    pub fn new(vm: &Vm, callee: Handle, this: Option<Value>, args: Option<Vec<Value>>) -> Self {
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
        _callee: Handle,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        let target_this = self.this.clone().unwrap_or(this);

        // TODO: args should be concatenated with self.args
        let target_args = self.args.clone().unwrap_or(args);

        self.callee.apply(scope, target_this, target_args)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn type_of(&self) -> Typeof {
        Typeof::Function
    }
}
