use std::any::TypeId;
use std::cell::{Cell, OnceCell, RefCell};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use dash_middle::compiler::constant::ConstantPool;
use dash_middle::interner::StringInterner;
use dash_regex::Regex;

use crate::value::Unrooted;
use crate::value::primitive::{Null, Number, Undefined};
use crate::value::typedarray::TypedArrayKind;

use super::{AllocFlags, Allocator, ObjectId};

pub struct TraceCtxt<'vm> {
    pub interner: &'vm mut StringInterner,
    pub alloc: &'vm mut Allocator,
}

impl<'vm> TraceCtxt<'vm> {
    pub fn new(interner: &'vm mut StringInterner, alloc: &'vm mut Allocator) -> Self {
        Self { interner, alloc }
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
unsafe impl<T: Trace, E: Trace> Trace for Result<T, E> {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        match self {
            Ok(v) => v.trace(cx),
            Err(v) => v.trace(cx),
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
unsafe impl<T: Trace> Trace for VecDeque<T> {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        self.iter().for_each(|t| t.trace(cx));
    }
}

unsafe impl<A: Trace, B: Trace> Trace for (A, B) {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        self.0.trace(cx);
        self.1.trace(cx);
    }
}

unsafe impl<T: Trace, S> Trace for HashSet<T, S> {
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

unsafe impl<T: Trace> Trace for OnceCell<T> {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        if let Some(value) = self.get() {
            value.trace(cx);
        }
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
            constants:
                ConstantPool {
                    numbers,
                    symbols,
                    booleans,
                    functions,
                    regexes,
                },
            externals: _,
            rest_local: _,
            source: Rc { .. },
            debug_symbols: _,
            references_arguments: _,
            has_extends_clause: _,
        } = self;
        name.trace(cx);
        numbers.as_slice().trace(cx);
        symbols.as_slice().trace(cx);
        booleans.as_slice().trace(cx);
        functions.as_slice().trace(cx);

        for (Regex { .. }, sym) in regexes.as_slice() {
            sym.trace(cx);
        }
    }
}

unsafe impl Trace for ObjectId {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        let (data, metadata) = cx.alloc.resolve_raw(*self);
        let info = cx.alloc.info(*self);

        unsafe {
            let flags = info.flags.get();
            if flags.contains(AllocFlags::VISITED) {
                // Already marked
                return;
            }
            info.flags.set(flags | AllocFlags::VISITED);
            ((*metadata).trace)(data, cx);
        };
    }
}

unsafe_empty_trace!(
    usize,
    u8,
    u32,
    u64,
    f64,
    bool,
    str,
    Undefined,
    Null,
    Number,
    TypedArrayKind,
    PathBuf,
    Path,
    String,
    &str,
    (),
    TypeId
);
