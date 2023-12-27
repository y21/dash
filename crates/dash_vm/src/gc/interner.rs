pub use dash_middle::interner::{sym, StringInterner, Symbol};

// use std::borrow::Borrow;
// use std::cell::Cell;
// use std::hash::{BuildHasherDefault, Hash, Hasher};
// use std::rc::Rc;

// use hashbrown::hash_map::EntryRef;
// use rustc_hash::{FxHashMap, FxHasher};

// use super::trace::{Trace, TraceCtxt};

// pub mod sym {
//     use super::Symbol;

//     // pub const VM_PREINTERNED_START: Symbol = Symbol(dash_middle::interner::sym::PRE_VM_INTERNED.raw() + 1);
//     pub const VM_PREINTERNED_START: Symbol = Symbol(0);
//     pub const VALUE: Symbol = VM_PREINTERNED_START;
//     pub const NAME: Symbol = Symbol(VALUE.0 + 1);
//     pub const PROTOTYPE: Symbol = Symbol(VALUE.0 + 2);
//     pub const LENGTH: Symbol = Symbol(VALUE.0 + 3);
//     pub const MESSAGE: Symbol = Symbol(VALUE.0 + 4);
//     pub const STACK: Symbol = Symbol(VALUE.0 + 5);
//     pub const ERROR: Symbol = Symbol(VALUE.0 + 6);
//     pub const EMPTY: Symbol = Symbol(VALUE.0 + 7);
//     pub const TO_STRING: Symbol = Symbol(VALUE.0 + 8);
//     pub const VALUE_OF: Symbol = Symbol(VALUE.0 + 9);
//     pub const UNDEFINED: Symbol = Symbol(VALUE.0 + 10);
//     pub const LO_OBJECT: Symbol = Symbol(VALUE.0 + 11);
//     pub const LO_BOOLEAN: Symbol = Symbol(VALUE.0 + 12);
//     pub const LO_NUMBER: Symbol = Symbol(VALUE.0 + 13);
//     pub const LO_BIGINT: Symbol = Symbol(VALUE.0 + 14);
//     pub const LO_STRING: Symbol = Symbol(VALUE.0 + 15);
//     pub const LO_SYMBOL: Symbol = Symbol(VALUE.0 + 16);
//     pub const LO_FUNCTION: Symbol = Symbol(VALUE.0 + 17);
//     pub const GET: Symbol = Symbol(VALUE.0 + 18);
//     pub const LO_SET: Symbol = Symbol(VALUE.0 + 19);
//     pub const WRITABLE: Symbol = Symbol(VALUE.0 + 20);
//     pub const ENUMERABLE: Symbol = Symbol(VALUE.0 + 21);
//     pub const CONFIGURABLE: Symbol = Symbol(VALUE.0 + 22);
//     pub const PROTO: Symbol = Symbol(VALUE.0 + 23);
//     pub const CONSTRUCTOR: Symbol = Symbol(VALUE.0 + 24);
//     pub const DEFAULT: Symbol = Symbol(VALUE.0 + 25);
//     pub const TRUE: Symbol = Symbol(VALUE.0 + 26);
//     pub const FALSE: Symbol = Symbol(VALUE.0 + 27);
//     pub const NULL: Symbol = Symbol(VALUE.0 + 28);
//     pub const EVAL_ERROR: Symbol = Symbol(VALUE.0 + 29);
//     pub const RANGE_ERROR: Symbol = Symbol(VALUE.0 + 30);
//     pub const REFERENCE_ERROR: Symbol = Symbol(VALUE.0 + 31);
//     pub const SYNTAX_ERROR: Symbol = Symbol(VALUE.0 + 32);
//     pub const TYPE_ERROR: Symbol = Symbol(VALUE.0 + 33);
//     pub const URI_ERROR: Symbol = Symbol(VALUE.0 + 34);
//     pub const AGGREGATE_ERROR: Symbol = Symbol(VALUE.0 + 35);

