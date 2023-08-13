use std::borrow;
use std::fmt;
use std::hash::BuildHasherDefault;
use std::hash::Hash;
use std::rc::Rc;

use hashbrown::hash_map::EntryRef;
use rustc_hash::FxHasher;

pub mod sym {
    use super::Symbol;

    pub const IF: Symbol = Symbol(0);
    pub const ELSE: Symbol = Symbol(1);
    pub const FUNCTION: Symbol = Symbol(2);
    pub const VAR: Symbol = Symbol(3);
    pub const LET: Symbol = Symbol(4);
    pub const CONST: Symbol = Symbol(5);
    pub const RETURN: Symbol = Symbol(6);
    pub const THROW: Symbol = Symbol(7);
    pub const TRY: Symbol = Symbol(8);
    pub const CATCH: Symbol = Symbol(9);
    pub const FINALLY: Symbol = Symbol(10);
    pub const TRUE_LIT: Symbol = Symbol(11);
    pub const FALSE_LIT: Symbol = Symbol(12);
    pub const NULL_LIT: Symbol = Symbol(13);
    pub const UNDEFINED_LIT: Symbol = Symbol(14);
    pub const YIELD: Symbol = Symbol(15);
    pub const NEW: Symbol = Symbol(16);
    pub const FOR: Symbol = Symbol(17);
    pub const DO: Symbol = Symbol(18);
    pub const WHILE: Symbol = Symbol(19);
    pub const IN: Symbol = Symbol(20);
    pub const INSTANCEOF: Symbol = Symbol(21);
    pub const ASYNC: Symbol = Symbol(22);
    pub const AWAIT: Symbol = Symbol(23);
    pub const DELETE: Symbol = Symbol(24);
    pub const VOID: Symbol = Symbol(25);
    pub const TYPEOF: Symbol = Symbol(26);
    pub const CONTINUE: Symbol = Symbol(27);
    pub const BREAK: Symbol = Symbol(28);
    pub const IMPORT: Symbol = Symbol(29);
    pub const EXPORT: Symbol = Symbol(30);
    pub const DEFAULT: Symbol = Symbol(31);
    pub const DEBUGGER: Symbol = Symbol(32);
    pub const OF: Symbol = Symbol(33);
    pub const CLASS: Symbol = Symbol(34);
    pub const EXTENDS: Symbol = Symbol(35);
    pub const STATIC: Symbol = Symbol(36);
    pub const SWITCH: Symbol = Symbol(37);
    pub const CASE: Symbol = Symbol(38);
    pub const GET: Symbol = Symbol(39);
    pub const SET: Symbol = Symbol(40);

    pub const PREINTERNED: &[(&str, Symbol); 84] = &[
        ("if", IF),
        ("else", ELSE),
        ("function", FUNCTION),
        ("var", VAR),
        ("let", LET),
        ("const", CONST),
        ("return", RETURN),
        ("throw", THROW),
        ("try", TRY),
        ("catch", CATCH),
        ("finally", FINALLY),
        ("true", TRUE_LIT),
        ("false", FALSE_LIT),
        ("null", NULL_LIT),
        ("undefined", UNDEFINED_LIT),
        ("yield", YIELD),
        ("new", NEW),
        ("for", FOR),
        ("do", DO),
        ("while", WHILE),
        ("in", IN),
        ("instanceof", INSTANCEOF),
        ("async", ASYNC),
        ("await", AWAIT),
        ("delete", DELETE),
        ("void", VOID),
        ("typeof", TYPEOF),
        ("continue", CONTINUE),
        ("break", BREAK),
        ("import", IMPORT),
        ("export", EXPORT),
        ("default", DEFAULT),
        ("debugger", DEBUGGER),
        ("of", OF),
        ("class", CLASS),
        ("extends", EXTENDS),
        ("static", STATIC),
        ("switch", SWITCH),
        ("case", CASE),
        ("get", GET),
        ("set", SET),
        ("$", DOLLAR),
        ("", EMPTY),
        ("constructor", CONSTRUCTOR),
        ("this", THIS),
        ("for_of_iter", FOR_OF_ITER),
        ("for_of_gen_step", FOR_OF_GEN_STEP),
        ("value", VALUE),
        ("done", DONE),
        ("next", NEXT),
        ("super", SUPER),
        ("globalThis", GLOBAL_THIS),
        ("Infinity", INFINITY),
        ("NaN", NAN),
        ("Math", MATH),
        ("exp", EXP),
        ("log2", LOG2),
        ("expm1", EXPM1),
        ("cbrt", CBRT),
        ("clz32", CLZ32),
        ("atanh", ATANH),
        ("atan2", ATAN2),
        ("round", ROUND),
        ("acosh", ACOSH),
        ("abs", ABS),
        ("sinh", SINH),
        ("sin", SIN),
        ("ceil", CEIL),
        ("tan", TAN),
        ("trunc", TRUNC),
        ("asinh", ASINH),
        ("log10", LOG10),
        ("asin", ASIN),
        ("random", RANDOM),
        ("log1p", LOG1P),
        ("sqrt", SQRT),
        ("atan", ATAN),
        ("log", LOG),
        ("floor", FLOOR),
        ("cosh", COSH),
        ("acos", ACOS),
        ("cos", COS),
        ("DesugaredClass", DESUGARED_CLASS),
        ("prototype", PROTOTYPE),
    ];

