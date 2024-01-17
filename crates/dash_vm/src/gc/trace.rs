use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use dash_middle::compiler::constant::Constant;

use crate::value::primitive::{Null, Number, Undefined};
use crate::value::typedarray::TypedArrayKind;
use crate::value::Unrooted;

use super::interner::StringInterner;

pub struct TraceCtxt<'vm> {
    pub interner: &'vm mut StringInterner,
}

impl<'vm> TraceCtxt<'vm> {
    pub fn new(interner: &'vm mut StringInterner) -> Self {
        Self { interner }
    }

    pub fn mark_symbol(&self, symbol: dash_middle::interner::Symbol) {
        self.interner.mark(symbol);
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

unsafe impl<K: Trace, V: Trace, S> Trace for hashbrown::HashMap<K, V, S> {
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

unsafe impl Trace for dash_middle::interner::Symbol {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        cx.mark_symbol(*self);
    }
}

unsafe impl Trace for dash_middle::compiler::constant::Function {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        let Self {
            name,
            buffer: _,
            ty: _,
            locals: _,
            params: _,
            constants,
            externals: _,
            r#async: _,
            rest_local: _,
            poison_ips: _,
            source: _,
            debug_symbols: _,
            references_arguments: _,
        } = self;
        name.trace(cx);
        constants.trace(cx);
    }
}

unsafe impl Trace for Constant {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        match self {
            Constant::Number(_) => {}
            Constant::String(sym) => sym.trace(cx),
            Constant::Identifier(sym) => sym.trace(cx),
            Constant::Boolean(_) => {}
            Constant::Function(func) => func.trace(cx),
            Constant::Regex(s) => {
                let (_, _, sym) = &**s;
                sym.trace(cx);
            }
            Constant::Null => {}
            Constant::Undefined => {}
        }
    }
}

unsafe_empty_trace!(
    usize,
    u8,
    f64,
    bool,
    str,
    Undefined,
    Null,
    // Symbol,
    Number,
    TypedArrayKind,
    PathBuf,
    Path,
    String
);
