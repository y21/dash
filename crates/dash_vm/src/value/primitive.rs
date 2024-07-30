use std::any::Any;
use std::hash::{Hash, Hasher};
use std::{fmt, iter};

use dash_middle::interner;
use dash_proc_macro::Trace;

use crate::gc::handle::Handle;
use crate::gc::interner::sym;
use crate::localscope::LocalScope;
use crate::throw;
use crate::util::{intern_f64, Captures};

use super::boxed::{Boolean as BoxedBoolean, Number as BoxedNumber, Symbol as BoxedSymbol};
use super::object::{Object, PropertyKey, PropertyValue};
use super::ops::conversions::{PreferredType, ValueConversion};
use super::string::JsString;
use super::{Typeof, Unrooted, Value};

pub const MIN_SAFE_INTEGER: i64 = -9007199254740991i64;
pub const MAX_SAFE_INTEGER: i64 = 9007199254740991i64;
pub const MAX_SAFE_INTEGERF: f64 = 9007199254740991f64;
pub const MIN_SAFE_INTEGERF: f64 = -9007199254740991f64;

impl Object for f64 {
    fn get_own_property_descriptor(
        &self,
        _sc: &mut LocalScope,
        _key: PropertyKey,
    ) -> Result<Option<PropertyValue>, Unrooted> {
        Ok(None)
    }

    fn set_property(&self, _sc: &mut LocalScope, _key: PropertyKey, _value: PropertyValue) -> Result<(), Value> {
        Ok(())
    }