//     pub const FUNCTION: Symbol = Symbol(VALUE.0 + 35);
//     pub const BIND: Symbol = Symbol(VALUE.0 + 36);
//     pub const CALL: Symbol = Symbol(VALUE.0 + 37);
//     pub const CREATE: Symbol = Symbol(VALUE.0 + 38);
//     pub const KEYS: Symbol = Symbol(VALUE.0 + 39);
//     pub const GET_OWN_PROPERTY_DESCRIPTOR: Symbol = Symbol(VALUE.0 + 40);
//     pub const GET_OWN_PROPERTY_DESCRIPTORS: Symbol = Symbol(VALUE.0 + 41);
//     pub const DEFINE_PROPERTY: Symbol = Symbol(VALUE.0 + 42);
//     pub const ENTRIES: Symbol = Symbol(VALUE.0 + 43);
//     pub const ASSIGN: Symbol = Symbol(VALUE.0 + 44);
//     pub const OBJECT: Symbol = Symbol(VALUE.0 + 45);
//     pub const HAS_OWN_PROPERTY: Symbol = Symbol(VALUE.0 + 46);
//     pub const LOG: Symbol = Symbol(VALUE.0 + 47);
//     pub const FLOOR: Symbol = Symbol(VALUE.0 + 48);
//     pub const ABS: Symbol = Symbol(VALUE.0 + 49);
//     pub const ACOS: Symbol = Symbol(VALUE.0 + 50);
//     pub const ACOSH: Symbol = Symbol(VALUE.0 + 51);
//     pub const ASIN: Symbol = Symbol(VALUE.0 + 52);
//     pub const ASINH: Symbol = Symbol(VALUE.0 + 53);
//     pub const ATAN: Symbol = Symbol(VALUE.0 + 54);
//     pub const ATANH: Symbol = Symbol(VALUE.0 + 55);
//     pub const ATAN2: Symbol = Symbol(VALUE.0 + 56);
//     pub const CBRT: Symbol = Symbol(VALUE.0 + 57);
//     pub const CEIL: Symbol = Symbol(VALUE.0 + 58);
//     pub const CLZ32: Symbol = Symbol(VALUE.0 + 59);
//     pub const COS: Symbol = Symbol(VALUE.0 + 60);
//     pub const COSH: Symbol = Symbol(VALUE.0 + 61);
//     pub const EXP: Symbol = Symbol(VALUE.0 + 62);
//     pub const EXPM1: Symbol = Symbol(VALUE.0 + 63);
//     pub const LOG1P: Symbol = Symbol(VALUE.0 + 64);
//     pub const LOG10: Symbol = Symbol(VALUE.0 + 65);
//     pub const LOG2: Symbol = Symbol(VALUE.0 + 66);
//     pub const ROUND: Symbol = Symbol(VALUE.0 + 67);
//     pub const SIN: Symbol = Symbol(VALUE.0 + 68);
//     pub const SINH: Symbol = Symbol(VALUE.0 + 69);
//     pub const SQRT: Symbol = Symbol(VALUE.0 + 70);
//     pub const TAN: Symbol = Symbol(VALUE.0 + 71);
//     pub const TANH: Symbol = Symbol(VALUE.0 + 72);
//     pub const TRUNC: Symbol = Symbol(VALUE.0 + 73);
//     pub const RANDOM: Symbol = Symbol(VALUE.0 + 74);
//     pub const MAX: Symbol = Symbol(VALUE.0 + 75);
//     pub const MIN: Symbol = Symbol(VALUE.0 + 76);
//     pub const PI: Symbol = Symbol(VALUE.0 + 77);
//     pub const IS_FINITE: Symbol = Symbol(VALUE.0 + 78);
//     pub const IS_NA_N: Symbol = Symbol(VALUE.0 + 79);
//     pub const IS_SAFE_INTEGER: Symbol = Symbol(VALUE.0 + 80);
//     pub const NUMBER: Symbol = Symbol(VALUE.0 + 81);
//     pub const TO_FIXED: Symbol = Symbol(VALUE.0 + 82);
//     pub const BOOLEAN: Symbol = Symbol(VALUE.0 + 83);
//     pub const FROM_CHAR_CODE: Symbol = Symbol(VALUE.0 + 84);
//     pub const STRING: Symbol = Symbol(VALUE.0 + 85);
//     pub const CHAR_AT: Symbol = Symbol(VALUE.0 + 86);
//     pub const CHAR_CODE_AT: Symbol = Symbol(VALUE.0 + 87);
//     pub const CONCAT: Symbol = Symbol(VALUE.0 + 88);
//     pub const ENDS_WITH: Symbol = Symbol(VALUE.0 + 89);
//     pub const STARTS_WITH: Symbol = Symbol(VALUE.0 + 90);
//     pub const INCLUDES: Symbol = Symbol(VALUE.0 + 91);
//     pub const INDEX_OF: Symbol = Symbol(VALUE.0 + 92);
//     pub const LAST_INDEX_OF: Symbol = Symbol(VALUE.0 + 93);
//     pub const PAD_END: Symbol = Symbol(VALUE.0 + 94);
//     pub const PAD_START: Symbol = Symbol(VALUE.0 + 95);
//     pub const REPEAT: Symbol = Symbol(VALUE.0 + 96);
//     pub const REPLACE: Symbol = Symbol(VALUE.0 + 97);
//     pub const REPLACE_ALL: Symbol = Symbol(VALUE.0 + 98);
//     pub const SPLIT: Symbol = Symbol(VALUE.0 + 99);
//     pub const TO_LOWER_CASE: Symbol = Symbol(VALUE.0 + 100);
//     pub const TO_UPPER_CASE: Symbol = Symbol(VALUE.0 + 101);
//     pub const BIG: Symbol = Symbol(VALUE.0 + 102);
//     pub const BLINK: Symbol = Symbol(VALUE.0 + 103);
//     pub const BOLD: Symbol = Symbol(VALUE.0 + 104);
//     pub const FIXED: Symbol = Symbol(VALUE.0 + 105);
//     pub const ITALICS: Symbol = Symbol(VALUE.0 + 106);
//     pub const STRIKE: Symbol = Symbol(VALUE.0 + 107);
//     pub const SUB: Symbol = Symbol(VALUE.0 + 108);
//     pub const SUP: Symbol = Symbol(VALUE.0 + 109);
//     pub const FONTCOLOR: Symbol = Symbol(VALUE.0 + 110);
//     pub const FONTSIZE: Symbol = Symbol(VALUE.0 + 111);
//     pub const LINK: Symbol = Symbol(VALUE.0 + 112);
//     pub const TRIM: Symbol = Symbol(VALUE.0 + 113);
//     pub const TRIM_START: Symbol = Symbol(VALUE.0 + 114);
//     pub const TRIM_END: Symbol = Symbol(VALUE.0 + 115);
//     pub const SUBSTR: Symbol = Symbol(VALUE.0 + 116);
//     pub const SUBSTRING: Symbol = Symbol(VALUE.0 + 117);
//     pub const FROM: Symbol = Symbol(VALUE.0 + 118);
//     pub const IS_ARRAY: Symbol = Symbol(VALUE.0 + 119);
//     pub const ARRAY: Symbol = Symbol(VALUE.0 + 120);
//     pub const JOIN: Symbol = Symbol(VALUE.0 + 121);
//     pub const VALUES: Symbol = Symbol(VALUE.0 + 122);
//     pub const AT: Symbol = Symbol(VALUE.0 + 123);
//     pub const EVERY: Symbol = Symbol(VALUE.0 + 124);
//     pub const SOME: Symbol = Symbol(VALUE.0 + 125);
//     pub const FILL: Symbol = Symbol(VALUE.0 + 126);
//     pub const FILTER: Symbol = Symbol(VALUE.0 + 127);
//     pub const REDUCE: Symbol = Symbol(VALUE.0 + 128);
//     pub const FIND: Symbol = Symbol(VALUE.0 + 129);
//     pub const FIND_INDEX: Symbol = Symbol(VALUE.0 + 130);
//     pub const FLAT: Symbol = Symbol(VALUE.0 + 131);
//     pub const FOR_EACH: Symbol = Symbol(VALUE.0 + 132);
//     pub const LO_MAP: Symbol = Symbol(VALUE.0 + 133);
//     pub const POP: Symbol = Symbol(VALUE.0 + 134);
//     pub const PUSH: Symbol = Symbol(VALUE.0 + 135);
//     pub const REVERSE: Symbol = Symbol(VALUE.0 + 136);
//     pub const SHIFT: Symbol = Symbol(VALUE.0 + 137);
//     pub const SORT: Symbol = Symbol(VALUE.0 + 138);
//     pub const UNSHIFT: Symbol = Symbol(VALUE.0 + 139);
//     pub const SLICE: Symbol = Symbol(VALUE.0 + 140);
//     pub const NEXT: Symbol = Symbol(VALUE.0 + 141);
//     pub const ASYNC_ITERATOR: Symbol = Symbol(VALUE.0 + 142);
//     pub const HAS_INSTANCE: Symbol = Symbol(VALUE.0 + 143);
//     pub const ITERATOR: Symbol = Symbol(VALUE.0 + 144);
//     pub const MATCH: Symbol = Symbol(VALUE.0 + 145);
//     pub const MATCH_ALL: Symbol = Symbol(VALUE.0 + 146);
//     pub const SEARCH: Symbol = Symbol(VALUE.0 + 147);
//     pub const SPECIES: Symbol = Symbol(VALUE.0 + 148);
//     pub const TO_PRIMITIVE: Symbol = Symbol(VALUE.0 + 149);
//     pub const TO_STRING_TAG: Symbol = Symbol(VALUE.0 + 150);
//     pub const UNSCOPABLES: Symbol = Symbol(VALUE.0 + 151);
//     pub const SYMBOL: Symbol = Symbol(VALUE.0 + 152);
//     pub const ARRAY_BUFFER: Symbol = Symbol(VALUE.0 + 153);
//     pub const BYTE_LENGTH: Symbol = Symbol(VALUE.0 + 154);
//     pub const UINT8ARRAY: Symbol = Symbol(VALUE.0 + 155);
//     pub const INT8ARRAY: Symbol = Symbol(VALUE.0 + 156);
//     pub const UINT16ARRAY: Symbol = Symbol(VALUE.0 + 157);
//     pub const INT16ARRAY: Symbol = Symbol(VALUE.0 + 158);
//     pub const UINT32ARRAY: Symbol = Symbol(VALUE.0 + 159);
//     pub const INT32ARRAY: Symbol = Symbol(VALUE.0 + 160);
//     pub const FLOAT32ARRAY: Symbol = Symbol(VALUE.0 + 161);
//     pub const FLOAT64ARRAY: Symbol = Symbol(VALUE.0 + 162);
//     pub const RESOLVE: Symbol = Symbol(VALUE.0 + 163);
//     pub const REJECT: Symbol = Symbol(VALUE.0 + 164);
//     pub const PROMISE: Symbol = Symbol(VALUE.0 + 165);
//     pub const THEN: Symbol = Symbol(VALUE.0 + 166);
//     pub const SET: Symbol = Symbol(VALUE.0 + 167);
//     pub const ADD: Symbol = Symbol(VALUE.0 + 168);
//     pub const HAS: Symbol = Symbol(VALUE.0 + 169);
//     pub const DELETE: Symbol = Symbol(VALUE.0 + 170);
//     pub const CLEAR: Symbol = Symbol(VALUE.0 + 171);
//     pub const SIZE: Symbol = Symbol(VALUE.0 + 172);
//     pub const MAP: Symbol = Symbol(VALUE.0 + 173);
//     pub const REG_EXP: Symbol = Symbol(VALUE.0 + 174);
//     pub const TEST: Symbol = Symbol(VALUE.0 + 175);
//     pub const EXEC: Symbol = Symbol(VALUE.0 + 176);
//     pub const NOW: Symbol = Symbol(VALUE.0 + 177);
//     pub const DATE: Symbol = Symbol(VALUE.0 + 178);
//     pub const PARSE: Symbol = Symbol(VALUE.0 + 179);
//     pub const PARSE_FLOAT: Symbol = Symbol(VALUE.0 + 180);
//     pub const PARSE_INT: Symbol = Symbol(VALUE.0 + 181);
//     pub const CONSOLE: Symbol = Symbol(VALUE.0 + 182);
//     pub const MATH: Symbol = Symbol(VALUE.0 + 183);
//     pub const JSON: Symbol = Symbol(VALUE.0 + 184);
//     pub const IS_CONCAT_SPREADABLE: Symbol = Symbol(VALUE.0 + 185);
//     pub const DONE: Symbol = Symbol(VALUE.0 + 186);
//     pub const ZERO: Symbol = Symbol(VALUE.0 + 187);
//     pub const ONE: Symbol = Symbol(VALUE.0 + 188);
//     pub const COMMA: Symbol = Symbol(VALUE.0 + 189);

