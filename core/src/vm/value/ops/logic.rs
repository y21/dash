use crate::{
    gc::Handle,
    vm::value::{
        object::{ExoticObject, Object},
        Value, ValueKind,
    },
};

impl Value {
    /// Implements the behavior of the == operator
    pub fn lossy_equal(&self, other: &Value) -> bool {
        self.strict_equal(other) // TODO: handle it separately
    }

    /// Implements the behavior of the === operator
    #[allow(clippy::float_cmp)]
    pub fn strict_equal(&self, other: &Value) -> bool {
        match &self.kind {
            ValueKind::Number(n) => {
                let other = match &other.kind {
                    ValueKind::Number(n) => n,
                    _ => return false,
                };

                *other == *n
            }
            ValueKind::Bool(b) => {
                let other = match &other.kind {
                    ValueKind::Bool(b) => b,
                    _ => return false,
                };

                *other == *b
            }
            ValueKind::Null => matches!(other.kind, ValueKind::Null),
            ValueKind::Undefined => matches!(other.kind, ValueKind::Undefined),
            ValueKind::Object(o) => o.strict_equal(other),
        }
    }

    /// Checks whether a value is considered to be truthy
    pub fn is_truthy(&self) -> bool {
        match &self.kind {
            ValueKind::Bool(b) => *b,
            ValueKind::Number(n) => *n != 0f64 && !n.is_nan(),
            ValueKind::Object(o) => o.is_truthy(),
            ValueKind::Undefined | ValueKind::Null => false,
        }
    }

    /// Checks whether a value is considered to be nullish
    pub fn is_nullish(&self) -> bool {
        matches!(self.kind, ValueKind::Null | ValueKind::Undefined)
    }

    /// Implements the logical and operator, given references to two [Value]s
    pub fn logical_and_ref<'a>(&'a self, other: &'a Value) -> &'a Value {
        let this = self.is_truthy();
        if this {
            other
        } else {
            self
        }
    }

    /// Implements the logical and operator, given cells to two [Value]s
    pub fn logical_and(this: Handle<Value>, other: Handle<Value>) -> Handle<Value> {
        if unsafe { this.borrow_unbounded() }.is_truthy() {
            other
        } else {
            this
        }
    }

    /// Implements the logical or operator, given references to two [Value]s
    pub fn logical_or_ref<'a>(&'a self, other: &'a Value) -> &'a Value {
        let this = self.is_truthy();
        if !this {
            other
        } else {
            self
        }
    }

    /// Implements the logical or operator, given cells to two [Value]s
    pub fn logical_or(this: Handle<Value>, other: Handle<Value>) -> Handle<Value> {
        if !unsafe { this.borrow_unbounded() }.is_truthy() {
            other
        } else {
            this
        }
    }

    /// Implements the nullish coalescing operator, given references to two [Value]s
    pub fn nullish_coalescing_ref<'a>(&'a self, other: &'a Value) -> &'a Value {
        let this = self.is_nullish();
        if this {
            other
        } else {
            self
        }
    }

    /// Implements the nullish coalescing operator, given cells to two [Value]s
    pub fn nullish_coalescing(this: Handle<Value>, other: Handle<Value>) -> Handle<Value> {
        if unsafe { this.borrow_unbounded() }.is_nullish() {
            other
        } else {
            this
        }
    }

    /// Implements the behavior of the typeof operator
    pub fn _typeof(&self) -> &'static str {
        match &self.kind {
            ValueKind::Bool(_) => "boolean",
            ValueKind::Null => "object",
            ValueKind::Object(o) => o._typeof(),
            ValueKind::Number(_) => "number",
            ValueKind::Undefined => "undefined",
        }
    }
}

impl Object {
    /// Implements the behavior of the typeof operator specifically on [Object]s
    pub fn _typeof(&self) -> &'static str {
        match self {
            Self::Ordinary
            | Self::Exotic(ExoticObject::Array(_))
            | Self::Exotic(ExoticObject::Weak(_))
            | Self::Exotic(ExoticObject::Promise(_))
            | Self::Exotic(ExoticObject::Custom(_)) => "object",
            Self::Exotic(ExoticObject::Function(_)) => "function",
            Self::Exotic(ExoticObject::String(_)) => "string",
        }
    }

    /// Checks whether an object is considered to be truthy
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Exotic(ExoticObject::String(s)) => !s.is_empty(),
            Self::Exotic(ExoticObject::Array(_)) => true,
            Self::Exotic(ExoticObject::Function(..)) => true,
            Self::Exotic(ExoticObject::Weak(_)) => true,
            Self::Exotic(ExoticObject::Promise(_)) => true,
            Self::Exotic(ExoticObject::Custom(_)) => true,
            Self::Ordinary => true,
        }
    }

    /// Implements the == operator on objects
    pub fn lossy_equal(&self, other: &Value) -> bool {
        self.strict_equal(other)
    }

    /// Implements the === operator on objects
    pub fn strict_equal(&self, other: &Value) -> bool {
        match self {
            Self::Exotic(ExoticObject::String(s)) => {
                let other = match &other.kind {
                    ValueKind::Object(o) => match &**o {
                        Self::Exotic(ExoticObject::String(s)) => s,
                        _ => return false,
                    },
                    _ => return false,
                };

                s.eq(other)
            }
            _ => {
                let other = match &other.kind {
                    ValueKind::Object(o) => &**o,
                    _ => return false,
                };

                std::ptr::eq(self as *const _, other as *const _)
            }
        }
    }
}