    fn delete_property(&self, _sc: &mut LocalScope, _key: PropertyKey) -> Result<Unrooted, Value> {
        Ok(Unrooted::new(Value::undefined()))
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
        _callee: Handle,
        _this: Value,
        _args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        throw!(scope, TypeError, "number is not a function")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn own_keys(&self, _: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
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
    ) -> Result<Option<PropertyValue>, Unrooted> {
        Ok(None)
    }

    fn set_property(&self, _sc: &mut LocalScope, _key: PropertyKey, _value: PropertyValue) -> Result<(), Value> {
        Ok(())
    }

    fn delete_property(&self, _sc: &mut LocalScope, _key: PropertyKey) -> Result<Unrooted, Value> {
        Ok(Unrooted::new(Value::undefined()))
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
        _callee: Handle,
        _this: Value,
        _args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        throw!(scope, TypeError, "boolean is not a function")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn own_keys(&self, _: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        Ok(Vec::new())
    }

    fn type_of(&self) -> Typeof {
        Typeof::Boolean
    }

    fn as_primitive_capable(&self) -> Option<&dyn PrimitiveCapabilities> {
        Some(self)
    }
}

// // TODO: impl<T: Deref<Target=O>, O: Object> Object for T  possible?
// impl Object for Rc<str> {
//     fn get_own_property_descriptor(
//         &self,
//         sc: &mut LocalScope,
//         key: PropertyKey,
//     ) -> Result<Option<PropertyValue>, Unrooted> {
//         str::get_own_property_descriptor(self, sc, key.clone())
//     }

//     fn set_property(
//         &self,
//         _sc: &mut LocalScope,
//         _key: PropertyKey<'static>,
//         _value: PropertyValue,
//     ) -> Result<(), Value> {
//         Ok(())
//     }

//     fn delete_property(&self, _sc: &mut LocalScope, _key: PropertyKey) -> Result<Unrooted, Value> {
//         Ok(Unrooted::new(Value::undefined()))
//     }

//     fn set_prototype(&self, _sc: &mut LocalScope, _value: Value) -> Result<(), Value> {
//         Ok(())
//     }

//     fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
//         Ok(sc.statics.string_prototype.clone().into())
//     }

//     fn apply(
//         &self,
//         scope: &mut LocalScope,
//         _callee: Handle,
//         _this: Value,
//         _args: Vec<Value>,
//     ) -> Result<Unrooted, Unrooted> {
//         throw!(scope, TypeError, "string is not a function")
//     }

//     fn as_any(&self) -> &dyn Any {
//         self
//     }

//     fn own_keys(&self, sc: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
//         str::own_keys(self, sc)
//     }

//     fn type_of(&self) -> Typeof {
//         str::type_of(self)
//     }

//     fn as_primitive_capable(&self) -> Option<&dyn PrimitiveCapabilities> {
//         Some(self)
//     }
// }

pub fn array_like_keys<'a, 'b>(
    sc: &'a mut LocalScope<'b>,
    len: usize,
) -> impl Iterator<Item = Value> + Captures<'a> + Captures<'b> {
    (0..len)
        .map(|i| sc.intern_usize(i))
        .chain(iter::once_with(|| sym::length))
        .map(|x| Value::String(x.into()))
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
    ) -> Result<Option<PropertyValue>, Unrooted> {
        let key = match key {
            PropertyKey::String(s) => s.res(sc).to_owned(),
            PropertyKey::Symbol(s) => sc.interner.resolve(s.sym()).to_owned(),
        };
        throw!(sc, TypeError, "Cannot read property {} of undefined", key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey, _value: PropertyValue) -> Result<(), Value> {
        throw!(sc, TypeError, "Cannot set property {:?} of undefined", key)
    }

    fn delete_property(&self, _sc: &mut LocalScope, _key: PropertyKey) -> Result<Unrooted, Value> {
        Ok(Unrooted::new(Value::undefined()))
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
        _callee: Handle,
        _this: Value,
        _args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        throw!(sc, TypeError, "undefined is not a function")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn own_keys(&self, _: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
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
    ) -> Result<Option<PropertyValue>, Unrooted> {
        let key = match key {
            PropertyKey::String(s) => s.res(sc).to_owned(),
            PropertyKey::Symbol(s) => sc.interner.resolve(s.sym()).to_owned(),
        };
        throw!(sc, TypeError, "Cannot read property {} of null", key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey, _value: PropertyValue) -> Result<(), Value> {
        throw!(sc, TypeError, "Cannot set property {:?} of null", key)
    }

    fn delete_property(&self, _sc: &mut LocalScope, _key: PropertyKey) -> Result<Unrooted, Value> {
        Ok(Unrooted::new(Value::undefined()))
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
        _callee: Handle,
        _this: Value,
        _args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        throw!(sc, TypeError, "null is not a function")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn own_keys(&self, _: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        Ok(Vec::new())
    }

    fn as_primitive_capable(&self) -> Option<&dyn PrimitiveCapabilities> {
        Some(self)
    }
}

// impl Object for str {
//     fn get_own_property_descriptor(
//         &self,
//         sc: &mut LocalScope,
//         key: PropertyKey,
//     ) -> Result<Option<PropertyValue>, Unrooted> {
//         if let PropertyKey::String(st) = key {
//             if st.sym() == sym::length {
//                 return Ok(Some(PropertyValue::static_default(Value::number(self.len() as f64))));
//             }

//             if let Ok(index) = st.res(sc).parse::<usize>() {
//                 let bytes = self.as_bytes();
//                 if let Some(&byte) = bytes.get(index) {
//                     let s = sc.intern((byte as char).to_string().as_ref());
//                     return Ok(Some(PropertyValue::static_default(Value::String(s.into()))));
//                 }
//             }
//         }

//         Ok(None)
//     }

//     fn set_property(&self, _sc: &mut LocalScope, _key: PropertyKey, _value: PropertyValue) -> Result<(), Value> {
//         Ok(())
//     }

//     fn delete_property(&self, _sc: &mut LocalScope, _key: PropertyKey) -> Result<Unrooted, Value> {
//         Ok(Unrooted::new(Value::undefined()))
//     }

//     fn set_prototype(&self, _sc: &mut LocalScope, _value: Value) -> Result<(), Value> {
//         Ok(())
//     }

//     fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
//         Ok(sc.statics.string_prototype.clone().into())
//     }

//     fn apply(
//         &self,
//         scope: &mut LocalScope,
//         _callee: Handle,
//         _this: Value,
//         _args: Vec<Value>,
//     ) -> Result<Unrooted, Unrooted> {
//         throw!(scope, TypeError, "string is not a function")
//     }

//     fn as_any(&self) -> &dyn Any {
//         panic!("cannot convert string to any")
//     }

//     fn own_keys(&self, sc: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
//         Ok(array_like_keys(self.len()).collect())
//     }

//     fn type_of(&self) -> Typeof {
//         Typeof::String
//     }
// }

// TODO: rename to JsSymbol
#[derive(Debug, Clone, Hash, PartialEq, Eq, Trace)]
pub struct Symbol {
    description: JsString,
}

impl Symbol {
    pub fn sym(&self) -> interner::Symbol {
        self.description.sym()
    }

    pub fn new(description: JsString) -> Self {
        Symbol { description }
    }
}

impl Object for Symbol {
    fn get_own_property_descriptor(
        &self,
        _sc: &mut LocalScope,
        _key: PropertyKey,
    ) -> Result<Option<PropertyValue>, Unrooted> {
        Ok(None)
    }

    fn set_property(&self, _sc: &mut LocalScope, _key: PropertyKey, _value: PropertyValue) -> Result<(), Value> {
        Ok(())
    }

    fn delete_property(&self, _sc: &mut LocalScope, _key: PropertyKey) -> Result<Unrooted, Value> {
        Ok(Unrooted::new(Value::undefined()))
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
        _callee: Handle,
        _this: Value,
        _args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        throw!(scope, TypeError, "symbol is not a function")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn own_keys(&self, _: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        Ok(Vec::new())
    }

    fn type_of(&self) -> Typeof {
        Typeof::Symbol
    }

    fn as_primitive_capable(&self) -> Option<&dyn PrimitiveCapabilities> {
        Some(self)
    }
}

pub trait PrimitiveCapabilities: ValueConversion + std::fmt::Debug {
    fn as_string(&self) -> Option<JsString> {
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

impl ValueConversion for f64 {
    fn to_primitive(&self, _sc: &mut LocalScope, _preferred_type: Option<PreferredType>) -> Result<Value, Value> {
        Ok(Value::number(*self))
    }

    fn to_number(&self, _sc: &mut LocalScope) -> Result<f64, Value> {
        Ok(*self)
    }

    fn to_boolean(&self, _: &mut LocalScope<'_>) -> Result<bool, Value> {
        Ok(*self != 0.0 && !self.is_nan())
    }

    fn to_js_string(&self, sc: &mut LocalScope) -> Result<JsString, Value> {
        Ok(intern_f64(sc, *self).into())
    }

    fn length_of_array_like(&self, _sc: &mut LocalScope) -> Result<usize, Value> {
        todo!() // TODO
    }

    fn to_object(&self, sc: &mut LocalScope) -> Result<Handle, Value> {
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

impl ValueConversion for bool {
    fn to_primitive(&self, _sc: &mut LocalScope, _preferred_type: Option<PreferredType>) -> Result<Value, Value> {
        Ok(Value::Boolean(*self))
    }

    fn to_number(&self, _sc: &mut LocalScope) -> Result<f64, Value> {
        Ok(*self as u8 as f64)
    }

    fn to_boolean(&self, _sc: &mut LocalScope<'_>) -> Result<bool, Value> {
        Ok(*self)
    }

    fn to_js_string(&self, _: &mut LocalScope) -> Result<JsString, Value> {
        Ok(if *self { sym::true_.into() } else { sym::false_.into() })
    }

    fn length_of_array_like(&self, _sc: &mut LocalScope) -> Result<usize, Value> {
        todo!() // TODO
    }

    fn to_object(&self, sc: &mut LocalScope) -> Result<Handle, Value> {
        let bool = BoxedBoolean::new(sc, *self);
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

impl ValueConversion for Undefined {
    fn to_primitive(&self, _sc: &mut LocalScope, _preferred_type: Option<PreferredType>) -> Result<Value, Value> {
        Ok(Value::undefined())
    }

    fn to_number(&self, _sc: &mut LocalScope) -> Result<f64, Value> {
        Ok(f64::NAN)
    }

    fn to_boolean(&self, _sc: &mut LocalScope<'_>) -> Result<bool, Value> {
        Ok(false)
    }

    fn to_js_string(&self, _: &mut LocalScope) -> Result<JsString, Value> {
        Ok(sym::undefined.into())
    }

    fn length_of_array_like(&self, _sc: &mut LocalScope) -> Result<usize, Value> {
        todo!() // TODO: throw?
    }

    fn to_object(&self, sc: &mut LocalScope) -> Result<Handle, Value> {
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

impl ValueConversion for Null {
    fn to_primitive(&self, _sc: &mut LocalScope, _preferred_type: Option<PreferredType>) -> Result<Value, Value> {
        Ok(Value::null())
    }

    fn to_number(&self, _sc: &mut LocalScope) -> Result<f64, Value> {
        Ok(0.0)
    }

    fn to_boolean(&self, _sc: &mut LocalScope<'_>) -> Result<bool, Value> {
        Ok(false)
    }

    fn to_js_string(&self, _: &mut LocalScope) -> Result<JsString, Value> {
        Ok(sym::null.into())
    }

    fn length_of_array_like(&self, _sc: &mut LocalScope) -> Result<usize, Value> {
        todo!() // TODO: throw?
    }

    fn to_object(&self, sc: &mut LocalScope) -> Result<Handle, Value> {
        throw!(sc, TypeError, "Cannot convert null to object");
    }
}

impl PrimitiveCapabilities for Symbol {
    fn unbox(&self) -> Value {
        Value::Symbol(self.clone())
    }
}

impl ValueConversion for Symbol {
    fn to_primitive(&self, _sc: &mut LocalScope, _preferred_type: Option<PreferredType>) -> Result<Value, Value> {
        Ok(Value::Symbol(self.clone()))
    }

    fn to_number(&self, sc: &mut LocalScope) -> Result<f64, Value> {
        throw!(sc, TypeError, "Cannot convert symbol to number");
    }

    fn to_boolean(&self, _: &mut LocalScope<'_>) -> Result<bool, Value> {
        Ok(true)
    }

    fn to_js_string(&self, sc: &mut LocalScope) -> Result<JsString, Value> {
        throw!(sc, TypeError, "Cannot convert symbol to string");
    }

    fn length_of_array_like(&self, _sc: &mut LocalScope) -> Result<usize, Value> {
        todo!() // TODO: throw?
    }

    fn to_object(&self, sc: &mut LocalScope) -> Result<Handle, Value> {
        let sym = BoxedSymbol::new(sc, self.clone());
        Ok(sc.register(sym))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct Number(pub f64);

impl From<bool> for Number {
    fn from(value: bool) -> Self {
        match value {
            true => Self(1.0),
            false => Self(0.0),
        }
    }
}

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
    ) -> Result<Option<PropertyValue>, Unrooted> {
        self.0.get_own_property_descriptor(sc, key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey, value: PropertyValue) -> Result<(), Value> {
        self.0.set_property(sc, key, value)
    }

    fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Unrooted, Value> {
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
        callee: Handle,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        self.0.apply(scope, callee, this, args)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn own_keys(&self, sc: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        self.0.own_keys(sc)
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

impl ValueConversion for Number {
    fn to_primitive(&self, sc: &mut LocalScope, preferred_type: Option<PreferredType>) -> Result<Value, Value> {
        self.0.to_primitive(sc, preferred_type)
    }

    fn to_number(&self, sc: &mut LocalScope) -> Result<f64, Value> {
        self.0.to_number(sc)
    }

    fn to_boolean(&self, sc: &mut LocalScope<'_>) -> Result<bool, Value> {
        self.0.to_boolean(sc)
    }

    fn to_js_string(&self, sc: &mut LocalScope) -> Result<JsString, Value> {
        ValueConversion::to_js_string(&self.0, sc)
    }

    fn length_of_array_like(&self, sc: &mut LocalScope) -> Result<usize, Value> {
        self.0.length_of_array_like(sc)
    }

    fn to_object(&self, sc: &mut LocalScope) -> Result<Handle, Value> {
        self.0.to_object(sc)
    }
}
