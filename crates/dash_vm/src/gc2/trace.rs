use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use dash_middle::compiler::constant::Function;

use crate::value::primitive::Null;
use crate::value::primitive::Number;
use crate::value::primitive::Symbol;
use crate::value::primitive::Undefined;
use crate::value::regex::RegExpInner;
use crate::value::typedarray::TypedArrayKind;

/// # Safety
/// Implementors of this trait must provide a valid trace implementation
/// by calling any possible, reachable [`super::Handle`]s
pub unsafe trait Trace {
    fn trace(&self);
}

unsafe impl<T: Trace> Trace for [T] {
    fn trace(&self) {
        for item in self {
            item.trace();
        }
    }
}

unsafe impl<T: Trace> Trace for Option<T> {
    fn trace(&self) {
        if let Some(t) = self {
            t.trace();
        }
    }
}

unsafe impl<T: Trace> Trace for Vec<T> {
    fn trace(&self) {
        self.as_slice().trace();
    }
}

unsafe impl<T: Trace> Trace for HashSet<T> {
    fn trace(&self) {
        for t in self.iter() {
            t.trace();
        }
    }
}

unsafe impl<T: Trace + ?Sized> Trace for Rc<T> {
    fn trace(&self) {
        T::trace(self)
    }
}

unsafe impl<T: Trace + Copy> Trace for Cell<T> {
    fn trace(&self) {
        Cell::get(self).trace();
    }
}

unsafe impl<T: Trace + ?Sized> Trace for Box<T> {
    fn trace(&self) {
        T::trace(self)
    }
}

unsafe impl<T: ?Sized + Trace> Trace for RefCell<T> {
    fn trace(&self) {
        T::trace(&RefCell::borrow(self));
    }
}

macro_rules! unsafe_empty_trace {
    ( $($t:ty),* ) => {
        $(
            unsafe impl Trace for $t {
                fn trace(&self) {
                }
            }
        )*
    };
}

unsafe_empty_trace!(
    Function,
    usize,
    u8,
    f64,
    bool,
    str,
    Undefined,
    Null,
    Symbol,
    Number,
    RegExpInner,
    TypedArrayKind
);
