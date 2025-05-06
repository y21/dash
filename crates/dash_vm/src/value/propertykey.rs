use dash_middle::interner::{self, sym};

use crate::gc::trace::{Trace, TraceCtxt};
use crate::localscope::LocalScope;
use crate::value::ops::conversions::ValueConversion;
use crate::value::{Unpack, ValueKind};

use super::Value;
use super::primitive::{Number, Symbol};
use super::string::JsString;

/// A property key: either a string or a symbol.
///
/// For optimization purposes internally here we differentiate between numeric and non-numeric keys
/// and types that can have numeric indices can use this via the `index` method to support indexing without having
/// to intern strings, but otherwise the `to_js_string` method should be used to get a string out of it (and will automatically
/// deal with interning numeric keys).
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct PropertyKey(pub PropertyKeyInner);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum PropertyKeyInner {
    String(JsString),
    Symbol(Symbol),
    Index(u32),
}

unsafe impl Trace for PropertyKey {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        let Self(inner) = self;
        match inner {
            PropertyKeyInner::String(s) => s.trace(cx),
            PropertyKeyInner::Symbol(s) => s.trace(cx),
            PropertyKeyInner::Index(_) => {}
        }
    }
}

pub trait ToPropertyKey {
    fn to_key(self, sc: &mut LocalScope<'_>) -> PropertyKey;
}

impl ToPropertyKey for interner::Symbol {
    fn to_key(self, sc: &mut LocalScope<'_>) -> PropertyKey {
        PropertyKey::from_js_string(self.into(), sc)
    }
}

impl ToPropertyKey for Symbol {
    fn to_key(self, _: &mut LocalScope<'_>) -> PropertyKey {
        PropertyKey(PropertyKeyInner::Symbol(self))
    }
}

impl ToPropertyKey for JsString {
    fn to_key(self, sc: &mut LocalScope<'_>) -> PropertyKey {
        PropertyKey::from_js_string(self, sc)
    }
}

impl ToPropertyKey for usize {
    fn to_key(self, sc: &mut LocalScope<'_>) -> PropertyKey {
        if let Ok(u) = u32::try_from(self) {
            PropertyKey(PropertyKeyInner::Index(u))
        } else {
            PropertyKey(PropertyKeyInner::String(sc.intern_usize(self).into()))
        }
    }
}

impl PropertyKey {
    pub const PROTO: PropertyKey = PropertyKey(PropertyKeyInner::String(JsString::from_sym(sym::__proto__)));
    pub const CONSTRUCTOR: PropertyKey = PropertyKey(PropertyKeyInner::String(JsString::from_sym(sym::constructor)));

    pub fn to_js_string(self, sc: &mut LocalScope<'_>) -> Option<interner::Symbol> {
        match self.0 {
            PropertyKeyInner::String(string) => Some(string.sym()),
            PropertyKeyInner::Index(u) => Some(sc.intern_usize(u as usize)),
            PropertyKeyInner::Symbol(_) => None,
        }
    }

    pub fn any_js_string(self, sc: &mut LocalScope<'_>) -> interner::Symbol {
        match self.0 {
            PropertyKeyInner::String(js_string) => js_string.sym(),
            PropertyKeyInner::Index(u) => sc.intern_usize(u as usize),
            PropertyKeyInner::Symbol(symbol) => symbol.sym(),
        }
    }

    pub fn to_value(&self, sc: &mut LocalScope<'_>) -> Value {
        match self.0 {
            PropertyKeyInner::String(s) => Value::string(s),
            PropertyKeyInner::Index(u) => Value::string(sc.intern_usize(u as usize).into()),
            PropertyKeyInner::Symbol(s) => Value::symbol(s),
        }
    }

    pub fn index(&self) -> Option<u32> {
        if let PropertyKeyInner::Index(idx) = self.0 {
            Some(idx)
        } else {
            None
        }
    }

    pub fn index_usize(&self) -> Option<usize> {
        self.index().map(|i| i as usize)
    }

    pub fn from_js_string(string: JsString, sc: &mut LocalScope<'_>) -> Self {
        if let Ok(n) = string.res(sc).parse::<u32>() {
            Self(PropertyKeyInner::Index(n))
        } else {
            Self(PropertyKeyInner::String(string))
        }
    }

    pub fn from_value(sc: &mut LocalScope<'_>, value: Value) -> Result<Self, Value> {
        // TODO: call ToPrimitive as specified by ToPropertyKey in the spec?
        match value.unpack() {
            ValueKind::Symbol(s) => Ok(Self(PropertyKeyInner::Symbol(s))),
            ValueKind::Number(Number(n)) if n.trunc() == n && n >= 0.0 && n < u32::MAX as f64 => {
                Ok(Self(PropertyKeyInner::Index(n as u32)))
            }
            _ => Ok(Self::from_js_string(value.to_js_string(sc)?, sc)),
        }
    }

    pub fn inner(&self) -> PropertyKeyInner {
        self.0
    }
}
