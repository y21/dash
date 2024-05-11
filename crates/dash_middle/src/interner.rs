use std::cell::Cell;
use std::hash::{BuildHasherDefault, Hash, Hasher};
use std::rc::Rc;
use std::{borrow, fmt};

use hashbrown::hash_map::RawEntryMut;
use rustc_hash::FxHasher;

#[cfg(feature = "format")]
use serde::{Deserialize, Serialize};

pub mod sym {
    #![allow(non_upper_case_globals)]

    use super::Symbol;

    dash_proc_macro::define_symbols! {
        [
            // Keywords, defined separately so that they can be contiguous in the symbol table
            // and we can easily check if a symbol is a keyword with a simple range check.
            Keywords {
                if_: "if",
                else_: "else",
                function,
                var,
                let_: "let",
                const_: "const",
                return_: "return",
                throw,
                try_: "try",
                catch,
                finally,
                true_: "true",
                false_: "false",
                null,
                undefined,
                yield_: "yield",
                new,
                for_: "for",
                do_: "do",
                while_: "while",
                in_: "in",
                instanceof,
                async_: "async",
                await_: "await",
                delete,
                void,
                typeof_: "typeof",
                continue_: "continue",
                break_: "break",
                import,
                export,
                default,
                debugger,
                of,
                class,
                extends,
                static_: "static",
                switch,
                case,
                get,
                set
            },
            // Other preinterned symbols that can be referred to statically
            Symbols {
                dollar: "$",
                empty: "",
                constructor,
                this,
                for_of_iter,
                for_of_gen_step,
                switch_cond_desugar,
                value,
                done,
                next,
                super_: "super",
                globalThis,
                Infinity,
                NegInfinity: "-Infinity",
                NaN,
                Math,
                exp,
                log2,
                expm1,
                cbrt,
                clz32,
                atanh,
                atan2,
                round,
                acosh,
                abs,
                sinh,
                sin,
                ceil,
                tan,
                trunc,
                asinh,
                log10,
                asin,
                random,
                log1p,
                sqrt,
                atan,
                log,
                floor,
                cosh,
                acos,
                cos,
                DesugaredClass,
                prototype,
                name,
                length,
                message,
                stack,
                Error,
                toString,
                valueOf,
                object,
                boolean,
                number,
                bigint,
                string,
                symbol,
                comma: ",",
                writable,
                enumerable,
                configurable,
                __proto__,
                EvalError,
                RangeError,
                ReferenceError,
                SyntaxError,
                TypeError,
                URIError,
                AggregateError,
                Function,
                bind,
                call,
                create,
                keys,
                getOwnPropertyNames,
                getOwnPropertyDescriptor,
                getOwnPropertyDescriptors,
                defineProperty,
                defineProperties,
                entries,
                assign,
                Object,
                hasOwnProperty,
                tanh,
                max,
                min,
                pow,
                PI,
                isFinite,
                isNaN,
                eval,
                isSafeInteger,
                EPSILON,
                MAX_SAFE_INTEGER,
                MAX_VALUE,
                MIN_SAFE_INTEGER,
                MIN_VALUE,
                NEGATIVE_INFINITY,
                POSITIVE_INFINITY,
                Number,
                toFixed,
                Boolean,
                fromCharCode,
                String,
                charAt,
                charCodeAt,
                concat,
                endsWith,
                startsWith,
                includes,
                indexOf,
                lastIndexOf,
                padEnd,
                padStart,
                repeat,
                replace,
                replaceAll,
                split,
                toLowerCase,
                toUpperCase,
                big,
                blink,
                bold,
                fixed,
                italics,
                strike,
                sub,
                sup,
                fontcolor,
                fontsize,
                link,
                trim,
                trimStart,
                trimEnd,
                substr,
                substring,
                from,
                isArray,
                Array,
                join,
                values,
                at,
                every,
                some,
                fill,
                filter,
                reduce,
                find,
                findIndex,
                flat,
                forEach,
                map,
                pop,
                push,
                reverse,
                shift,
                sort,
                unshift,
                slice,
                asyncIterator,
                hasInstance,
                iterator,
                match_: "match",
                matchAll,
                search,
                species,
                toPrimitive,
                toStringTag,
                unscopables,
                JsSymbol: "Symbol",
                ArrayBuffer,
                byteLength,
                Uint8Array,
                Int8Array,
                Uint16Array,
                Int16Array,
                Uint32Array,
                Int32Array,
                Float32Array,
                Float64Array,
                resolve,
                reject,
                Promise,
                then,
                Set,
                add,
                has,
                clear,
                size,
                Map,
                RegExp,
                test,
                exec,
                now,
                Date,
                parse,
                parseFloat,
                parseInt,
                console,
                JSON,
                isConcatSpreadable,
                zero: "0",
                one: "1",
                getPrototypeOf,
                setPrototypeOf,
                isPrototypeOf,
                arguments,
                propertyIsEnumerable,
                apply
            }
        ]
    }
}

