use std::any::Any;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::iter;
use std::rc::Rc;

use crate::gc2::handle::Handle;
use crate::local::LocalScope;
use crate::throw;

use super::boxed::Boolean as BoxedBoolean;
use super::boxed::Number as BoxedNumber;
use super::boxed::String as BoxedString;
use super::boxed::Symbol as BoxedSymbol;
use super::object::Object;
use super::object::PropertyKey;
use super::object::PropertyValue;
use super::ops::abstractions::conversions::PreferredType;
use super::ops::abstractions::conversions::ValueConversion;
use super::ops::equality::ValueEquality;
use super::Typeof;
use super::Value;

pub const MAX_SAFE_INTEGER: u64 = 9007199254740991u64;
pub const MAX_SAFE_INTEGERF: f64 = 9007199254740991f64;

impl Object for f64 {
    fn get_own_property_descriptor(
        &self,
        _sc: &mut LocalScope,
        _key: PropertyKey,
    ) -> Result<Option<PropertyValue>, Value> {
        Ok(None)
    }

    fn set_property(
        &self,
        _sc: &mut LocalScope,
        _key: PropertyKey<'static>,
        _value: PropertyValue,
    ) -> Result<(), Value> {
        Ok(())
    }

    fn delete_property(&self, _sc: &mut LocalScope, _key: PropertyKey) -> Result<Value, Value> {
        Ok(Value::undefined())
    }

    fn set_prototype(&self, _sc: &mut LocalScope, _value: Value) -> Result<(), Value> {
        // TODO: Reflect.setPrototypeOf(this, value); should throw
        Ok(())
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        Ok(sc.statics.number_prototype.clone().into())
    }

    fn apply(
        &self,
        scope: &mut LocalScope,
        _callee: Handle<dyn Object>,
        _this: Value,
        _args: Vec<Value>,
    ) -> Result<Value, Value> {
        throw!(scope, TypeError, "number is not a function")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        Ok(Vec::new())
    }

    fn type_of(&self) -> Typeof {
        Typeof::Number
    }

    fn as_primitive_capable(&self) -> Option<&dyn PrimitiveCapabilities> {
        Some(self)
    }
}

impl Object for bool {
    fn get_own_property_descriptor(
        &self,
        _sc: &mut LocalScope,
        _key: PropertyKey,
    ) -> Result<Option<PropertyValue>, Value> {
        Ok(None)
    }

    fn set_property(
        &self,
        _sc: &mut LocalScope,
        _key: PropertyKey<'static>,
        _value: PropertyValue,
    ) -> Result<(), Value> {
        Ok(())
    }

    fn delete_property(&self, _sc: &mut LocalScope, _key: PropertyKey) -> Result<Value, Value> {
        Ok(Value::undefined())
    }

    fn set_prototype(&self, _sc: &mut LocalScope, _value: Value) -> Result<(), Value> {
        Ok(())
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        Ok(sc.statics.boolean_prototype.clone().into())
    }

    fn apply(
        &self,
        scope: &mut LocalScope,
        _callee: Handle<dyn Object>,
        _this: Value,
        _args: Vec<Value>,
    ) -> Result<Value, Value> {
        throw!(scope, TypeError, "boolean is not a function")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        Ok(Vec::new())
    }

    fn type_of(&self) -> Typeof {
        Typeof::Boolean
    }

    fn as_primitive_capable(&self) -> Option<&dyn PrimitiveCapabilities> {
        Some(self)
    }
}

// TODO: impl<T: Deref<Target=O>, O: Object> Object for T  possible?
impl Object for Rc<str> {
    fn get_own_property_descriptor(
        &self,
        sc: &mut LocalScope,
        key: PropertyKey,
    ) -> Result<Option<PropertyValue>, Value> {
        str::get_own_property_descriptor(self, sc, key.clone())
    }

    fn set_property(
        &self,
        _sc: &mut LocalScope,
        _key: PropertyKey<'static>,
        _value: PropertyValue,
    ) -> Result<(), Value> {
        Ok(())
    }

    fn delete_property(&self, _sc: &mut LocalScope, _key: PropertyKey) -> Result<Value, Value> {
        Ok(Value::undefined())
    }

