use dash_proc_macro::Trace;

use crate::gc::interner::{StringInterner, Symbol};
use crate::localscope::LocalScope;
use crate::value::boxed::String as BoxedString;

use super::object::Object;
use super::ops::conversions::{PreferredType, ValueConversion};
use super::ops::equality::ValueEquality;
use super::primitive::PrimitiveCapabilities;
use super::Value;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Trace)]
pub struct JsString {
    sym: Symbol,
}

impl From<Symbol> for JsString {
    fn from(sym: Symbol) -> Self {
        Self { sym }
    }
}

impl JsString {
    pub fn sym(self) -> Symbol {
        self.sym
    }

    pub fn res<'a>(self, sc: &LocalScope<'a>) -> &'a str {
        sc.interner.resolve(self.sym)
    }

    pub fn len(self, sc: &mut LocalScope<'_>) -> usize {
        self.res(&sc).len()
    }
}

impl ValueEquality for JsString {
    fn lt(&self, other: &super::Value, sc: &mut LocalScope) -> Result<super::Value, super::Value> {
        let other = other.to_js_string(sc)?;
        Ok(Value::Boolean(self.res(&sc) < other.res(&sc)))
    }

    fn le(&self, other: &super::Value, sc: &mut LocalScope) -> Result<super::Value, super::Value> {
        let other = other.to_js_string(sc)?;
        Ok(Value::Boolean(self.res(&sc) <= other.res(&sc)))
    }

    fn gt(&self, other: &super::Value, sc: &mut LocalScope) -> Result<super::Value, super::Value> {
        let other = other.to_js_string(sc)?;
        Ok(Value::Boolean(self.res(&sc) > other.res(&sc)))
    }

    fn ge(&self, other: &super::Value, sc: &mut LocalScope) -> Result<super::Value, super::Value> {
        let other = other.to_js_string(sc)?;
        Ok(Value::Boolean(self.res(&sc) >= other.res(&sc)))
    }

    fn eq(&self, other: &super::Value, sc: &mut LocalScope) -> Result<super::Value, super::Value> {
        Ok(Value::Boolean(*self == other.to_js_string(sc)?))
    }

    fn strict_eq(&self, other: &Value, sc: &mut LocalScope) -> Result<super::Value, super::Value> {
        if let Value::String(other) = other {
            Ok(Value::Boolean(self == other))
        } else {
            Ok(Value::Boolean(false))
        }
    }
}

impl ValueConversion for JsString {
    fn to_primitive(&self, sc: &mut LocalScope, preferred_type: Option<PreferredType>) -> Result<Value, Value> {
        Ok(Value::String(self.clone()))
    }

    fn to_number(&self, sc: &mut LocalScope) -> Result<f64, Value> {
        Ok(self.res(sc).parse().unwrap_or(f64::NAN))
    }

    fn to_boolean(&self, sc: &mut LocalScope<'_>) -> Result<bool, Value> {
        Ok(!self.res(sc).is_empty())
    }

    fn to_js_string(&self, sc: &mut LocalScope) -> Result<JsString, Value> {
        Ok(self.clone())
    }

    fn length_of_array_like(&self, sc: &mut LocalScope) -> Result<usize, Value> {
        Ok(self.res(sc).len())
    }

    fn to_object(&self, sc: &mut LocalScope) -> Result<crate::gc::handle::Handle<dyn super::object::Object>, Value> {
        let bx = BoxedString::new(sc, self.clone());
        Ok(sc.register(bx))
    }
}

impl Object for JsString {
    fn get_own_property_descriptor(
        &self,
        sc: &mut LocalScope,
        key: super::object::PropertyKey,
    ) -> Result<Option<super::object::PropertyValue>, super::Unrooted> {
        todo!()
    }

    fn set_property(
        &self,
        sc: &mut LocalScope,
        key: super::object::PropertyKey,
        value: super::object::PropertyValue,
    ) -> Result<(), Value> {
        todo!()
    }

    fn delete_property(&self, sc: &mut LocalScope, key: super::object::PropertyKey) -> Result<super::Unrooted, Value> {
        todo!()
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        todo!()
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        todo!()
    }

    fn apply(
        &self,
        scope: &mut LocalScope,
        callee: crate::gc::handle::Handle<dyn Object>,
        this: Value,
        args: Vec<Value>,
    ) -> Result<super::Unrooted, super::Unrooted> {
        todo!()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        todo!()
    }

    fn own_keys(&self, sc: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        todo!()
    }
}

impl PrimitiveCapabilities for JsString {
    fn as_string(&self) -> Option<JsString> {
        Some(self.clone())
    }

    fn unbox(&self) -> Value {
        Value::String(self.clone())
    }
}
