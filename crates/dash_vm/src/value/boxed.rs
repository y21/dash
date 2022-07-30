use std::any::Any;
use std::rc::Rc;

use super::ops::abstractions::conversions::PreferredType;
use super::ops::abstractions::conversions::ValueConversion;
use super::ops::equality::ValueEquality;
use crate::gc::handle::Handle;
use crate::gc::trace::Trace;
use crate::local::LocalScope;
use crate::value::object::PropertyValue;
use crate::Vm;

use super::object::NamedObject;
use super::object::Object;
use super::object::PropertyKey;
use super::primitive::BuiltinCapabilities;
use super::primitive::Symbol as PrimitiveSymbol;
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

                pub fn value(&self) -> &$t {
                    &self.0
                }
            }

            unsafe impl Trace for $name {
                fn trace(&self) {
                    self.1.trace();
                }
            }

            impl Object for $name {
                fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
                    self.1.get_property(sc, key)
                }

                fn apply(&self, sc: &mut LocalScope,
                    callee: Handle<dyn Object>,this: Value, args: Vec<Value>) -> Result<Value, Value> {
                    self.1.apply(sc, callee, this, args)
                }

                fn set_property(&self, sc: &mut LocalScope, key: PropertyKey<'static>, value: PropertyValue) -> Result<(), Value> {
                    self.1.set_property(sc, key, value)
                }

                fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
                    self.1.delete_property(sc, key)
                }

                fn as_any(&self) -> &dyn Any {
                    self
                }

                fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
                    self.1.set_prototype(sc, value)
                }

                fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
                    self.1.get_prototype(sc)
                }

                fn own_keys(&self) -> Result<Vec<Value>, Value> {
                    self.1.own_keys()
                }

                fn as_builtin_capable(&self) -> Option<&dyn BuiltinCapabilities> {
                    Some(self)
                }
            }

            impl ValueEquality for $name {
                fn lt(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
                    ValueEquality::lt(&self.0, other, sc)
                }

                fn le(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
                    ValueEquality::le(&self.0, other, sc)
                }

                fn gt(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
                    ValueEquality::gt(&self.0, other, sc)
                }

                fn ge(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
                    ValueEquality::ge(&self.0, other, sc)
                }

                fn eq(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
                    ValueEquality::eq(&self.0, other, sc)
                }

                fn strict_eq(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
                    // TODO: compare pointers
                    ValueEquality::strict_eq(&self.0, other, sc)
                }

                fn ne(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
                    ValueEquality::ne(&self.0, other, sc)
                }

                fn strict_ne(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
                    ValueEquality::strict_ne(&self.0, other, sc)
                }
            }

            impl ValueConversion for $name {
                fn to_primitive(&self, sc: &mut LocalScope, preferred_type: Option<PreferredType>) -> Result<Value, Value> {
                    ValueConversion::to_primitive(&self.0, sc, preferred_type)
                }

                fn to_number(&self, sc: &mut LocalScope) -> Result<f64, Value> {
                    ValueConversion::to_number(&self.0, sc)
                }

                fn to_boolean(&self) -> Result<bool, Value> {
                    ValueConversion::to_boolean(&self.0)
                }

                fn to_string(&self, sc: &mut LocalScope) -> Result<Rc<str>, Value> {
                    ValueConversion::to_string(&self.0, sc)
                }

                fn length_of_array_like(&self, sc: &mut LocalScope) -> Result<usize, Value> {
                    ValueConversion::length_of_array_like(&self.0, sc)
                }

                fn to_object(&self, sc: &mut LocalScope) -> Result<Handle<dyn Object>, Value> {
                    ValueConversion::to_object(&self.0, sc)
                }
            }
        )*
    }
}

boxed_primitive! {
    Number: f64,
    Boolean: bool,
    String: Rc<str>,
    Symbol: PrimitiveSymbol
}

impl BuiltinCapabilities for Number {
    fn as_number(&self) -> Option<f64> {
        Some(self.0)
    }
}

impl BuiltinCapabilities for Boolean {
    fn as_bool(&self) -> Option<bool> {
        Some(self.0)
    }
}

impl BuiltinCapabilities for String {
    fn as_string(&self) -> Option<Rc<str>> {
        Some(self.0.clone())
    }
}

impl BuiltinCapabilities for Symbol {}