    fn set_prototype(&self, _sc: &mut LocalScope, _value: Value) -> Result<(), Value> {
        Ok(())
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        Ok(sc.statics.string_prototype.clone().into())
    }

    fn apply(
        &self,
        scope: &mut LocalScope,
        _callee: Handle<dyn Object>,
        _this: Value,
        _args: Vec<Value>,
    ) -> Result<Value, Value> {
        throw!(scope, TypeError, "string is not a function")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        str::own_keys(self)
    }

    fn type_of(&self) -> Typeof {
        str::type_of(self)
    }

    fn as_primitive_capable(&self) -> Option<&dyn PrimitiveCapabilities> {
        Some(self)
    }
}

pub fn array_like_keys(len: usize) -> impl Iterator<Item = Value> {
    (0..len)
        .map(|i| i.to_string())
        .chain(iter::once_with(|| "length".to_string()))
        .map(|x| Value::String(x.as_str().into()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Undefined;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Null;

impl Object for Undefined {
    fn get_own_property_descriptor(
        &self,
        sc: &mut LocalScope,
        key: PropertyKey,
    ) -> Result<Option<PropertyValue>, Value> {
        throw!(sc, TypeError, "Cannot read property {:?} of undefined", key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey<'static>, _value: PropertyValue) -> Result<(), Value> {
        throw!(sc, TypeError, "Cannot set property {:?} of undefined", key)
    }

    fn delete_property(&self, _sc: &mut LocalScope, _key: PropertyKey) -> Result<Value, Value> {
        Ok(Value::undefined())
    }

    fn set_prototype(&self, sc: &mut LocalScope, _value: Value) -> Result<(), Value> {
        throw!(sc, TypeError, "Cannot set prototype of undefined")
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        throw!(sc, TypeError, "Cannot get prototype of undefined")
    }

    fn apply(
        &self,
        sc: &mut LocalScope,
        _callee: Handle<dyn Object>,
        _this: Value,
        _args: Vec<Value>,
    ) -> Result<Value, Value> {
        throw!(sc, TypeError, "undefined is not a function")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        Ok(Vec::new())
    }

    fn type_of(&self) -> Typeof {
        Typeof::Undefined
    }

    fn as_primitive_capable(&self) -> Option<&dyn PrimitiveCapabilities> {
        Some(self)
    }
}

impl Object for Null {
    fn get_own_property_descriptor(
        &self,
        sc: &mut LocalScope,
        key: PropertyKey,
    ) -> Result<Option<PropertyValue>, Value> {
        throw!(sc, TypeError, "Cannot read property {:?} of null", key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey<'static>, _value: PropertyValue) -> Result<(), Value> {
        throw!(sc, TypeError, "Cannot set property {:?} of null", key)
    }

    fn delete_property(&self, _sc: &mut LocalScope, _key: PropertyKey) -> Result<Value, Value> {
        Ok(Value::undefined())
    }

    fn set_prototype(&self, sc: &mut LocalScope, _value: Value) -> Result<(), Value> {
        throw!(sc, TypeError, "Cannot set prototype of null")
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        throw!(sc, TypeError, "Cannot get prototype of null")
    }

    fn apply(
        &self,
        sc: &mut LocalScope,
        _callee: Handle<dyn Object>,
        _this: Value,
        _args: Vec<Value>,
    ) -> Result<Value, Value> {
        throw!(sc, TypeError, "null is not a function")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        Ok(Vec::new())
    }

    fn as_primitive_capable(&self) -> Option<&dyn PrimitiveCapabilities> {
        Some(self)
    }
}

impl Object for str {
    fn get_own_property_descriptor(
        &self,
        _sc: &mut LocalScope,
        key: PropertyKey,
    ) -> Result<Option<PropertyValue>, Value> {
        if let PropertyKey::String(st) = key {
            if st == "length" {
                return Ok(Some(PropertyValue::static_default(Value::number(self.len() as f64))));
            }

            if let Ok(index) = st.parse::<usize>() {
                let bytes = self.as_bytes();
                if let Some(&byte) = bytes.get(index) {
                    return Ok(Some(PropertyValue::static_default(Value::String(
                        (byte as char).to_string().into(),
                    ))));
                }
            }
        }

        Ok(None)
    }

    fn set_property(
        &self,
        _sc: &mut LocalScope,
        _key: PropertyKey<'static>,
        _value: PropertyValue,
    ) -> Result<(), Value> {
        Ok(())
    }

    fn delete_property(&self, _sc: &mut LocalScope, _key: PropertyKey) -> Result<Value, Value> {
        Ok(Value::undefined())
    }

    fn set_prototype(&self, _sc: &mut LocalScope, _value: Value) -> Result<(), Value> {
        Ok(())
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        Ok(sc.statics.string_prototype.clone().into())
    }

    fn apply(
        &self,
        scope: &mut LocalScope,
        _callee: Handle<dyn Object>,
        _this: Value,
        _args: Vec<Value>,
    ) -> Result<Value, Value> {
        throw!(scope, TypeError, "string is not a function")
    }

    fn as_any(&self) -> &dyn Any {
        panic!("cannot convert string to any")
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        Ok(array_like_keys(self.len()).collect())
    }

    fn type_of(&self) -> Typeof {
        Typeof::String
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Symbol(Rc<str>);

impl Symbol {
    pub fn new(description: Rc<str>) -> Self {
        Symbol(description)
    }
}

impl Object for Symbol {
    fn get_own_property_descriptor(
        &self,
        _sc: &mut LocalScope,
        _key: PropertyKey,
    ) -> Result<Option<PropertyValue>, Value> {
        Ok(None)
    }

    fn set_property(
        &self,
        _sc: &mut LocalScope,
        _key: PropertyKey<'static>,
        _value: PropertyValue,
    ) -> Result<(), Value> {
        Ok(())
    }

    fn delete_property(&self, _sc: &mut LocalScope, _key: PropertyKey) -> Result<Value, Value> {
        Ok(Value::undefined())
    }

    fn set_prototype(&self, _sc: &mut LocalScope, _value: Value) -> Result<(), Value> {
        Ok(())
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        Ok(sc.statics.symbol_prototype.clone().into())
    }

    fn apply(
        &self,
        scope: &mut LocalScope,
        _callee: Handle<dyn Object>,
        _this: Value,
        _args: Vec<Value>,
    ) -> Result<Value, Value> {
        throw!(scope, TypeError, "symbol is not a function")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        Ok(Vec::new())
    }

    fn type_of(&self) -> Typeof {
        Typeof::Symbol
    }

    fn as_primitive_capable(&self) -> Option<&dyn PrimitiveCapabilities> {
        Some(self)
    }
}

pub trait PrimitiveCapabilities: ValueConversion + ValueEquality + std::fmt::Debug {
    fn as_string(&self) -> Option<Rc<str>> {
        None
    }
    fn as_number(&self) -> Option<f64> {
        None
    }
    fn as_bool(&self) -> Option<bool> {
        None
    }
    fn is_undefined(&self) -> bool {
        false
    }
    fn is_null(&self) -> bool {
        false
    }
    fn unbox(&self) -> Value;
}

impl PrimitiveCapabilities for f64 {
    fn as_number(&self) -> Option<f64> {
        Some(*self)
    }

    fn unbox(&self) -> Value {
        Value::number(*self)
    }
}

impl ValueEquality for f64 {
    fn lt(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        other.to_number(sc).map(|other| Value::Boolean(*self < other))
    }

    fn le(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        other.to_number(sc).map(|other| Value::Boolean(*self <= other))
    }

    fn gt(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        other.to_number(sc).map(|other| Value::Boolean(*self > other))
    }

    fn ge(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        other.to_number(sc).map(|other| Value::Boolean(*self >= other))
    }

    fn eq(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        other.to_number(sc).map(|other| Value::Boolean(*self == other))
    }

    fn strict_eq(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        ValueEquality::eq(self, other, sc)
    }
}

impl ValueConversion for f64 {
    fn to_primitive(&self, _sc: &mut LocalScope, _preferred_type: Option<PreferredType>) -> Result<Value, Value> {
        Ok(Value::number(*self))
    }

    fn to_number(&self, _sc: &mut LocalScope) -> Result<f64, Value> {
        Ok(*self)
    }

    fn to_boolean(&self) -> Result<bool, Value> {
        Ok(*self != 0.0 && !self.is_nan())
    }

    fn to_string(&self, _sc: &mut LocalScope) -> Result<Rc<str>, Value> {
        // TODO: optimize
        Ok(ToString::to_string(self).into())
    }

    fn length_of_array_like(&self, _sc: &mut LocalScope) -> Result<usize, Value> {
        todo!() // TODO
    }

    fn to_object(&self, sc: &mut LocalScope) -> Result<Handle<dyn Object>, Value> {
        let num = BoxedNumber::new(sc, *self);
        Ok(sc.register(num))
    }
}

impl PrimitiveCapabilities for bool {
    fn as_bool(&self) -> Option<bool> {
        Some(*self)
    }

    fn unbox(&self) -> Value {
        Value::Boolean(*self)
    }
}

impl ValueEquality for bool {
    fn lt(&self, other: &Value, _sc: &mut LocalScope) -> Result<Value, Value> {
        other
            .to_boolean()
            .map(|other| Value::Boolean((*self as u8) < other as u8))
    }

    fn le(&self, other: &Value, _sc: &mut LocalScope) -> Result<Value, Value> {
        other
            .to_boolean()
            .map(|other| Value::Boolean((*self as u8) <= other as u8))
    }

    fn gt(&self, other: &Value, _sc: &mut LocalScope) -> Result<Value, Value> {
        other
            .to_boolean()
            .map(|other| Value::Boolean((*self as u8) > other as u8))
    }

    fn ge(&self, other: &Value, _sc: &mut LocalScope) -> Result<Value, Value> {
        other
            .to_boolean()
            .map(|other| Value::Boolean((*self as u8) >= other as u8))
    }

    fn eq(&self, other: &Value, _sc: &mut LocalScope) -> Result<Value, Value> {
        other.to_boolean().map(|other| Value::Boolean(*self == other))
    }

    fn strict_eq(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        ValueEquality::eq(self, other, sc)
    }
}

impl ValueConversion for bool {
    fn to_primitive(&self, _sc: &mut LocalScope, _preferred_type: Option<PreferredType>) -> Result<Value, Value> {
        Ok(Value::Boolean(*self))
    }

    fn to_number(&self, _sc: &mut LocalScope) -> Result<f64, Value> {
        Ok(*self as u8 as f64)
    }

    fn to_boolean(&self) -> Result<bool, Value> {
        Ok(*self)
    }

    fn to_string(&self, sc: &mut LocalScope) -> Result<Rc<str>, Value> {
        Ok(if *self {
            sc.statics().get_true()
        } else {
            sc.statics().get_false()
        })
    }

    fn length_of_array_like(&self, _sc: &mut LocalScope) -> Result<usize, Value> {
        todo!() // TODO
    }

    fn to_object(&self, sc: &mut LocalScope) -> Result<Handle<dyn Object>, Value> {
        let bool = BoxedBoolean::new(sc, *self);
        Ok(sc.register(bool))
    }
}

impl PrimitiveCapabilities for Rc<str> {
    fn as_string(&self) -> Option<Rc<str>> {
        Some(self.clone())
    }

    fn unbox(&self) -> Value {
        Value::String(Rc::clone(self))
    }
}

impl ValueEquality for Rc<str> {
    fn lt(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        other.to_string(sc).map(|other| Value::Boolean(self < &other))
    }

    fn le(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        other.to_string(sc).map(|other| Value::Boolean(self <= &other))
    }

    fn gt(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        other.to_string(sc).map(|other| Value::Boolean(self > &other))
    }

    fn ge(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        other.to_string(sc).map(|other| Value::Boolean(self >= &other))
    }

    fn eq(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        other.to_string(sc).map(|other| Value::Boolean(self == &other))
    }

    fn strict_eq(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        ValueEquality::eq(self, other, sc)
    }
}

impl ValueConversion for Rc<str> {
    fn to_primitive(&self, _sc: &mut LocalScope, _preferred_type: Option<PreferredType>) -> Result<Value, Value> {
        Ok(Value::String(Rc::clone(self)))
    }

    fn to_number(&self, _sc: &mut LocalScope) -> Result<f64, Value> {
        Ok(self.parse().unwrap_or(f64::NAN))
    }

    fn to_boolean(&self) -> Result<bool, Value> {
        Ok(!self.is_empty())
    }

    fn to_string(&self, _sc: &mut LocalScope) -> Result<Rc<str>, Value> {
        Ok(Rc::clone(self))
    }

    fn length_of_array_like(&self, _sc: &mut LocalScope) -> Result<usize, Value> {
        Ok(self.len())
    }

    fn to_object(&self, sc: &mut LocalScope) -> Result<Handle<dyn Object>, Value> {
        let bool = BoxedString::new(sc, self.clone());
        Ok(sc.register(bool))
    }
}

impl PrimitiveCapabilities for Undefined {
    fn is_undefined(&self) -> bool {
        true
    }

    fn unbox(&self) -> Value {
        Value::undefined()
    }
}

impl ValueEquality for Undefined {
    fn lt(&self, _other: &Value, _sc: &mut LocalScope) -> Result<Value, Value> {
        // TODO: invoke toString
        Ok(Value::Boolean(false))
    }

    fn le(&self, _other: &Value, _sc: &mut LocalScope) -> Result<Value, Value> {
        Ok(Value::Boolean(false))
    }

    fn gt(&self, _other: &Value, _sc: &mut LocalScope) -> Result<Value, Value> {
        Ok(Value::Boolean(false))
    }

    fn ge(&self, _other: &Value, _sc: &mut LocalScope) -> Result<Value, Value> {
        Ok(Value::Boolean(false))
    }

    fn eq(&self, other: &Value, _sc: &mut LocalScope) -> Result<Value, Value> {
        match other {
            Value::Undefined(_) => Ok(Value::Boolean(true)),
            Value::Object(o) | Value::External(o) => Ok(Value::Boolean(
                o.as_primitive_capable().map_or(false, |p| p.is_undefined()),
            )),
            _ => Ok(Value::Boolean(false)),
        }
    }

    fn strict_eq(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        ValueEquality::eq(self, other, sc)
    }
}

impl ValueConversion for Undefined {
    fn to_primitive(&self, _sc: &mut LocalScope, _preferred_type: Option<PreferredType>) -> Result<Value, Value> {
        Ok(Value::undefined())
    }

    fn to_number(&self, _sc: &mut LocalScope) -> Result<f64, Value> {
        Ok(f64::NAN)
    }

    fn to_boolean(&self) -> Result<bool, Value> {
        Ok(false)
    }

    fn to_string(&self, sc: &mut LocalScope) -> Result<Rc<str>, Value> {
        Ok(sc.statics().undefined_str())
    }

    fn length_of_array_like(&self, _sc: &mut LocalScope) -> Result<usize, Value> {
        todo!() // TODO: throw?
    }

    fn to_object(&self, sc: &mut LocalScope) -> Result<Handle<dyn Object>, Value> {
        throw!(sc, TypeError, "Cannot convert undefined to object")
    }
}

impl PrimitiveCapabilities for Null {
    fn is_null(&self) -> bool {
        true
    }

    fn unbox(&self) -> Value {
        Value::null()
    }
}

impl ValueEquality for Null {
    fn lt(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        other.to_number(sc).map(|other| Value::Boolean(0.0 < other))
    }

    fn le(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        other.to_number(sc).map(|other| Value::Boolean(0.0 <= other))
    }

    fn gt(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        other.to_number(sc).map(|other| Value::Boolean(0.0 > other))
    }

    fn ge(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        other.to_number(sc).map(|other| Value::Boolean(0.0 >= other))
    }

    fn eq(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        other.to_number(sc).map(|other| Value::Boolean(0.0 == other))
    }

    fn strict_eq(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        ValueEquality::eq(self, other, sc)
    }
}

impl ValueConversion for Null {
    fn to_primitive(&self, _sc: &mut LocalScope, _preferred_type: Option<PreferredType>) -> Result<Value, Value> {
        Ok(Value::null())
    }

    fn to_number(&self, _sc: &mut LocalScope) -> Result<f64, Value> {
        Ok(0.0)
    }

    fn to_boolean(&self) -> Result<bool, Value> {
        Ok(false)
    }

    fn to_string(&self, sc: &mut LocalScope) -> Result<Rc<str>, Value> {
        Ok(sc.statics().null_str())
    }

    fn length_of_array_like(&self, _sc: &mut LocalScope) -> Result<usize, Value> {
        todo!() // TODO: throw?
    }

    fn to_object(&self, sc: &mut LocalScope) -> Result<Handle<dyn Object>, Value> {
        throw!(sc, TypeError, "Cannot convert null to object");
    }
}

impl PrimitiveCapabilities for Symbol {
    fn unbox(&self) -> Value {
        Value::Symbol(self.clone())
    }
}

impl ValueEquality for Symbol {
    fn lt(&self, _other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        throw!(sc, TypeError, "Cannot convert a Symbol value to a number")
    }

    fn le(&self, _other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        throw!(sc, TypeError, "Cannot convert a Symbol value to a number")
    }

    fn gt(&self, _other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        throw!(sc, TypeError, "Cannot convert a Symbol value to a number")
    }

    fn ge(&self, _other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        throw!(sc, TypeError, "Cannot convert a Symbol value to a number")
    }

    fn eq(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        other.to_number(sc).map(|other| Value::Boolean(0.0 == other))
    }

    fn strict_eq(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        ValueEquality::eq(self, other, sc)
    }
}

impl ValueConversion for Symbol {
    fn to_primitive(&self, _sc: &mut LocalScope, _preferred_type: Option<PreferredType>) -> Result<Value, Value> {
        Ok(Value::Symbol(self.clone()))
    }

    fn to_number(&self, sc: &mut LocalScope) -> Result<f64, Value> {
        throw!(sc, TypeError, "Cannot convert symbol to number");
    }

    fn to_boolean(&self) -> Result<bool, Value> {
        Ok(true)
    }

    fn to_string(&self, sc: &mut LocalScope) -> Result<Rc<str>, Value> {
        throw!(sc, TypeError, "Cannot convert symbol to string");
    }

    fn length_of_array_like(&self, _sc: &mut LocalScope) -> Result<usize, Value> {
        todo!() // TODO: throw?
    }

    fn to_object(&self, sc: &mut LocalScope) -> Result<Handle<dyn Object>, Value> {
        let sym = BoxedSymbol::new(sc, self.clone());
        Ok(sc.register(sym))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct Number(pub f64);

impl Eq for Number {}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Hash for Number {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state)
    }
}

impl Object for Number {
    fn get_own_property_descriptor(
        &self,
        sc: &mut LocalScope,
        key: PropertyKey,
    ) -> Result<Option<PropertyValue>, Value> {
        self.0.get_own_property_descriptor(sc, key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey<'static>, value: PropertyValue) -> Result<(), Value> {
        self.0.set_property(sc, key, value)
    }

    fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        self.0.delete_property(sc, key)
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        self.0.set_prototype(sc, value)
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        self.0.get_prototype(sc)
    }

    fn apply(
        &self,
        scope: &mut LocalScope,
        callee: Handle<dyn Object>,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        self.0.apply(scope, callee, this, args)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        self.0.own_keys()
    }

    fn type_of(&self) -> Typeof {
        self.0.type_of()
    }

    fn as_primitive_capable(&self) -> Option<&dyn PrimitiveCapabilities> {
        Some(self)
    }
}
impl PrimitiveCapabilities for Number {
    fn as_number(&self) -> Option<f64> {
        Some(self.0)
    }

    fn unbox(&self) -> Value {
        Value::Number(*self)
    }
}

impl ValueEquality for Number {
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
        ValueEquality::strict_eq(&self.0, other, sc)
    }
}

impl ValueConversion for Number {
    fn to_primitive(&self, sc: &mut LocalScope, preferred_type: Option<PreferredType>) -> Result<Value, Value> {
        self.0.to_primitive(sc, preferred_type)
    }

    fn to_number(&self, sc: &mut LocalScope) -> Result<f64, Value> {
        self.0.to_number(sc)
    }

    fn to_boolean(&self) -> Result<bool, Value> {
        self.0.to_boolean()
    }

    fn to_string(&self, sc: &mut LocalScope) -> Result<Rc<str>, Value> {
        ValueConversion::to_string(&self.0, sc)
    }

    fn length_of_array_like(&self, sc: &mut LocalScope) -> Result<usize, Value> {
        self.0.length_of_array_like(sc)
    }

    fn to_object(&self, sc: &mut LocalScope) -> Result<Handle<dyn Object>, Value> {
        self.0.to_object(sc)
    }
}
