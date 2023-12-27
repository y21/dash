use std::hash::{BuildHasherDefault, Hash, Hasher};
use std::rc::Rc;
use std::{borrow, fmt};

use hashbrown::hash_map::RawEntryMut;
use rustc_hash::FxHasher;
use serde::{Deserialize, Serialize};

pub mod sym {
    use super::Symbol;

    pub const IF: Symbol = Symbol(0);
    pub const ELSE: Symbol = Symbol(1);
    pub const LO_FUNCTION: Symbol = Symbol(2);
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
    pub const LO_SET: Symbol = Symbol(40);

    pub const PREINTERNED: &[(&str, Symbol)] = &[
        ("if", IF),
        ("else", ELSE),
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
        ("name", NAME),
        ("length", LENGTH),
        ("message", MESSAGE),
        ("stack", STACK),
        ("Error", ERROR),
        ("toString", TO_STRING),
        ("valueOf", VALUE_OF),
        ("undefined", UNDEFINED),
        ("object", LO_OBJECT),
        ("boolean", LO_BOOLEAN),
        ("number", LO_NUMBER),
        ("bigInt", LO_BIGINT),
        ("string", LO_STRING),
        ("symbol", LO_SYMBOL),
        ("function", LO_FUNCTION),
        ("set", LO_SET),
        ("writable", WRITABLE),
        ("enumerable", ENUMERABLE),
        ("configurable", CONFIGURABLE),
        ("__proto__", PROTO),
        ("true", TRUE),
        ("false", FALSE),
        ("null", NULL),
        ("EvalError", EVAL_ERROR),
        ("RangeError", RANGE_ERROR),
        ("ReferenceError", REFERENCE_ERROR),
        ("SyntaxError", SYNTAX_ERROR),
        ("TypeError", TYPE_ERROR),
        ("URIError", URI_ERROR),
        ("AggregateError", AGGREGATE_ERROR),
        ("Function", FUNCTION),
        ("bind", BIND),
        ("call", CALL),
        ("create", CREATE),
        ("keys", KEYS),
        ("getOwnPropertyDescriptor", GET_OWN_PROPERTY_DESCRIPTOR),
        ("getOwnPropertyDescriptors", GET_OWN_PROPERTY_DESCRIPTORS),
        ("defineProperty", DEFINE_PROPERTY),
        ("entries", ENTRIES),
        ("assign", ASSIGN),
        ("Object", OBJECT),
        ("hasOwnProperty", HAS_OWN_PROPERTY),
        ("tanh", TANH),
        ("max", MAX),
        ("min", MIN),
        ("PI", PI),
        ("isFinite", IS_FINITE),
        ("isNaN", IS_NA_N),
        ("isSafeInteger", IS_SAFE_INTEGER),
        ("Number", NUMBER),
        ("toFixed", TO_FIXED),
        ("Boolean", BOOLEAN),
        ("fromCharCode", FROM_CHAR_CODE),
        ("String", STRING),
        ("charAt", CHAR_AT),
        ("charCodeAt", CHAR_CODE_AT),
        ("concat", CONCAT),
        ("endsWith", ENDS_WITH),
        ("startsWith", STARTS_WITH),
        ("includes", INCLUDES),
        ("indexOf", INDEX_OF),
        ("lastIndexOf", LAST_INDEX_OF),
        ("padEnd", PAD_END),
        ("padStart", PAD_START),
        ("repeat", REPEAT),
        ("replace", REPLACE),
        ("replaceAll", REPLACE_ALL),
        ("split", SPLIT),
        ("toLowerCase", TO_LOWER_CASE),
        ("toUpperCase", TO_UPPER_CASE),
        ("big", BIG),
        ("blink", BLINK),
        ("bold", BOLD),
        ("fixed", FIXED),
        ("italics", ITALICS),
        ("strike", STRIKE),
        ("sub", SUB),
        ("sup", SUP),
        ("fontcolor", FONTCOLOR),
        ("fontsize", FONTSIZE),
        ("link", LINK),
        ("trim", TRIM),
        ("trimStart", TRIM_START),
        ("trimEnd", TRIM_END),
        ("substr", SUBSTR),
        ("substring", SUBSTRING),
        ("from", FROM),
        ("isArray", IS_ARRAY),
        ("Array", ARRAY),
        ("join", JOIN),
        ("values", VALUES),
        ("at", AT),
        ("every", EVERY),
        ("some", SOME),
        ("fill", FILL),
        ("filter", FILTER),
        ("reduce", REDUCE),
        ("find", FIND),
        ("findIndex", FIND_INDEX),
        ("flat", FLAT),
        ("forEach", FOR_EACH),
        ("map", LO_MAP),
        ("pop", POP),
        ("push", PUSH),
        ("reverse", REVERSE),
        ("shift", SHIFT),
        ("sort", SORT),
        ("unshift", UNSHIFT),
        ("slice", SLICE),
        ("next", NEXT),
        ("asyncIterator", ASYNC_ITERATOR),
        ("hasInstance", HAS_INSTANCE),
        ("iterator", ITERATOR),
        ("match", MATCH),
        ("matchAll", MATCH_ALL),
        ("search", SEARCH),
        ("species", SPECIES),
        ("toPrimitive", TO_PRIMITIVE),
        ("toStringTag", TO_STRING_TAG),
        ("unscopables", UNSCOPABLES),
        ("Symbol", SYMBOL),
        ("ArrayBuffer", ARRAY_BUFFER),
        ("byteLength", BYTE_LENGTH),
        ("Uint8Array", UINT8ARRAY),
        ("Int8Array", INT8ARRAY),
        ("Uint16Array", UINT16ARRAY),
        ("Int16Array", INT16ARRAY),
        ("Uint32Array", UINT32ARRAY),
        ("Int32Array", INT32ARRAY),
        ("Float32Array", FLOAT32ARRAY),
        ("Float64Array", FLOAT64ARRAY),
        ("resolve", RESOLVE),
        ("reject", REJECT),
        ("Promise", PROMISE),
        ("then", THEN),
        ("Set", SET),
        ("add", ADD),
        ("has", HAS),
        ("delete", DELETE),
        ("clear", CLEAR),
        ("size", SIZE),
        ("Map", MAP),
        ("RegExp", REG_EXP),
        ("test", TEST),
        ("exec", EXEC),
        ("now", NOW),
        ("Date", DATE),
        ("parse", PARSE),
        ("parseFloat", PARSE_FLOAT),
        ("parseInt", PARSE_INT),
        ("console", CONSOLE),
        ("JSON", JSON),
        ("isConcatSpreadable", IS_CONCAT_SPREADABLE),
        ("0", ZERO),
        ("1", ONE),
        (",", COMMA),
    ];

