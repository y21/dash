use dash_middle::interner::sym;
use dash_proc_macro::Trace;

use crate::gc::interner::Symbol;
use crate::localscope::LocalScope;
use crate::throw;
use crate::value::boxed::String as BoxedString;

use super::object::{Object, PropertyKey, PropertyValue};
use super::ops::conversions::{PreferredType, ValueConversion};
use super::primitive::{array_like_keys, InternalSlots};
use super::{Typeof, Unrooted, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Trace)]
pub struct JsString {
    sym: Symbol,
}

impl From<Symbol> for JsString {
    fn from(sym: Symbol) -> Self {
        Self { sym }
    }
}

impl PartialEq<Symbol> for JsString {
    fn eq(&self, other: &Symbol) -> bool {
        self.sym == *other
    }
}

impl JsString {
    pub fn sym(self) -> Symbol {
        self.sym
    }

    pub fn res<'a>(self, sc: &'a LocalScope<'_>) -> &'a str {
        sc.interner.resolve(self.sym)
    }

    pub fn len(self, sc: &mut LocalScope<'_>) -> usize {
        self.res(sc).len()
    }
}

impl ValueConversion for JsString {
    fn to_primitive(&self, _: &mut LocalScope, _: Option<PreferredType>) -> Result<Value, Value> {
        Ok(Value::String(*self))
    }

    fn to_number(&self, sc: &mut LocalScope) -> Result<f64, Value> {
        if self.sym == sym::empty {
            Ok(0.0)
        } else {
            Ok(self.res(sc).parse().unwrap_or(f64::NAN))
        }
    }

    fn to_boolean(&self, sc: &mut LocalScope<'_>) -> Result<bool, Value> {
        Ok(!self.res(sc).is_empty())
    }

    fn to_js_string(&self, _: &mut LocalScope) -> Result<JsString, Value> {
        Ok(*self)
    }

    fn length_of_array_like(&self, sc: &mut LocalScope) -> Result<usize, Value> {
        Ok(self.res(sc).len())
    }

    fn to_object(&self, sc: &mut LocalScope) -> Result<crate::gc::handle::Handle, Value> {
        let bx = BoxedString::new(sc, *self);
        Ok(sc.register(bx))
    }
}

impl Object for JsString {
    fn get_own_property_descriptor(
        &self,
        sc: &mut LocalScope,
        key: super::object::PropertyKey,
    ) -> Result<Option<super::object::PropertyValue>, super::Unrooted> {
        if let PropertyKey::String(st) = key {
            if st.sym() == sym::length {
                return Ok(Some(PropertyValue::static_empty(Value::number(self.len(sc) as f64))));
            }

            if let Ok(index) = st.res(sc).parse::<usize>() {
                let bytes = self.res(sc).as_bytes();
                if let Some(&byte) = bytes.get(index) {
                    let s = sc.intern((byte as char).to_string().as_ref());
                    return Ok(Some(PropertyValue::static_non_enumerable(Value::String(s.into()))));
                }
            }
        }

        Ok(None)
    }

    fn set_property(
        &self,
        _: &mut LocalScope,
        _: super::object::PropertyKey,
        _: super::object::PropertyValue,
    ) -> Result<(), Value> {
        Ok(())
    }

    fn delete_property(&self, _: &mut LocalScope, _: super::object::PropertyKey) -> Result<super::Unrooted, Value> {
        Ok(Unrooted::new(Value::undefined()))
    }

    fn set_prototype(&self, _: &mut LocalScope, _: Value) -> Result<(), Value> {
        Ok(())
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        Ok(sc.statics.string_prototype.clone().into())
    }

    fn apply(
        &self,
        scope: &mut LocalScope,
        _: crate::gc::handle::Handle,
        _: Value,
        _: Vec<Value>,
    ) -> Result<super::Unrooted, super::Unrooted> {
        let v = self.res(scope).to_owned();
        throw!(scope, TypeError, "'{}' is not a function", v)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn own_keys(&self, sc: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        let len = self.len(sc);
        Ok(array_like_keys(sc, len).collect())
    }

    fn type_of(&self) -> Typeof {
        Typeof::String
    }

    fn internal_slots(&self) -> Option<&dyn InternalSlots> {
        Some(self)
    }
}

impl InternalSlots for JsString {
    fn string_value(&self) -> Option<JsString> {
        Some(*self)
    }
}