#[derive(Clone, Debug)]
struct StringData {
    visited: Cell<bool>,
    value: Rc<str>,
}

#[derive(Default, Clone, Debug)]
pub struct StringInterner {
    store: Vec<Option<StringData>>,
    mapping: hashbrown::HashMap<Rc<str>, RawSymbol, BuildHasherDefault<FxHasher>>,
    /// List of free indices in the storage
    free: Vec<RawSymbol>,
}

fn fxhash(s: &str) -> u64 {
    let mut hasher = FxHasher::default();
    s.hash(&mut hasher);
    hasher.finish()
}

impl StringInterner {
    pub fn new() -> Self {
        let mut store = Vec::with_capacity(sym::PREINTERNED.len());
        let mut mapping =
            hashbrown::HashMap::with_capacity_and_hasher(sym::PREINTERNED.len(), BuildHasherDefault::default());

        for (s, index) in sym::PREINTERNED {
            let s: Rc<str> = Rc::from(*s);
            debug_assert!(store.len() == index.0 as usize);
            mapping.insert(s.clone(), index.0);
            store.push(Some(StringData {
                visited: Cell::new(false),
                value: s,
            }));
        }

        Self {
            store,
            mapping,
            free: Vec::new(),
        }
    }

    pub fn resolve(&self, symbol: Symbol) -> &str {
        self.store[symbol.0 as usize].as_ref().unwrap().value.as_ref()
    }

    // TODO: perf improvement idea: use interior mutability and allow calling with just a `&self`
    // would save a bunch of useless clones
    pub fn intern(&mut self, value: impl borrow::Borrow<str>) -> Symbol {
        let value = value.borrow();
        let hash = fxhash(value);

        match self.mapping.raw_entry_mut().from_hash(hash, |k| &**k == value) {
            RawEntryMut::Occupied(entry) => Symbol(*entry.get()),
            RawEntryMut::Vacant(entry) => {
                if let Some(id) = self.free.pop() {
                    let value: Rc<str> = Rc::from(value);
                    self.store[id as usize] = Some(StringData {
                        value: Rc::clone(&value),
                        visited: Cell::new(false),
                    });
                    entry.insert_hashed_nocheck(hash, value, id);
                    Symbol(id)
                } else {
                    let id = self.store.len() as RawSymbol;
                    let value: Rc<str> = Rc::from(value);
                    self.store.push(Some(StringData {
                        value: Rc::clone(&value),
                        visited: Cell::new(false),
                    }));
                    entry.insert_hashed_nocheck(hash, value, id);
                    Symbol(id)
                }
            }
        }
    }