    // ⚠️⚠️⚠️⚠️ Update these constants when adding a keyword.
    // We rely on the fact that the keywords are contiguous in the symbol table,
    // making it very easy and cheap to check if a symbol is a keyword.
    // TODO: automate this with a proc macro or sorts.
    pub const KEYWORD_START: Symbol = IF;
    pub const KEYWORD_END: Symbol = LO_SET;

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

    // ⚠️⚠️⚠️⚠️ Update these constants when adding a post-keyword symbol.
    pub const PRE_VM_INTERNED_END: Symbol = PROTOTYPE;

    pub const VM_PREINTERNED_START: Symbol = Symbol(PRE_VM_INTERNED_END.0 + 1);
    pub const NAME: Symbol = VM_PREINTERNED_START;

    pub const LENGTH: Symbol = Symbol(VM_PREINTERNED_START.0 + 1);
    pub const MESSAGE: Symbol = Symbol(VM_PREINTERNED_START.0 + 2);
    pub const STACK: Symbol = Symbol(VM_PREINTERNED_START.0 + 3);
    pub const ERROR: Symbol = Symbol(VM_PREINTERNED_START.0 + 4);
    pub const TO_STRING: Symbol = Symbol(VM_PREINTERNED_START.0 + 5);
    pub const VALUE_OF: Symbol = Symbol(VM_PREINTERNED_START.0 + 6);
    pub const UNDEFINED: Symbol = Symbol(VM_PREINTERNED_START.0 + 7);
    pub const LO_OBJECT: Symbol = Symbol(VM_PREINTERNED_START.0 + 8);
    pub const LO_BOOLEAN: Symbol = Symbol(VM_PREINTERNED_START.0 + 9);
    pub const LO_NUMBER: Symbol = Symbol(VM_PREINTERNED_START.0 + 10);
    pub const LO_BIGINT: Symbol = Symbol(VM_PREINTERNED_START.0 + 11);
    pub const LO_STRING: Symbol = Symbol(VM_PREINTERNED_START.0 + 12);
    pub const LO_SYMBOL: Symbol = Symbol(VM_PREINTERNED_START.0 + 13);
    pub const FUNCTION: Symbol = Symbol(VM_PREINTERNED_START.0 + 14);
    pub const COMMA: Symbol = Symbol(VM_PREINTERNED_START.0 + 15);
    pub const WRITABLE: Symbol = Symbol(VM_PREINTERNED_START.0 + 16);
    pub const ENUMERABLE: Symbol = Symbol(VM_PREINTERNED_START.0 + 17);
    pub const CONFIGURABLE: Symbol = Symbol(VM_PREINTERNED_START.0 + 18);
    pub const PROTO: Symbol = Symbol(VM_PREINTERNED_START.0 + 19);
    pub const TRUE: Symbol = Symbol(VM_PREINTERNED_START.0 + 20);
    pub const FALSE: Symbol = Symbol(VM_PREINTERNED_START.0 + 21);
    pub const NULL: Symbol = Symbol(VM_PREINTERNED_START.0 + 22);
    pub const EVAL_ERROR: Symbol = Symbol(VM_PREINTERNED_START.0 + 23);
    pub const RANGE_ERROR: Symbol = Symbol(VM_PREINTERNED_START.0 + 24);
    pub const REFERENCE_ERROR: Symbol = Symbol(VM_PREINTERNED_START.0 + 25);
    pub const SYNTAX_ERROR: Symbol = Symbol(VM_PREINTERNED_START.0 + 26);
    pub const TYPE_ERROR: Symbol = Symbol(VM_PREINTERNED_START.0 + 27);
    pub const URI_ERROR: Symbol = Symbol(VM_PREINTERNED_START.0 + 28);
    pub const AGGREGATE_ERROR: Symbol = Symbol(VM_PREINTERNED_START.0 + 29);

