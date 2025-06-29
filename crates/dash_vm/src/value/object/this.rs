use std::fmt::Debug;

use crate::gc::ObjectId;
use crate::gc::trace::{Trace, TraceCtxt};
use crate::localscope::LocalScope;
use crate::throw;
use crate::value::Value;

#[derive(Debug, Clone, Copy)]
pub enum ThisKind {
    /// No `this` binding. Evaluates to the global object in non-strict mode, or undefined in strict mode
    Default,
    /// Initial state of `this` in subclass constructors. Throws an error if attempted to evaluate as a value.
    /// Gets changed to `Bound` by the call to super().
    BeforeSuper { super_constructor: ObjectId },
    /// Bound as a value.
    Bound(Value),
}

#[repr(u64)]
#[derive(Copy, Clone)]
enum PackedDiscr {
    Default = 0,
    BeforeSuper = 1,
    Bound = 2,
}

/// A more ABI-optimized version of `ThisKind` that should be used whenever it is passed around
/// between function boundaries. This is a workaround for rust-lang/rust#143050
#[derive(Copy, Clone)]
pub struct This {
    discr: PackedDiscr,
    data: u64,
}

unsafe impl Trace for This {
    fn trace(&self, cx: &mut TraceCtxt) {
        match self.kind() {
            ThisKind::Default => {}
            ThisKind::BeforeSuper { super_constructor } => super_constructor.trace(cx),
            ThisKind::Bound(value) => value.trace(cx),
        }
    }
}

impl Debug for This {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind().fmt(f)
    }
}

impl This {
    pub fn kind(self) -> ThisKind {
        match self.discr {
            PackedDiscr::Default => ThisKind::Default,
            PackedDiscr::Bound => ThisKind::Bound(Value::from_raw(self.data)),
            PackedDiscr::BeforeSuper => ThisKind::BeforeSuper {
                super_constructor: ObjectId::from_raw(self.data as u32),
            },
        }
    }

    #[expect(clippy::should_implement_trait)]
    #[inline]
    pub fn default() -> Self {
        Self {
            discr: PackedDiscr::Default,
            data: 0,
        }
    }

    #[inline]
    pub fn before_super(super_constructor: ObjectId) -> Self {
        Self {
            discr: PackedDiscr::BeforeSuper,
            data: super_constructor.raw() as u64,
        }
    }

    #[inline]
    pub fn bound(value: Value) -> Self {
        Self {
            discr: PackedDiscr::Bound,
            data: value.raw(),
        }
    }

    pub fn to_value(self, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
        match self.kind() {
            // TODO: once we have strict mode, eval to undefined
            ThisKind::Default => Ok(Value::object(scope.global)),
            ThisKind::Bound(value) => Ok(value),
            ThisKind::BeforeSuper { .. } => {
                throw!(scope, Error, "`super()` must be called before accessing `this`")
            }
        }
    }
}