//     pub const PREINTERNED: &[(&str, Symbol)] = &[
//         ("value", VALUE),
//         ("name", NAME),
//         ("prototype", PROTOTYPE),
//         ("length", LENGTH),
//         ("message", MESSAGE),
//         ("stack", STACK),
//         ("Error", ERROR),
//         ("", EMPTY),
//         ("toString", TO_STRING),
//         ("valueOf", VALUE_OF),
//         ("undefined", UNDEFINED),
//         ("object", LO_OBJECT),
//         ("boolean", LO_BOOLEAN),
//         ("number", LO_NUMBER),
//         ("bigInt", LO_BIGINT),
//         ("string", LO_STRING),
//         ("symbol", LO_SYMBOL),
//         ("function", LO_FUNCTION),
//         ("get", GET),
//         ("set", LO_SET),
//         ("writable", WRITABLE),
//         ("enumerable", ENUMERABLE),
//         ("configurable", CONFIGURABLE),
//         ("__proto__", PROTO),
//         ("constructor", CONSTRUCTOR),
//         ("default", DEFAULT),
//         ("true", TRUE),
//         ("false", FALSE),
//         ("null", NULL),
//         ("EvalError", EVAL_ERROR),
//         ("RangeError", RANGE_ERROR),
//         ("ReferenceError", REFERENCE_ERROR),
//         ("SyntaxError", SYNTAX_ERROR),
//         ("TypeError", TYPE_ERROR),
//         ("URIError", URI_ERROR),
//         ("AggregateError", AGGREGATE_ERROR),
//         ("Function", FUNCTION),
//         ("bind", BIND),
//         ("call", CALL),
//         ("create", CREATE),
//         ("keys", KEYS),
//         ("getOwnPropertyDescriptor", GET_OWN_PROPERTY_DESCRIPTOR),
//         ("getOwnPropertyDescriptors", GET_OWN_PROPERTY_DESCRIPTORS),
//         ("defineProperty", DEFINE_PROPERTY),
//         ("entries", ENTRIES),
//         ("assign", ASSIGN),
//         ("Object", OBJECT),
//         ("hasOwnProperty", HAS_OWN_PROPERTY),
//         ("log", LOG),
//         ("floor", FLOOR),
//         ("abs", ABS),
//         ("acos", ACOS),
//         ("acosh", ACOSH),
//         ("asin", ASIN),
//         ("asinh", ASINH),
//         ("atan", ATAN),
//         ("atanh", ATANH),
//         ("atan2", ATAN2),
//         ("cbrt", CBRT),
//         ("ceil", CEIL),
//         ("clz32", CLZ32),
//         ("cos", COS),
//         ("cosh", COSH),
//         ("exp", EXP),
//         ("expm1", EXPM1),
//         ("log1p", LOG1P),
//         ("log10", LOG10),
//         ("log2", LOG2),
//         ("round", ROUND),
//         ("sin", SIN),
//         ("sinh", SINH),
//         ("sqrt", SQRT),
//         ("tan", TAN),
//         ("tanh", TANH),
//         ("trunc", TRUNC),
//         ("random", RANDOM),
//         ("max", MAX),
//         ("min", MIN),
//         ("PI", PI),
//         ("isFinite", IS_FINITE),
//         ("isNaN", IS_NA_N),
//         ("isSafeInteger", IS_SAFE_INTEGER),
//         ("Number", NUMBER),
//         ("toFixed", TO_FIXED),
//         ("Boolean", BOOLEAN),
//         ("fromCharCode", FROM_CHAR_CODE),
//         ("String", STRING),
//         ("charAt", CHAR_AT),
//         ("charCodeAt", CHAR_CODE_AT),
//         ("concat", CONCAT),
//         ("endsWith", ENDS_WITH),
//         ("startsWith", STARTS_WITH),
//         ("includes", INCLUDES),
//         ("indexOf", INDEX_OF),
//         ("lastIndexOf", LAST_INDEX_OF),
//         ("padEnd", PAD_END),
//         ("padStart", PAD_START),
//         ("repeat", REPEAT),
//         ("replace", REPLACE),
//         ("replaceAll", REPLACE_ALL),
//         ("split", SPLIT),
//         ("toLowerCase", TO_LOWER_CASE),
//         ("toUpperCase", TO_UPPER_CASE),
//         ("big", BIG),
//         ("blink", BLINK),
//         ("bold", BOLD),
//         ("fixed", FIXED),
//         ("italics", ITALICS),
//         ("strike", STRIKE),
//         ("sub", SUB),
//         ("sup", SUP),
//         ("fontcolor", FONTCOLOR),
//         ("fontsize", FONTSIZE),
//         ("link", LINK),
//         ("trim", TRIM),
//         ("trimStart", TRIM_START),
//         ("trimEnd", TRIM_END),
//         ("substr", SUBSTR),
//         ("substring", SUBSTRING),
//         ("from", FROM),
//         ("isArray", IS_ARRAY),
//         ("Array", ARRAY),
//         ("join", JOIN),
//         ("values", VALUES),
//         ("at", AT),
//         ("every", EVERY),
//         ("some", SOME),
//         ("fill", FILL),
//         ("filter", FILTER),
//         ("reduce", REDUCE),
//         ("find", FIND),
//         ("findIndex", FIND_INDEX),
//         ("flat", FLAT),
//         ("forEach", FOR_EACH),
//         ("map", LO_MAP),
//         ("pop", POP),
//         ("push", PUSH),
//         ("reverse", REVERSE),
//         ("shift", SHIFT),
//         ("sort", SORT),
//         ("unshift", UNSHIFT),
//         ("slice", SLICE),
//         ("next", NEXT),
//         ("asyncIterator", ASYNC_ITERATOR),
//         ("hasInstance", HAS_INSTANCE),
//         ("iterator", ITERATOR),
//         ("match", MATCH),
//         ("matchAll", MATCH_ALL),
//         ("search", SEARCH),
//         ("species", SPECIES),
//         ("toPrimitive", TO_PRIMITIVE),
//         ("toStringTag", TO_STRING_TAG),
//         ("unscopables", UNSCOPABLES),
//         ("Symbol", SYMBOL),
//         ("ArrayBuffer", ARRAY_BUFFER),
//         ("byteLength", BYTE_LENGTH),
//         ("Uint8Array", UINT8ARRAY),
//         ("Int8Array", INT8ARRAY),
//         ("Uint16Array", UINT16ARRAY),
//         ("Int16Array", INT16ARRAY),
//         ("Uint32Array", UINT32ARRAY),
//         ("Int32Array", INT32ARRAY),
//         ("Float32Array", FLOAT32ARRAY),
//         ("Float64Array", FLOAT64ARRAY),
//         ("resolve", RESOLVE),
//         ("reject", REJECT),
//         ("Promise", PROMISE),
//         ("then", THEN),
//         ("Set", SET),
//         ("add", ADD),
//         ("has", HAS),
//         ("delete", DELETE),
//         ("clear", CLEAR),
//         ("size", SIZE),
//         ("Map", MAP),
//         ("RegExp", REG_EXP),
//         ("test", TEST),
//         ("exec", EXEC),
//         ("now", NOW),
//         ("Date", DATE),
//         ("parse", PARSE),
//         ("parseFloat", PARSE_FLOAT),
//         ("parseInt", PARSE_INT),
//         ("console", CONSOLE),
//         ("Math", MATH),
//         ("JSON", JSON),
//         ("isConcatSpreadable", IS_CONCAT_SPREADABLE),
//         ("done", DONE),
//         ("0", ZERO),
//         ("1", ONE),
//         (",", COMMA),
//     ];
// }

