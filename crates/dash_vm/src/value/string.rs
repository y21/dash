use dash_middle::interner::{Symbol, sym};
use dash_proc_macro::Trace;

use crate::localscope::LocalScope;
use crate::value::boxed::String as BoxedString;
use crate::value::object::This;
use crate::{Vm, extract, throw};

use super::function::args::CallArgs;
use super::object::{Object, PropertyValue};
use super::ops::conversions::{PreferredType, ValueConversion};
use super::primitive::{InternalSlots, array_like_keys};
use super::propertykey::PropertyKey;
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
    pub const fn from_sym(sym: Symbol) -> Self {
        Self { sym }
    }

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
        Ok(Value::string(*self))
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

    fn to_object(&self, sc: &mut LocalScope) -> Result<crate::gc::ObjectId, Value> {
        let bx = BoxedString::new(sc, *self);
        Ok(sc.register(bx))
    }
}

impl Object for JsString {
    fn get_own_property_descriptor(
        &self,
        key: PropertyKey,
        sc: &mut LocalScope,
    ) -> Result<Option<PropertyValue>, Unrooted> {
        // TODO: method for unpacking into either string or numeric

        if let Some(index) = key.index_usize() {
            let bytes = self.res(sc).as_bytes();
            if let Some(&byte) = bytes.get(index) {
                let s = sc.intern((byte as char).to_string().as_ref());
                return Ok(Some(PropertyValue::static_non_enumerable(Value::string(s.into()))));
            }
        } else if let Some(sym::length) = key.to_js_string(sc) {
            return Ok(Some(PropertyValue::static_empty(Value::number(self.len(sc) as f64))));
        }

        Ok(None)
    }

    fn set_property(&self, _: PropertyKey, _: PropertyValue, _: &mut LocalScope) -> Result<(), Value> {
        Ok(())
    }

    fn delete_property(&self, _: PropertyKey, _: &mut LocalScope) -> Result<Unrooted, Value> {
        Ok(Unrooted::new(Value::undefined()))
    }

    fn set_prototype(&self, _: Value, _: &mut LocalScope) -> Result<(), Value> {
        Ok(())
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        Ok(sc.statics.string_prototype.into())
    }

    fn apply(
        &self,
        _: crate::gc::ObjectId,
        _: This,
        _: CallArgs,
        scope: &mut LocalScope,
    ) -> Result<Unrooted, Unrooted> {
        let v = self.res(scope).to_owned();
        throw!(scope, TypeError, "'{}' is not a function", v)
    }

    fn own_keys(&self, sc: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        let len = self.len(sc);
        Ok(array_like_keys(sc, len).collect())
    }

    fn type_of(&self, _: &Vm) -> Typeof {
        Typeof::String
    }

    fn internal_slots(&self, _: &Vm) -> Option<&dyn InternalSlots> {
        Some(self)
    }

    extract!(self);
}

impl InternalSlots for JsString {
    fn string_value(&self, _: &Vm) -> Option<JsString> {
        Some(*self)
    }
}
