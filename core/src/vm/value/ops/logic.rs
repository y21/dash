use std::{cell::RefCell, rc::Rc};

use crate::vm::value::{object::Object, Value, ValueKind};

impl Value {
    pub fn lossy_equal(&self, other: &Value) -> bool {
        self.strict_equal(other) // TODO: handle it separately
    }

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
            _ => false,
        }
    }

    pub fn is_truthy(&self) -> bool {
        match &self.kind {
            ValueKind::Bool(b) => *b,
            ValueKind::Number(n) => *n != 0f64,
            ValueKind::Object(o) => o.is_truthy(),
            ValueKind::Undefined | ValueKind::Null => false,
            _ => unreachable!(),
        }
    }

    pub fn is_nullish(&self) -> bool {
        match &self.kind {
            ValueKind::Null | ValueKind::Undefined => true,
            _ => false,
        }
    }

    pub fn logical_and_ref<'a>(&'a self, other: &'a Value) -> &'a Value {
        let this = self.is_truthy();
        if this {
            other
        } else {
            self
        }
    }

    pub fn logical_and(this: Rc<RefCell<Value>>, other: Rc<RefCell<Value>>) -> Rc<RefCell<Value>> {
        if this.borrow().is_truthy() {
            other
        } else {
            this
        }
    }

    pub fn logical_or_ref<'a>(&'a self, other: &'a Value) -> &'a Value {
        let this = self.is_truthy();
        if !this {
            other
        } else {
            self
        }
    }

    pub fn logical_or(this: Rc<RefCell<Value>>, other: Rc<RefCell<Value>>) -> Rc<RefCell<Value>> {
        if !this.borrow().is_truthy() {
            other
        } else {
            this
        }
    }

    pub fn nullish_coalescing_ref<'a>(&'a self, other: &'a Value) -> &'a Value {
        let this = self.is_nullish();
        if this {
            other
        } else {
            self
        }
    }

    pub fn nullish_coalescing(
        this: Rc<RefCell<Value>>,
        other: Rc<RefCell<Value>>,
    ) -> Rc<RefCell<Value>> {
        if this.borrow().is_nullish() {
            other
        } else {
            this
        }
    }

    pub fn _typeof(&self) -> &'static str {
        match &self.kind {
            ValueKind::Bool(_) => "boolean",
            ValueKind::Null => "object",
            ValueKind::Object(o) => o._typeof(),
            ValueKind::Number(_) => "number",
            ValueKind::Undefined => "undefined",
            _ => unreachable!(),
        }
    }
}

impl Object {
    pub fn _typeof(&self) -> &'static str {
        match self {
            Self::Any(_) | Self::Array(_) | Self::WeakSet(_) => "object",
            Self::Function(_) => "function",
            Self::String(_) => "string",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Self::String(s) => !s.is_empty(),
            Self::Array(_) => true,
            Self::Function(..) => true,
            Self::Any(_) => true,
            Self::WeakSet(_) => true,
        }
    }
    pub fn lossy_equal(&self, other: &Value) -> bool {
        self.strict_equal(other)
    }

    pub fn strict_equal(&self, other: &Value) -> bool {
        match self {
            Self::String(s) => {
                let other = match &other.kind {
                    ValueKind::Object(o) => match &**o {
                        Object::String(s) => s,
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