// fn fxhash(s: &str) -> u64 {
//     let mut hasher = FxHasher::default();
//     s.hash(&mut hasher);
//     hasher.finish()
// }

// pub struct StringData {
//     string: Rc<str>,
// }

// pub struct StringInterner {
//     storage: Vec<Option<StringData>>,
//     mapping: hashbrown::HashMap<Rc<str>, Symbol, BuildHasherDefault<FxHasher>>,
//     /// List of free indices in the storage
//     free: Vec<RawSymbol>,
// }

// impl StringInterner {
//     pub fn new() -> Self {
//         let mut storage = Vec::new();
//         let mut mapping = hashbrown::HashMap::default();

//         for (name, sym) in sym::PREINTERNED {
//             let string = Rc::from(*name);
//             debug_assert_eq!(sym.0, storage.len() as RawSymbol);
//             storage.push(Some(StringData { string }));
//         }

//         Self {
//             storage,
//             mapping,
//             free: Vec::new(),
//         }
//     }

//     pub fn intern(&mut self, string: impl Borrow<str>) -> Symbol {
//         let string = string.borrow();
//         let hash = fxhash(string);
//         match self.mapping.entry_ref(string) {
//             EntryRef::Occupied(occ) => occ.get().clone(),
//             EntryRef::Vacant(vac) => {
//                 if let Some(id) = self.free.pop() {
//                     self.storage[id as usize] = Some(StringData {
//                         string: Rc::from(string),
//                     });
//                     vac.insert(Symbol(id));
//                     Symbol(id)
//                 } else {
//                     let id: RawSymbol = self.storage.len().try_into().expect("too many strings");
//                     let string = Rc::from(string);
//                     self.storage.push(Some(StringData { string }));
//                     vac.insert(Symbol(id));
//                     Symbol(id)
//                 }
//             }
//         }
//     }