    pub const BIND: Symbol = Symbol(VM_PREINTERNED_START.0 + 30);
    pub const CALL: Symbol = Symbol(VM_PREINTERNED_START.0 + 31);
    pub const CREATE: Symbol = Symbol(VM_PREINTERNED_START.0 + 32);
    pub const KEYS: Symbol = Symbol(VM_PREINTERNED_START.0 + 33);
    pub const GET_OWN_PROPERTY_DESCRIPTOR: Symbol = Symbol(VM_PREINTERNED_START.0 + 34);
    pub const GET_OWN_PROPERTY_DESCRIPTORS: Symbol = Symbol(VM_PREINTERNED_START.0 + 35);
    pub const DEFINE_PROPERTY: Symbol = Symbol(VM_PREINTERNED_START.0 + 36);
    pub const ENTRIES: Symbol = Symbol(VM_PREINTERNED_START.0 + 37);
    pub const ASSIGN: Symbol = Symbol(VM_PREINTERNED_START.0 + 38);
    pub const OBJECT: Symbol = Symbol(VM_PREINTERNED_START.0 + 39);
    pub const HAS_OWN_PROPERTY: Symbol = Symbol(VM_PREINTERNED_START.0 + 40);
    pub const TANH: Symbol = Symbol(VM_PREINTERNED_START.0 + 41);
    pub const MAX: Symbol = Symbol(VM_PREINTERNED_START.0 + 42);
    pub const MIN: Symbol = Symbol(VM_PREINTERNED_START.0 + 43);
    pub const PI: Symbol = Symbol(VM_PREINTERNED_START.0 + 44);
    pub const IS_FINITE: Symbol = Symbol(VM_PREINTERNED_START.0 + 45);
    pub const IS_NA_N: Symbol = Symbol(VM_PREINTERNED_START.0 + 46);
    pub const IS_SAFE_INTEGER: Symbol = Symbol(VM_PREINTERNED_START.0 + 47);
    pub const NUMBER: Symbol = Symbol(VM_PREINTERNED_START.0 + 48);
    pub const TO_FIXED: Symbol = Symbol(VM_PREINTERNED_START.0 + 49);
    pub const BOOLEAN: Symbol = Symbol(VM_PREINTERNED_START.0 + 50);
    pub const FROM_CHAR_CODE: Symbol = Symbol(VM_PREINTERNED_START.0 + 51);
    pub const STRING: Symbol = Symbol(VM_PREINTERNED_START.0 + 52);
    pub const CHAR_AT: Symbol = Symbol(VM_PREINTERNED_START.0 + 53);
    pub const CHAR_CODE_AT: Symbol = Symbol(VM_PREINTERNED_START.0 + 54);
    pub const CONCAT: Symbol = Symbol(VM_PREINTERNED_START.0 + 55);
    pub const ENDS_WITH: Symbol = Symbol(VM_PREINTERNED_START.0 + 56);
    pub const STARTS_WITH: Symbol = Symbol(VM_PREINTERNED_START.0 + 57);
    pub const INCLUDES: Symbol = Symbol(VM_PREINTERNED_START.0 + 58);
    pub const INDEX_OF: Symbol = Symbol(VM_PREINTERNED_START.0 + 59);
    pub const LAST_INDEX_OF: Symbol = Symbol(VM_PREINTERNED_START.0 + 60);
    pub const PAD_END: Symbol = Symbol(VM_PREINTERNED_START.0 + 61);
    pub const PAD_START: Symbol = Symbol(VM_PREINTERNED_START.0 + 62);
    pub const REPEAT: Symbol = Symbol(VM_PREINTERNED_START.0 + 63);
    pub const REPLACE: Symbol = Symbol(VM_PREINTERNED_START.0 + 64);
    pub const REPLACE_ALL: Symbol = Symbol(VM_PREINTERNED_START.0 + 65);
    pub const SPLIT: Symbol = Symbol(VM_PREINTERNED_START.0 + 66);
    pub const TO_LOWER_CASE: Symbol = Symbol(VM_PREINTERNED_START.0 + 67);
    pub const TO_UPPER_CASE: Symbol = Symbol(VM_PREINTERNED_START.0 + 68);
    pub const BIG: Symbol = Symbol(VM_PREINTERNED_START.0 + 69);
    pub const BLINK: Symbol = Symbol(VM_PREINTERNED_START.0 + 70);
    pub const BOLD: Symbol = Symbol(VM_PREINTERNED_START.0 + 71);
    pub const FIXED: Symbol = Symbol(VM_PREINTERNED_START.0 + 72);
    pub const ITALICS: Symbol = Symbol(VM_PREINTERNED_START.0 + 73);
    pub const STRIKE: Symbol = Symbol(VM_PREINTERNED_START.0 + 74);
    pub const SUB: Symbol = Symbol(VM_PREINTERNED_START.0 + 75);
    pub const SUP: Symbol = Symbol(VM_PREINTERNED_START.0 + 76);
    pub const FONTCOLOR: Symbol = Symbol(VM_PREINTERNED_START.0 + 77);
    pub const FONTSIZE: Symbol = Symbol(VM_PREINTERNED_START.0 + 78);
    pub const LINK: Symbol = Symbol(VM_PREINTERNED_START.0 + 79);
    pub const TRIM: Symbol = Symbol(VM_PREINTERNED_START.0 + 80);
    pub const TRIM_START: Symbol = Symbol(VM_PREINTERNED_START.0 + 81);
    pub const TRIM_END: Symbol = Symbol(VM_PREINTERNED_START.0 + 82);
    pub const SUBSTR: Symbol = Symbol(VM_PREINTERNED_START.0 + 83);
    pub const SUBSTRING: Symbol = Symbol(VM_PREINTERNED_START.0 + 84);
    pub const FROM: Symbol = Symbol(VM_PREINTERNED_START.0 + 85);
    pub const IS_ARRAY: Symbol = Symbol(VM_PREINTERNED_START.0 + 86);
    pub const ARRAY: Symbol = Symbol(VM_PREINTERNED_START.0 + 87);
    pub const JOIN: Symbol = Symbol(VM_PREINTERNED_START.0 + 88);
    pub const VALUES: Symbol = Symbol(VM_PREINTERNED_START.0 + 89);
    pub const AT: Symbol = Symbol(VM_PREINTERNED_START.0 + 90);
    pub const EVERY: Symbol = Symbol(VM_PREINTERNED_START.0 + 91);
    pub const SOME: Symbol = Symbol(VM_PREINTERNED_START.0 + 92);
    pub const FILL: Symbol = Symbol(VM_PREINTERNED_START.0 + 93);
    pub const FILTER: Symbol = Symbol(VM_PREINTERNED_START.0 + 94);
    pub const REDUCE: Symbol = Symbol(VM_PREINTERNED_START.0 + 95);
    pub const FIND: Symbol = Symbol(VM_PREINTERNED_START.0 + 96);
    pub const FIND_INDEX: Symbol = Symbol(VM_PREINTERNED_START.0 + 97);
    pub const FLAT: Symbol = Symbol(VM_PREINTERNED_START.0 + 98);
    pub const FOR_EACH: Symbol = Symbol(VM_PREINTERNED_START.0 + 99);
    pub const LO_MAP: Symbol = Symbol(VM_PREINTERNED_START.0 + 100);
    pub const POP: Symbol = Symbol(VM_PREINTERNED_START.0 + 101);
    pub const PUSH: Symbol = Symbol(VM_PREINTERNED_START.0 + 102);
    pub const REVERSE: Symbol = Symbol(VM_PREINTERNED_START.0 + 103);
    pub const SHIFT: Symbol = Symbol(VM_PREINTERNED_START.0 + 104);
    pub const SORT: Symbol = Symbol(VM_PREINTERNED_START.0 + 105);
    pub const UNSHIFT: Symbol = Symbol(VM_PREINTERNED_START.0 + 106);
    pub const SLICE: Symbol = Symbol(VM_PREINTERNED_START.0 + 107);
    pub const ASYNC_ITERATOR: Symbol = Symbol(VM_PREINTERNED_START.0 + 108);
    pub const HAS_INSTANCE: Symbol = Symbol(VM_PREINTERNED_START.0 + 109);
    pub const ITERATOR: Symbol = Symbol(VM_PREINTERNED_START.0 + 110);
    pub const MATCH: Symbol = Symbol(VM_PREINTERNED_START.0 + 111);
    pub const MATCH_ALL: Symbol = Symbol(VM_PREINTERNED_START.0 + 112);
    pub const SEARCH: Symbol = Symbol(VM_PREINTERNED_START.0 + 113);
    pub const SPECIES: Symbol = Symbol(VM_PREINTERNED_START.0 + 114);
    pub const TO_PRIMITIVE: Symbol = Symbol(VM_PREINTERNED_START.0 + 115);
    pub const TO_STRING_TAG: Symbol = Symbol(VM_PREINTERNED_START.0 + 116);
    pub const UNSCOPABLES: Symbol = Symbol(VM_PREINTERNED_START.0 + 117);
    pub const SYMBOL: Symbol = Symbol(VM_PREINTERNED_START.0 + 118);
    pub const ARRAY_BUFFER: Symbol = Symbol(VM_PREINTERNED_START.0 + 119);
    pub const BYTE_LENGTH: Symbol = Symbol(VM_PREINTERNED_START.0 + 120);
    pub const UINT8ARRAY: Symbol = Symbol(VM_PREINTERNED_START.0 + 121);
    pub const INT8ARRAY: Symbol = Symbol(VM_PREINTERNED_START.0 + 122);
    pub const UINT16ARRAY: Symbol = Symbol(VM_PREINTERNED_START.0 + 123);
    pub const INT16ARRAY: Symbol = Symbol(VM_PREINTERNED_START.0 + 124);
    pub const UINT32ARRAY: Symbol = Symbol(VM_PREINTERNED_START.0 + 125);
    pub const INT32ARRAY: Symbol = Symbol(VM_PREINTERNED_START.0 + 126);
    pub const FLOAT32ARRAY: Symbol = Symbol(VM_PREINTERNED_START.0 + 127);
    pub const FLOAT64ARRAY: Symbol = Symbol(VM_PREINTERNED_START.0 + 128);
    pub const RESOLVE: Symbol = Symbol(VM_PREINTERNED_START.0 + 129);
    pub const REJECT: Symbol = Symbol(VM_PREINTERNED_START.0 + 130);
    pub const PROMISE: Symbol = Symbol(VM_PREINTERNED_START.0 + 131);
    pub const THEN: Symbol = Symbol(VM_PREINTERNED_START.0 + 132);
    pub const ADD: Symbol = Symbol(VM_PREINTERNED_START.0 + 133);
    pub const HAS: Symbol = Symbol(VM_PREINTERNED_START.0 + 134);
    pub const CLEAR: Symbol = Symbol(VM_PREINTERNED_START.0 + 135);
    pub const SIZE: Symbol = Symbol(VM_PREINTERNED_START.0 + 136);
    pub const MAP: Symbol = Symbol(VM_PREINTERNED_START.0 + 137);
    pub const REG_EXP: Symbol = Symbol(VM_PREINTERNED_START.0 + 138);
    pub const TEST: Symbol = Symbol(VM_PREINTERNED_START.0 + 139);
    pub const EXEC: Symbol = Symbol(VM_PREINTERNED_START.0 + 140);
    pub const NOW: Symbol = Symbol(VM_PREINTERNED_START.0 + 141);
    pub const DATE: Symbol = Symbol(VM_PREINTERNED_START.0 + 142);
    pub const PARSE: Symbol = Symbol(VM_PREINTERNED_START.0 + 143);
    pub const PARSE_FLOAT: Symbol = Symbol(VM_PREINTERNED_START.0 + 144);
    pub const PARSE_INT: Symbol = Symbol(VM_PREINTERNED_START.0 + 145);
    pub const CONSOLE: Symbol = Symbol(VM_PREINTERNED_START.0 + 146);
    pub const JSON: Symbol = Symbol(VM_PREINTERNED_START.0 + 147);
    pub const IS_CONCAT_SPREADABLE: Symbol = Symbol(VM_PREINTERNED_START.0 + 148);
    pub const ZERO: Symbol = Symbol(VM_PREINTERNED_START.0 + 149);
    pub const ONE: Symbol = Symbol(VM_PREINTERNED_START.0 + 150);
    pub const SET: Symbol = Symbol(VM_PREINTERNED_START.0 + 151);
}