    // ⚠️⚠️⚠️⚠️ Update these constants when adding a keyword.
    // We rely on the fact that the keywords are contiguous in the symbol table,
    // making it very easy and cheap to check if a symbol is a keyword.
    // TODO: automate this with a proc macro or sorts.
    pub const KEYWORD_START: Symbol = IF;
    pub const KEYWORD_END: Symbol = SET;

    // Other non-keyword preinterned symbols
    pub const DOLLAR: Symbol = Symbol(KEYWORD_END.0 + 1);
    pub const EMPTY: Symbol = Symbol(KEYWORD_END.0 + 2);
    pub const CONSTRUCTOR: Symbol = Symbol(KEYWORD_END.0 + 3);
    pub const THIS: Symbol = Symbol(KEYWORD_END.0 + 4);
    pub const FOR_OF_ITER: Symbol = Symbol(KEYWORD_END.0 + 5);
    pub const FOR_OF_GEN_STEP: Symbol = Symbol(KEYWORD_END.0 + 6);
    pub const VALUE: Symbol = Symbol(KEYWORD_END.0 + 7);
    pub const DONE: Symbol = Symbol(KEYWORD_END.0 + 8);
    pub const NEXT: Symbol = Symbol(KEYWORD_END.0 + 9);
    pub const SUPER: Symbol = Symbol(KEYWORD_END.0 + 10);
    pub const GLOBAL_THIS: Symbol = Symbol(KEYWORD_END.0 + 11);
    pub const INFINITY: Symbol = Symbol(KEYWORD_END.0 + 12);
    pub const NAN: Symbol = Symbol(KEYWORD_END.0 + 13);
    pub const MATH: Symbol = Symbol(KEYWORD_END.0 + 14);
    pub const EXP: Symbol = Symbol(KEYWORD_END.0 + 15);
    pub const LOG2: Symbol = Symbol(KEYWORD_END.0 + 16);
    pub const EXPM1: Symbol = Symbol(KEYWORD_END.0 + 17);
    pub const CBRT: Symbol = Symbol(KEYWORD_END.0 + 18);
    pub const CLZ32: Symbol = Symbol(KEYWORD_END.0 + 19);
    pub const ATANH: Symbol = Symbol(KEYWORD_END.0 + 20);
    pub const ATAN2: Symbol = Symbol(KEYWORD_END.0 + 21);
    pub const ROUND: Symbol = Symbol(KEYWORD_END.0 + 22);
    pub const ACOSH: Symbol = Symbol(KEYWORD_END.0 + 23);
    pub const ABS: Symbol = Symbol(KEYWORD_END.0 + 24);
    pub const SINH: Symbol = Symbol(KEYWORD_END.0 + 25);
    pub const SIN: Symbol = Symbol(KEYWORD_END.0 + 26);
    pub const CEIL: Symbol = Symbol(KEYWORD_END.0 + 27);
    pub const TAN: Symbol = Symbol(KEYWORD_END.0 + 28);
    pub const TRUNC: Symbol = Symbol(KEYWORD_END.0 + 29);
    pub const ASINH: Symbol = Symbol(KEYWORD_END.0 + 30);
    pub const LOG10: Symbol = Symbol(KEYWORD_END.0 + 31);
    pub const ASIN: Symbol = Symbol(KEYWORD_END.0 + 32);
    pub const RANDOM: Symbol = Symbol(KEYWORD_END.0 + 33);
    pub const LOG1P: Symbol = Symbol(KEYWORD_END.0 + 34);
    pub const SQRT: Symbol = Symbol(KEYWORD_END.0 + 35);
    pub const ATAN: Symbol = Symbol(KEYWORD_END.0 + 36);
    pub const LOG: Symbol = Symbol(KEYWORD_END.0 + 37);
    pub const FLOOR: Symbol = Symbol(KEYWORD_END.0 + 38);
    pub const COSH: Symbol = Symbol(KEYWORD_END.0 + 39);
    pub const ACOS: Symbol = Symbol(KEYWORD_END.0 + 40);
    pub const COS: Symbol = Symbol(KEYWORD_END.0 + 41);
    pub const DESUGARED_CLASS: Symbol = Symbol(KEYWORD_END.0 + 42);
    pub const PROTOTYPE: Symbol = Symbol(KEYWORD_END.0 + 43);
}

#[derive(Default, Debug)]
pub struct StringInterner {
    store: Vec<Rc<str>>,
    mapping: hashbrown::HashMap<Rc<str>, RawSymbol, BuildHasherDefault<FxHasher>>,
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
            store.push(s);
        }

        Self { store, mapping }
    }

    pub fn resolve(&self, symbol: Symbol) -> &Rc<str> {
        &self.store[symbol.0 as usize]
    }

    pub fn intern(&mut self, value: impl borrow::Borrow<str>) -> Symbol {
        match self.mapping.entry_ref(value.borrow()) {
            EntryRef::Occupied(entry) => Symbol(*entry.get()),
            EntryRef::Vacant(entry) => {
                let id = self.store.len() as RawSymbol;
                let value: Rc<str> = Rc::from(value.borrow());
                self.store.push(value.clone());
                entry.insert(id);
                Symbol(id)
            }
        }
    }
}

type RawSymbol = u32;

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
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
    pub fn is_keyword(self) -> bool {
        #![allow(clippy::absurd_extreme_comparisons)]

        self.0 >= sym::KEYWORD_START.0 && self.0 <= sym::KEYWORD_END.0
    }
}
