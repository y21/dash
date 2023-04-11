use super::ops::abstractions::conversions::PreferredType;
use super::ops::abstractions::conversions::ValueConversion;
use super::ops::equality::ValueEquality;
use crate::delegate;
use crate::gc::handle::Handle;
use crate::local::LocalScope;
use crate::value::PropertyKey;
use crate::PropertyValue;
use crate::Vm;
use dash_proc_macro::Trace;
use std::any::Any;
use std::rc::Rc;

use super::object::NamedObject;
use super::object::Object;
use super::primitive::PrimitiveCapabilities;
use super::primitive::Symbol as PrimitiveSymbol;
use super::Value;

macro_rules! boxed_primitive {
    ($($name:ident: $t:ty),*) => {
        $(
            #[derive(Debug, Trace)]
            pub struct $name {
                inner: $t,
                obj: NamedObject
            }

            impl $name {
                pub fn new(vm: &mut Vm, value: $t) -> Self {
                    Self { inner: value, obj: NamedObject::new(vm) }
                }

                pub fn with_obj(value: $t, obj: NamedObject) -> Self {
                    Self { inner: value, obj }
                }

                pub fn value(&self) -> &$t {
                    &self.inner
                }
            }

            impl Object for $name {
                delegate!(
                    obj,
                    set_property,
                    delete_property,
                    set_prototype,
                    get_prototype,
                    own_keys,
                    apply
                );


                fn get_own_property_descriptor(
                    &self,
                    sc: &mut LocalScope,
                    key: PropertyKey,
                ) -> Result<Option<PropertyValue>, Value> {
                    if let Some(x) = self.inner.get_own_property_descriptor(sc, key.clone())? {
                        return Ok(Some(x));
                    }

                    return self.obj.get_own_property_descriptor(sc, key);
                }

                fn as_any(&self) -> &dyn Any {
                    self
                }

                fn as_primitive_capable(&self) -> Option<&dyn PrimitiveCapabilities> {
                    Some(self)
                }
            }

            impl ValueEquality for $name {
                fn lt(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
                    ValueEquality::lt(&self.inner, other, sc)
                }

                fn le(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
                    ValueEquality::le(&self.inner, other, sc)
                }

                fn gt(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
                    ValueEquality::gt(&self.inner, other, sc)
                }

                fn ge(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
                    ValueEquality::ge(&self.inner, other, sc)
                }

                fn eq(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
                    ValueEquality::eq(&self.inner, other, sc)
                }

                fn strict_eq(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
                    // TODO: compare pointers
                    ValueEquality::strict_eq(&self.inner, other, sc)
                }

                fn ne(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
                    ValueEquality::ne(&self.inner, other, sc)
                }

                fn strict_ne(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
                    ValueEquality::strict_ne(&self.inner, other, sc)
                }
            }

            impl ValueConversion for $name {
                fn to_primitive(&self, sc: &mut LocalScope, preferred_type: Option<PreferredType>) -> Result<Value, Value> {
                    ValueConversion::to_primitive(&self.inner, sc, preferred_type)
                }

                fn to_number(&self, sc: &mut LocalScope) -> Result<f64, Value> {
                    ValueConversion::to_number(&self.inner, sc)
                }

                fn to_boolean(&self) -> Result<bool, Value> {
                    ValueConversion::to_boolean(&self.inner)
                }

                fn to_string(&self, sc: &mut LocalScope) -> Result<Rc<str>, Value> {
                    ValueConversion::to_string(&self.inner, sc)
                }

                fn length_of_array_like(&self, sc: &mut LocalScope) -> Result<usize, Value> {
                    ValueConversion::length_of_array_like(&self.inner, sc)
                }

                fn to_object(&self, sc: &mut LocalScope) -> Result<Handle<dyn Object>, Value> {
                    ValueConversion::to_object(&self.inner, sc)
                }
            }
        )*
    }
}

boxed_primitive! {
    Number: f64, // TODO: should this store a primitive::Number?
    Boolean: bool,
    String: Rc<str>,
    Symbol: PrimitiveSymbol
}

impl PrimitiveCapabilities for Number {
    fn as_number(&self) -> Option<f64> {
        Some(self.inner)
    }

    fn unbox(&self) -> Value {
        Value::number(self.inner)
    }
}

impl PrimitiveCapabilities for Boolean {
    fn as_bool(&self) -> Option<bool> {
        Some(self.inner)
    }

    fn unbox(&self) -> Value {
        Value::Boolean(self.inner)
    }
}

impl PrimitiveCapabilities for String {
    fn as_string(&self) -> Option<Rc<str>> {
        Some(self.inner.clone())
    }

    fn unbox(&self) -> Value {
        Value::String(Rc::clone(&self.inner))
    }
}

impl PrimitiveCapabilities for Symbol {
    fn unbox(&self) -> Value {
        Value::Symbol(self.inner.clone())
    }
}