#[derive(Default, Debug)]
pub struct StringInterner {
    store: Vec<Option<Rc<str>>>,
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
            println!("{s} {}", index.0);
            debug_assert!(store.len() == index.0 as usize);
            mapping.insert(s.clone(), index.0);
            store.push(Some(s));
        }

        Self {
            store,
            mapping,
            free: Vec::new(),
        }
    }

    pub fn resolve(&self, symbol: Symbol) -> &str {
        self.store[symbol.0 as usize].as_ref().unwrap()
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
                    self.store[id as usize] = Some(Rc::clone(&value));
                    entry.insert_hashed_nocheck(hash, value, id);
                    Symbol(id)
                } else {
                    let id = self.store.len() as RawSymbol;
                    let value: Rc<str> = Rc::from(value);
                    self.store.push(Some(Rc::clone(&value)));
                    entry.insert_hashed_nocheck(hash, value, id);
                    Symbol(id)
                }
            }
        }
    }

    pub fn intern_usize(&mut self, val: usize) -> Symbol {
        // for now this just calls `intern`, but we might want to specialize this
        let string = val.to_string();
        self.intern(string.as_ref())
    }

    pub fn intern_isize(&mut self, val: isize) -> Symbol {
        // for now this just calls `intern`, but we might want to specialize this
        let string = val.to_string();
        self.intern(string.as_ref())
    }

    pub fn intern_char(&mut self, val: char) -> Symbol {
        // for now this just calls `intern`, but we might want to specialize this
        let string = val.to_string();
        self.intern(string.as_ref())
    }

    // pub fn remove(&mut self, symbol: Symbol) {
    //     self.storage[symbol.0 as usize] = None;
    //     self.free.push(symbol.0);
    // }
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

        self.0 >= sym::KEYWORD_START.0 && self.0 <= sym::KEYWORD_END.0
    }
}
