use crate::gc::trace::{Trace, TraceCtxt};
use crate::localscope::LocalScope;

use super::ops::conversions::ValueConversion;
use super::primitive::Symbol;
use super::string::JsString;
use super::{Unpack, Value, ValueKind};

// TODO: optimization opportunity: some kind of Number variant for faster indexing without .to_string()
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum PropertyKey {
    String(JsString),
    Symbol(Symbol),
}

unsafe impl Trace for PropertyKey {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        match self {
            PropertyKey::String(s) => s.trace(cx),
            PropertyKey::Symbol(s) => s.trace(cx),
        }
    }
}

impl From<JsString> for PropertyKey {
    fn from(value: JsString) -> Self {
        PropertyKey::String(value)
    }
}
impl From<dash_middle::interner::Symbol> for PropertyKey {
    fn from(value: dash_middle::interner::Symbol) -> Self {
        PropertyKey::String(value.into())
    }
}

impl From<Symbol> for PropertyKey {
    fn from(s: Symbol) -> Self {
        PropertyKey::Symbol(s)
    }
}

impl PropertyKey {
    pub fn as_value(&self) -> Value {
        match *self {
            PropertyKey::String(s) => Value::string(s),
            PropertyKey::Symbol(s) => Value::symbol(s),
        }
    }

    pub fn from_value(sc: &mut LocalScope<'_>, value: Value) -> Result<Self, Value> {
        // TODO: call ToPrimitive as specified by ToPropertyKey in the spec?
        match value.unpack() {
            ValueKind::Symbol(s) => Ok(Self::Symbol(s)),
            _ => Ok(PropertyKey::String(value.to_js_string(sc)?)),
        }
    }

    pub fn as_string(&self) -> Option<JsString> {
        match self {
            PropertyKey::String(s) => Some(*s),
            _ => None,
        }
    }
}
