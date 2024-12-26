use super::ops::conversions::{PreferredType, ValueConversion};
use crate::gc::ObjectId;
use crate::localscope::LocalScope;
use crate::value::{JsString, PropertyKey, Unrooted};
use crate::{PropertyValue, Vm, delegate, extract};
use dash_proc_macro::Trace;

use super::Value;
use super::object::{NamedObject, Object};
use super::primitive::{InternalSlots, Symbol as PrimitiveSymbol};

macro_rules! boxed_primitive {
    ($($name:ident $prototype:ident $constructor:ident $t:ty),*) => {
        $(
            #[derive(Debug, Trace)]
            pub struct $name {
                inner: $t,
                obj: NamedObject
            }

            impl $name {
                pub fn new(vm: &mut Vm, value: $t) -> Self {
                    let prototype = vm.statics.$prototype.clone();
                    let ctor = vm.statics.$constructor.clone();
                    Self { inner: value, obj: NamedObject::with_prototype_and_constructor(prototype, ctor) }
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
                ) -> Result<Option<PropertyValue>, Unrooted> {
                    if let Some(x) = self.inner.get_own_property_descriptor(sc, key.clone())? {
                        return Ok(Some(x));
                    }

                    return self.obj.get_own_property_descriptor(sc, key);
                }

                fn internal_slots(&self, _: &Vm) -> Option<&dyn InternalSlots> {
                    Some(self)
                }

                extract!(self, inner);
            }

            impl ValueConversion for $name {
                fn to_primitive(&self, sc: &mut LocalScope, preferred_type: Option<PreferredType>) -> Result<Value, Value> {
                    ValueConversion::to_primitive(&self.inner, sc, preferred_type)
                }

                fn to_number(&self, sc: &mut LocalScope) -> Result<f64, Value> {
                    ValueConversion::to_number(&self.inner, sc)
                }

                fn to_boolean(&self, sc: &mut LocalScope<'_>) -> Result<bool, Value> {
                    ValueConversion::to_boolean(&self.inner, sc)
                }

                fn to_js_string(&self, sc: &mut LocalScope) -> Result<JsString, Value> {
                    ValueConversion::to_js_string(&self.inner, sc)
                }

                fn length_of_array_like(&self, sc: &mut LocalScope) -> Result<usize, Value> {
                    ValueConversion::length_of_array_like(&self.inner, sc)
                }

                fn to_object(&self, sc: &mut LocalScope) -> Result<ObjectId, Value> {
                    ValueConversion::to_object(&self.inner, sc)
                }
            }
        )*
    }
}

boxed_primitive! {
    Number number_prototype number_ctor f64, // TODO: should this store a primitive::Number?
    Boolean boolean_prototype boolean_ctor bool,
    String string_prototype string_ctor JsString,
    Symbol symbol_prototype symbol_ctor PrimitiveSymbol
}

impl InternalSlots for Number {
    fn number_value(&self, _: &Vm) -> Option<f64> {
        Some(self.inner)
    }
}

impl InternalSlots for Boolean {
    fn boolean_value(&self, _: &Vm) -> Option<bool> {
        Some(self.inner)
    }
}

impl InternalSlots for String {
    fn string_value(&self, _: &Vm) -> Option<JsString> {
        Some(self.inner)
    }
}

impl InternalSlots for Symbol {}
