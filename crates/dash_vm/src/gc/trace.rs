use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use dash_middle::compiler::constant::Function;

use crate::value::function::native::CallContext;
use crate::value::primitive::{Null, Number, Symbol, Undefined};
use crate::value::regex::RegExpInner;
use crate::value::typedarray::TypedArrayKind;
use crate::value::Unrooted;

use super::interner::StringInterner;

pub struct TraceCtxt<'vm> {
    interner: &'vm mut StringInterner,
}

impl<'vm> TraceCtxt<'vm> {
    pub fn new(interner: &'vm mut StringInterner) -> Self {
        Self { interner }
    }
}

/// # Safety
/// Implementors of this trait must provide a valid trace implementation
/// by calling any possible, reachable [`super::Handle`]s
///
/// Consider deriving this trait using the derive macro provided by the `dash_proc_macro` crate
pub unsafe trait Trace {
    fn trace(&self, cx: &mut TraceCtxt<'_>);
}

unsafe impl<T: Trace> Trace for [T] {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        for item in self {
            item.trace(cx);
        }
    }
}

unsafe impl<T: Trace> Trace for Option<T> {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        if let Some(t) = self {
            t.trace(cx);
        }
    }
}
unsafe impl Trace for Unrooted {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        unsafe { self.get().trace(cx) }
    }
}

unsafe impl<T: Trace> Trace for Vec<T> {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        self.as_slice().trace(cx);
    }
}

unsafe impl<T: Trace> Trace for HashSet<T> {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        for t in self.iter() {
            t.trace(cx);
        }
    }
}

unsafe impl<K: Trace, V: Trace, S> Trace for HashMap<K, V, S> {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        for (k, v) in self.iter() {
            k.trace(cx);
            v.trace(cx);
        }
    }
}

unsafe impl<T: Trace + ?Sized> Trace for Rc<T> {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        T::trace(self, cx)
    }
}

unsafe impl<T: Trace + Copy> Trace for Cell<T> {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        Cell::get(self).trace(cx);
    }
}

unsafe impl<T: Trace + ?Sized> Trace for Box<T> {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        T::trace(self, cx)
    }
}

unsafe impl<T: ?Sized + Trace> Trace for RefCell<T> {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        T::trace(&RefCell::borrow(self), cx);
    }
}

macro_rules! unsafe_empty_trace {
    ( $($t:ty),* ) => {
        $(
            unsafe impl Trace for $t {
                fn trace(&self, _: &mut TraceCtxt<'_>) {
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
    // Symbol,
    Number,
    RegExpInner,
    TypedArrayKind,
    PathBuf,
    Path,
    String
);