//     pub fn intern_usize(&mut self, val: usize) -> Symbol {
//         // for now this just calls `intern`, but we might want to specialize this
//         let string = val.to_string();
//         self.intern(string.as_ref())
//     }

//     pub fn intern_isize(&mut self, val: isize) -> Symbol {
//         // for now this just calls `intern`, but we might want to specialize this
//         let string = val.to_string();
//         self.intern(string.as_ref())
//     }

//     pub fn intern_char(&mut self, val: char) -> Symbol {
//         // for now this just calls `intern`, but we might want to specialize this
//         let string = val.to_string();
//         self.intern(string.as_ref())
//     }

//     // pub fn remove(&mut self, symbol: Symbol) {
//     //     self.storage[symbol.0 as usize] = None;
//     //     self.free.push(symbol.0);
//     // }

//     pub fn resolve(&mut self, symbol: Symbol) -> &str {
//         &self.storage[symbol.0 as usize]
//             .as_ref()
//             .expect("tombstone symbol")
//             .string
//     }
// }

// type RawSymbol = u32;

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub struct Symbol(RawSymbol);

// impl Symbol {}

// unsafe impl Trace for Symbol {
//     fn trace(&self, cx: &mut TraceCtxt<'_>) {
//         todo!();
//     }
// }

// // TODO: implement Trace for StringInterner which marks the preinterned strings