    pub fn intern_usize(&mut self, mut val: usize) -> Symbol {
        // TODO: for small N, have a static array of numbers
        const _: () = assert!(std::mem::size_of::<usize>() <= 8);
        // `usize::MAX` is at most 20 digits long
        let mut buf = [0; 20];

        let mut from_index = 19;
        loop {
            let digit = val % 10;
            val /= 10;
            buf[from_index] = (digit as u8) + b'0';
            if val == 0 || from_index == 0 {
                break;
            } else {
                from_index -= 1;
            }
        }

        debug_assert!(std::str::from_utf8(&buf).is_ok());

        // SAFETY: `buf` always contains values in the range `b'0'..=b'9'`, which are valid UTF-8
        // this could use `std::ascii::Char` once stable
        self.intern(unsafe { std::str::from_utf8_unchecked(&buf[from_index..]) })
    }

    pub fn intern_isize(&mut self, val: isize) -> Symbol {
        // for now this just calls `intern`, but we might want to specialize this
        let string = val.to_string();
        self.intern(string.as_ref())
    }

    pub fn intern_char(&mut self, val: char) -> Symbol {
        let mut buf = [0; 4];
        self.intern(val.encode_utf8(&mut buf))
    }

    pub fn mark(&self, sym: Symbol) {
        self.store[sym.0 as usize].as_ref().unwrap().visited.set(true);
    }

    /// You must mark all reachable symbols before calling this.
    /// It won't cause undefined behavior if you don't (hence not unsafe), but it can lead to oddities such as panics.
    pub fn sweep(&mut self) {
        // Preinterned symbols are always kept, since they can be referred to statically.
        for i in sym::PREINTERNED.len()..self.store.len() {
            if let Some(data) = self.store[i].as_ref() {
                if !data.visited.get() {
                    self.mapping.remove(&data.value);
                    self.store[i] = None;
                    self.free.push(i as RawSymbol);
                } else {
                    data.visited.set(false);
                }
            }
        }
    }
}

type RawSymbol = u32;

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "format", derive(Serialize, Deserialize))]
pub struct Symbol(RawSymbol);

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_keyword() {
            write!(f, "{}", sym::PREINTERNED[self.0 as usize].0)
        } else {
            write!(f, "<interned id: {}>", self.0)
        }
    }
}

impl Symbol {
    /// This should only be used if you *really* need to. Prefer `Symbol`s directly wherever possible.
    pub fn raw(self) -> u32 {
        self.0
    }

    pub fn is_keyword(self) -> bool {
        #![allow(clippy::absurd_extreme_comparisons)]

        self.0 >= sym::KEYWORD_START && self.0 <= sym::KEYWORD_END
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! expand {
    ($interner:ident $sym:ident : $val:expr) => {
        let $sym = $interner.intern($val);
    };
    ($interner:ident $sym:ident) => {
        let $sym = $interner.intern(stringify!($sym));
    };
}

#[macro_export]
macro_rules! define_symbol_set {
    (
        $(#[$($meta:meta)*])?
        $name:ident => [$($sym:ident$(: $val:expr)?),*]
    ) => {
        mod inner {
            #![allow(non_snake_case)]
            use super::*;

            use $crate::interner::Symbol;
            use $crate::interner::StringInterner;

            $(#[$($meta)*])?
            pub struct $name {
                $(pub $sym: Symbol),*
            }
            impl $name {
                pub fn new(interner: &mut StringInterner) -> Self {
                    $($crate::expand!(interner $sym $(: $val)?);)*

                    Self {
                        $($sym),*
                    }
                }
            }
        }
        pub use inner::*;
    };
}

#[cfg(test)]
mod tests {
    use super::StringInterner;

    #[test]
    fn interning() {
        let interner = &mut StringInterner::new();
        let k1 = interner.intern_usize(usize::MAX);
        assert_eq!(interner.resolve(k1), usize::MAX.to_string());

        let k2 = interner.intern_usize(usize::MIN);
        assert_eq!(interner.resolve(k2), usize::MIN.to_string());

        let k2 = interner.intern_usize(10);
        assert_eq!(interner.resolve(k2), "10");

        let k3 = interner.intern_usize(192837465123);
        assert_eq!(interner.resolve(k3), 192837465123usize.to_string());

        let k4 = interner.intern_char('ä');
        assert_eq!(interner.resolve(k4), "ä");
    }
}
