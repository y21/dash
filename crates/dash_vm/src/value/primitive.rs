use std::hash::{Hash, Hasher};
use std::{fmt, iter};

use dash_middle::interner::{self, sym};
use dash_proc_macro::Trace;

use crate::frame::This;
use crate::gc::ObjectId;
use crate::localscope::LocalScope;
use crate::util::intern_f64;
use crate::{Vm, extract, throw};

use super::boxed::{Boolean as BoxedBoolean, Number as BoxedNumber, Symbol as BoxedSymbol};
use super::function::args::CallArgs;
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
        Ok(sc.statics.number_prototype.into())
    }

    fn apply(
        &self,
        scope: &mut LocalScope,
        _callee: ObjectId,
        _this: This,
        _args: CallArgs,
    ) -> Result<Unrooted, Unrooted> {
        throw!(scope, TypeError, "number is not a function")
    }

    fn own_keys(&self, _: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        Ok(Vec::new())
    }

    fn type_of(&self, _: &Vm) -> Typeof {
        Typeof::Number
    }

    fn internal_slots(&self, _: &Vm) -> Option<&dyn InternalSlots> {
        Some(self)
    }

    extract!(self);
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
        Ok(sc.statics.boolean_prototype.into())
    }

    fn apply(
        &self,
        scope: &mut LocalScope,
        _callee: ObjectId,
        _this: This,
        _args: CallArgs,
    ) -> Result<Unrooted, Unrooted> {
        throw!(scope, TypeError, "boolean is not a function")
    }

    fn own_keys(&self, _: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        Ok(Vec::new())
    }

    fn type_of(&self, _: &Vm) -> Typeof {
        Typeof::Boolean
    }

    fn internal_slots(&self, _: &Vm) -> Option<&dyn InternalSlots> {
        Some(self)
    }

    extract!(self);
}

pub fn array_like_keys<'a, 'b>(sc: &'a mut LocalScope<'b>, len: usize) -> impl Iterator<Item = Value> + use<'a, 'b> {
    (0..len)
        .map(|i| sc.intern_usize(i))
        .chain(iter::once_with(|| sym::length))
        .map(|x| Value::string(x.into()))
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
        let key = match key {
            PropertyKey::String(s) => s.res(sc).to_owned(),
            PropertyKey::Symbol(s) => sc.interner.resolve(s.sym()).to_owned(),
        };
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
        _callee: ObjectId,
        _this: This,
        _args: CallArgs,
    ) -> Result<Unrooted, Unrooted> {
        throw!(sc, TypeError, "undefined is not a function")
    }

    fn own_keys(&self, _: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        Ok(Vec::new())
    }

    fn type_of(&self, _: &Vm) -> Typeof {
        Typeof::Undefined
    }

    extract!(self);
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
        let key = match key {
            PropertyKey::String(s) => s.res(sc).to_owned(),
            PropertyKey::Symbol(s) => sc.interner.resolve(s.sym()).to_owned(),
        };
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
        _callee: ObjectId,
        _this: This,
        _args: CallArgs,
    ) -> Result<Unrooted, Unrooted> {
        throw!(sc, TypeError, "null is not a function")
    }

    fn own_keys(&self, _: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        Ok(Vec::new())
    }

    extract!(self);
}

// TODO: rename to JsSymbol
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Trace)]
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
        Ok(sc.statics.symbol_prototype.into())
    }

    fn apply(
        &self,
        scope: &mut LocalScope,
        _callee: ObjectId,
        _this: This,
        _args: CallArgs,
    ) -> Result<Unrooted, Unrooted> {
        throw!(scope, TypeError, "symbol is not a function")
    }

    fn own_keys(&self, _: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        Ok(Vec::new())
    }

    fn type_of(&self, _: &Vm) -> Typeof {
        Typeof::Symbol
    }

    fn internal_slots(&self, _: &Vm) -> Option<&dyn InternalSlots> {
        Some(self)
    }

    extract!(self);
}

impl InternalSlots for Symbol {}

// TODO: can this be removed in favor of the `extract` system?
pub trait InternalSlots {
    // TODO: rename as_number to number_value?
    fn string_value(&self, _: &Vm) -> Option<JsString> {
        None
    }
    fn number_value(&self, _: &Vm) -> Option<f64> {
        None
    }
    fn boolean_value(&self, _: &Vm) -> Option<bool> {
        None
    }
}

// TODO: do we even need this given that we have it for the Number wrapper? same for Rc<str> str etc
impl InternalSlots for f64 {
    fn number_value(&self, _: &Vm) -> Option<f64> {
        Some(*self)
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

    fn to_object(&self, sc: &mut LocalScope) -> Result<ObjectId, Value> {
        let num = BoxedNumber::new(sc, *self);
        Ok(sc.register(num))
    }
}

impl InternalSlots for bool {
    fn boolean_value(&self, _: &Vm) -> Option<bool> {
        Some(*self)
    }
}

impl ValueConversion for bool {
    fn to_primitive(&self, _sc: &mut LocalScope, _preferred_type: Option<PreferredType>) -> Result<Value, Value> {
        Ok(Value::boolean(*self))
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

    fn to_object(&self, sc: &mut LocalScope) -> Result<ObjectId, Value> {
        let bool = BoxedBoolean::new(sc, *self);
        Ok(sc.register(bool))
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

    fn to_object(&self, sc: &mut LocalScope) -> Result<ObjectId, Value> {
        throw!(sc, TypeError, "Cannot convert undefined to object")
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

    fn to_object(&self, sc: &mut LocalScope) -> Result<ObjectId, Value> {
        throw!(sc, TypeError, "Cannot convert null to object");
    }
}

impl ValueConversion for Symbol {
    fn to_primitive(&self, _sc: &mut LocalScope, _preferred_type: Option<PreferredType>) -> Result<Value, Value> {
        Ok(Value::symbol(*self))
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

    fn to_object(&self, sc: &mut LocalScope) -> Result<ObjectId, Value> {
        let sym = BoxedSymbol::new(sc, *self);
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
        callee: ObjectId,
        this: This,
        args: CallArgs,
    ) -> Result<Unrooted, Unrooted> {
        self.0.apply(scope, callee, this, args)
    }

    fn own_keys(&self, sc: &mut LocalScope<'_>) -> Result<Vec<Value>, Value> {
        self.0.own_keys(sc)
    }

    fn type_of(&self, vm: &Vm) -> Typeof {
        self.0.type_of(vm)
    }

    fn internal_slots(&self, _: &Vm) -> Option<&dyn InternalSlots> {
        Some(self)
    }

    extract!(self);
}

impl InternalSlots for Number {
    fn number_value(&self, _: &Vm) -> Option<f64> {
        Some(self.0)
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

    fn to_object(&self, sc: &mut LocalScope) -> Result<ObjectId, Value> {
        self.0.to_object(sc)
    }
}
