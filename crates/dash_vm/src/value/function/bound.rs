use std::any::Any;

use dash_proc_macro::Trace;

use crate::delegate;
use crate::gc2::handle::Handle;
use crate::value::object::NamedObject;
use crate::value::object::Object;
use crate::value::Typeof;
use crate::value::Value;
use crate::Vm;

#[derive(Debug, Trace)]
pub struct BoundFunction {
    callee: Handle<dyn Object>,
    this: Option<Value>,
    args: Option<Vec<Value>>,
    obj: NamedObject,
}

impl BoundFunction {
    pub fn new(vm: &mut Vm, callee: Handle<dyn Object>, this: Option<Value>, args: Option<Vec<Value>>) -> Self {
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
        scope: &mut crate::local::LocalScope,
        _callee: Handle<dyn Object>,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
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
