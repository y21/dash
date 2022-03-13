use std::rc::Rc;

use crate::gc::trace::Trace;
use crate::vm::local::LocalScope;
use crate::vm::Vm;

use super::object::NamedObject;
use super::object::Object;
use super::Value;

macro_rules! boxed_primitive {
    ($($name:ident: $t:ty),*) => {
        $(
            #[derive(Debug)]
            pub struct $name($t, NamedObject);

            impl $name {
                pub fn new(vm: &mut Vm, value: $t) -> Self {
                    Self(value, NamedObject::new(vm))
                }

                pub fn with_obj(value: $t, obj: NamedObject) -> Self {
                    Self(value, obj)
                }
            }

            unsafe impl Trace for $name {
                fn trace(&self) {
                    self.1.trace();
                }
            }

            impl Object for $name {
                fn get_property(&self, sc: &mut LocalScope, key: &str) -> Result<Value, Value> {
                    self.1.get_property(sc, key)
                }

                fn apply(&self, sc: &mut LocalScope, this: Value, args: Vec<Value>) -> Result<Value, Value> {
                    self.1.apply(sc, this, args)
                }

                fn set_property(&self, sc: &mut LocalScope, key: &str, value: Value) -> Result<(), Value> {
                    self.1.set_property(sc, key, value)
                }

                fn as_any(&self) -> &dyn std::any::Any {
                    self
                }

                fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
                    self.1.set_prototype(sc, value)
                }

                fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
                    self.1.get_prototype(sc)
                }
            }
        )*
    }
}

boxed_primitive! {
    Number: f64,
    Boolean: bool,
    String: Rc<str>
}
