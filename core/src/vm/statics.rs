use crate::gc::{Gc, Handle};
use crate::js_std;
use crate::vm::value::function::Constructor;
use crate::vm::value::object::ObjectKind;

use super::value::function::{NativeFunction, NativeFunctionCallback};
use super::value::object::Object;
use super::value::symbol::Symbol;

/// Static data used by the VM
pub struct Statics {
    /// Represents Boolean.prototype
    pub boolean_proto: Handle<Object>,
    /// Represents Number.prototype
    pub number_proto: Handle<Object>,
    /// Represents String.prototype
    pub string_proto: Handle<Object>,
    /// Represents Function.prototype
    pub function_proto: Handle<Object>,
    /// Represents Array.prototype
    pub array_proto: Handle<Object>,
    /// Represents WeakSet.prototype
    pub weakset_proto: Handle<Object>,
    /// Represents WeakMap.prototype
    pub weakmap_proto: Handle<Object>,
    /// Represents Object.prototype
    pub object_proto: Handle<Object>,
    /// Represents Error.prototype
    pub error_proto: Handle<Object>,
    /// Represents the prototype of a generator iterator
    pub generator_iterator_proto: Handle<Object>,
    /// Represents Promise.prototype
    pub promise_proto: Handle<Object>,
    /// Represents Symbol.prototype
    pub symbol_proto: Handle<Object>,
    /// Represents the Boolean constructor
    pub boolean_ctor: Handle<Object>,
    /// Represents the Number constructor
    pub number_ctor: Handle<Object>,
    /// Represents the String constructor
    pub string_ctor: Handle<Object>,
    /// Represents the Function constructor
    pub function_ctor: Handle<Object>,
    /// Represents the Array constructor
    pub array_ctor: Handle<Object>,
    /// Represents the WeakSet constructor
    pub weakset_ctor: Handle<Object>,
    /// Represents the WeakMap constructor
    pub weakmap_ctor: Handle<Object>,
    /// Represents the Object constructor
    pub object_ctor: Handle<Object>,
    /// Represents the Error constructor
    pub error_ctor: Handle<Object>,
    /// Represents the Promise constructor
    pub promise_ctor: Handle<Object>,
    /// Represents the Symbol constructor
    pub symbol_ctor: Handle<Object>,
    /// Represents Symbol.for
    pub symbol_for: Handle<Object>,
    /// Represents Symbol.keyFor
    pub symbol_key_for: Handle<Object>,
    /// Represents console.log
    pub console_log: Handle<Object>,
    /// Represents isNaN
    pub isnan: Handle<Object>,
    /// Represents GeneratorIterator.prototype.next
    pub generator_iterator_next: Handle<Object>,
    /// Represents GeneratorIterator.prototype.return
    pub generator_iterator_return: Handle<Object>,
    /// Represents Array.prototype.push
    pub array_push: Handle<Object>,
    /// Represents Array.prototype.concat
    pub array_concat: Handle<Object>,
    /// Represents Array.prototype.map
    pub array_map: Handle<Object>,
    /// Represents Array.prototype.every
    pub array_every: Handle<Object>,
    /// Represents Array.prototype.fill
    pub array_fill: Handle<Object>,
    /// Represents Array.prototype.filter
    pub array_filter: Handle<Object>,
    /// Represents Array.prototype.find
    pub array_find: Handle<Object>,
    /// Represents Array.prototype.findIndex
    pub array_find_index: Handle<Object>,
    /// Represents Array.prototype.flat
    pub array_flat: Handle<Object>,
    /// Represents Array.prototype.forEach
    pub array_for_each: Handle<Object>,
    /// Represents Array.from
    pub array_from: Handle<Object>,
    /// Represents Array.prototype.includes
    pub array_includes: Handle<Object>,
    /// Represents Array.prototype.indexOf
    pub array_index_of: Handle<Object>,
    /// Represents Array.isArray
    pub array_is_array: Handle<Object>,
    /// Represents Array.prototype.join
    pub array_join: Handle<Object>,
    /// Represents Array.prototype.lastIndexOf
    pub array_last_index_of: Handle<Object>,
    /// Represents Array.of
    pub array_of: Handle<Object>,
    /// Represents Array.prototype.pop
    pub array_pop: Handle<Object>,
    /// Represents Array.prototype.reduce
    pub array_reduce: Handle<Object>,
    /// Represents Array.prototype.reduceRight
    pub array_reduce_right: Handle<Object>,
    /// Represents Array.prototype.reverse
    pub array_reverse: Handle<Object>,
    /// Represents Array.prototype.shift
    pub array_shift: Handle<Object>,
    /// Represents Array.prototype.slice
    pub array_slice: Handle<Object>,
    /// Represents Array.prototype.some
    pub array_some: Handle<Object>,
    /// Represents Array.prototype.sort
    pub array_sort: Handle<Object>,
    /// Represents Array.prototype.splice
    pub array_splice: Handle<Object>,
    /// Represents Array.prototype.unshift
    pub array_unshift: Handle<Object>,
    /// Represents String.prototype.charAt
    pub string_char_at: Handle<Object>,
    /// Represents String.prototype.charCodeAt
    pub string_char_code_at: Handle<Object>,
    /// Represents String.prototype.endsWith
    pub string_ends_with: Handle<Object>,
    /// Represents String.prototype.anchor
    pub string_anchor: Handle<Object>,
    /// Represents String.prototype.big
    pub string_big: Handle<Object>,
    /// Represents String.prototype.blink
    pub string_blink: Handle<Object>,
    /// Represents String.prototype.bold
    pub string_bold: Handle<Object>,
    /// Represents String.prototype.fixed
    pub string_fixed: Handle<Object>,
    /// Represents String.prototype.fontcolor
    pub string_fontcolor: Handle<Object>,
    /// Represents String.prototype.fontsize
    pub string_fontsize: Handle<Object>,
    /// Represents String.prototype.italics
    pub string_italics: Handle<Object>,
    /// Represents String.prototype.link
    pub string_link: Handle<Object>,
    /// Represents String.prototype.small
    pub string_small: Handle<Object>,
    /// Represents String.prototype.strike
    pub string_strike: Handle<Object>,
    /// Represents String.prototype.sub
    pub string_sub: Handle<Object>,
    /// Represents String.prototype.sup
    pub string_sup: Handle<Object>,
    /// Represents String.prototype.includes
    pub string_includes: Handle<Object>,
    /// Represents String.prototype.indexOf
    pub string_index_of: Handle<Object>,
    /// Represents String.prototype.padStart
    pub string_pad_start: Handle<Object>,
    /// Represents String.prototype.padEnd
    pub string_pad_end: Handle<Object>,
    /// Represents String.prototype.repeat
    pub string_repeat: Handle<Object>,
    /// Represents String.prototype.toLowerCase
    pub string_to_lowercase: Handle<Object>,
    /// Represents String.prototype.toUpperCase
    pub string_to_uppercase: Handle<Object>,
    /// Represents String.prototype.replace
    pub string_replace: Handle<Object>,
    /// Represents Math.pow
    pub math_pow: Handle<Object>,
    /// Represents Math.abs
    pub math_abs: Handle<Object>,
    /// Represents Math.ceil
    pub math_ceil: Handle<Object>,
    /// Represents Math.floor
    pub math_floor: Handle<Object>,
    /// Represents Math.max
    pub math_max: Handle<Object>,
    /// Represents Math.random
    pub math_random: Handle<Object>,
    /// Represents Object.defineProperty
    pub object_define_property: Handle<Object>,
    /// Represents Object.getOwnPropertyNames
    pub object_get_own_property_names: Handle<Object>,
    /// Represents Object.getOwnPropertySymbols
    pub object_get_own_property_symbols: Handle<Object>,
    /// Represents Object.getPrototypeOf
    pub object_get_prototype_of: Handle<Object>,
    /// Represents Object.prototype.toString
    pub object_to_string: Handle<Object>,
    /// Represents WeakSet.prototype.has
    pub weakset_has: Handle<Object>,
    /// Represents WeakSet.prototype.add
    pub weakset_add: Handle<Object>,
    /// Represents WeakSet.prototype.delete
    pub weakset_delete: Handle<Object>,
    /// Represents WeakMap.prototype.has
    pub weakmap_has: Handle<Object>,
    /// Represents WeakMap.prototype.add
    pub weakmap_add: Handle<Object>,
    /// Represents WeakMap.prototype.get
    pub weakmap_get: Handle<Object>,
    /// Represents WeakMap.prototype.delete
    pub weakmap_delete: Handle<Object>,
    /// Represents JSON.parse
    pub json_parse: Handle<Object>,
    /// Represents JSON.stringify
    pub json_stringify: Handle<Object>,
    /// Represents Promise.resolve
    pub promise_resolve: Handle<Object>,
    /// Represents Promise.reject
    pub promise_reject: Handle<Object>,

    /// Represents Symbol.asyncIterator
    pub symbol_async_iterator: Handle<Object>,
    /// Represents Symbol.hasInstance
    pub symbol_has_instance: Handle<Object>,
    /// Represents Symbol.isConcatSpreadable
    pub symbol_is_concat_spreadable: Handle<Object>,
    /// Represents Symbol.iterator
    pub symbol_iterator: Handle<Object>,
    /// Represents Symbol.match
    pub symbol_match: Handle<Object>,
    /// Represents Symbol.matchAll
    pub symbol_match_all: Handle<Object>,
    /// Represents Symbol.replace
    pub symbol_replace: Handle<Object>,
    /// Represents Symbol.search
    pub symbol_search: Handle<Object>,
    /// Represents Symbol.species
    pub symbol_species: Handle<Object>,
    /// Represents Symbol.split
    pub symbol_split: Handle<Object>,
    /// Represents Symbol.toPrimitive
    pub symbol_to_primitive: Handle<Object>,
    /// Represents Symbol.toStringTag
    pub symbol_to_string_tag: Handle<Object>,
    /// Represents Symbol.unscopables
    pub symbol_unscopables: Handle<Object>,
    /// Represents the identity function
    ///
    /// It is used for functions that return its `this` argument
    /// For example GeneratorIterator[Symbol.iterator]
    pub identity: Handle<Object>,
}

fn register_function(
    gc: &mut Gc<Object>,
    name: &'static str,
    func: NativeFunctionCallback,
    constructor: Constructor,
) -> Handle<Object> {
    gc.register(Object::from(NativeFunction::new(
        name,
        func,
        None,
        constructor,
    )))
}

fn register_function_no_ctor(
    gc: &mut Gc<Object>,
    name: &'static str,
    func: NativeFunctionCallback,
) -> Handle<Object> {
    register_function(gc, name, func, Constructor::NoCtor)
}

fn register_function_ctor(
    gc: &mut Gc<Object>,
    name: &'static str,
    func: NativeFunctionCallback,
) -> Handle<Object> {
    register_function(gc, name, func, Constructor::Ctor)
}

fn register_symbol(gc: &mut Gc<Object>, name: &str) -> Handle<Object> {
    gc.register(Object::from(Symbol(Some(name.into()))))
}

impl Statics {
    /// Creates a new global data object
    #[rustfmt::skip]
    pub fn new(gc: &mut Gc<Object>) -> Self {
        Self {
            // Proto
            boolean_proto: gc.register(Object::from(ObjectKind::Ordinary)),
            number_proto: gc.register(Object::from(ObjectKind::Ordinary)),
            string_proto: gc.register(Object::from(ObjectKind::Ordinary)),
            function_proto: gc.register(Object::from(ObjectKind::Ordinary)),
            array_proto: gc.register(Object::from(ObjectKind::Ordinary)),
            weakset_proto: gc.register(Object::from(ObjectKind::Ordinary)),
            weakmap_proto: gc.register(Object::from(ObjectKind::Ordinary)),
            object_proto: gc.register(Object::from(ObjectKind::Ordinary)),
            error_proto: gc.register(Object::from(ObjectKind::Ordinary)),
            promise_proto: gc.register(Object::from(ObjectKind::Ordinary)),
            symbol_proto: gc.register(Object::from(ObjectKind::Ordinary)),
            generator_iterator_proto: gc.register(Object::from(ObjectKind::Ordinary)),
            // Ctor
            error_ctor: register_function_ctor(gc, "Error", js_std::error::error_constructor),
            weakset_ctor: register_function_ctor(gc, "WeakSet", js_std::weakset::weakset_constructor),
            weakmap_ctor: register_function_ctor(gc, "WeakMap", js_std::weakmap::weakmap_constructor),
            boolean_ctor: register_function_ctor(gc, "Boolean", js_std::boolean::boolean_constructor),
            number_ctor: register_function_ctor(gc, "Number", js_std::number::number_constructor),
            string_ctor: register_function_ctor(gc, "String", js_std::string::string_constructor),
            function_ctor: register_function_ctor(gc, "Function", js_std::function::function_constructor),
            array_ctor: register_function_ctor(gc, "Array", js_std::array::array_constructor),
            object_ctor: register_function_ctor(gc, "Object", js_std::object::object_constructor),
            promise_ctor: register_function_ctor(gc, "Promise", js_std::promise::promise_constructor),
            symbol_ctor: register_function_ctor(gc, "Symbol", js_std::symbol::symbol_constructor),
            symbol_for: register_function_ctor(gc, "for", js_std::symbol::symbol_for),
            symbol_key_for: register_function_ctor(gc, "keyFor", js_std::symbol::symbol_key_for),
            // Methods
            console_log: register_function_no_ctor(gc, "log", js_std::console::log),
            isnan: register_function_no_ctor(gc, "isNaN", js_std::functions::is_nan),
            generator_iterator_next: register_function_no_ctor(gc, "next", js_std::generator::next),
            generator_iterator_return: register_function_no_ctor(
                gc,
                "return",
                js_std::generator::return_
            ),
            array_push: register_function_no_ctor(gc, "push", js_std::array::push),
            array_concat: register_function_no_ctor(gc, "concat", js_std::array::concat),
            array_map: register_function_no_ctor(gc, "map", js_std::array::map),
            array_every: register_function_no_ctor(gc, "every", js_std::array::every),
            array_fill: register_function_no_ctor(gc, "fill", js_std::array::fill),
            array_filter: register_function_no_ctor(gc, "filter", js_std::array::filter),
            array_find: register_function_no_ctor(gc, "find", js_std::array::find),
            array_find_index: register_function_no_ctor(gc, "findIndex", js_std::array::find_index),
            array_flat: register_function_no_ctor(gc, "flat", js_std::array::flat),
            array_for_each: register_function_no_ctor(gc, "forEach", js_std::array::for_each),
            array_from: register_function_no_ctor(gc, "from", js_std::array::from),
            array_includes: register_function_no_ctor(gc, "includes", js_std::array::includes),
            array_index_of: register_function_no_ctor(gc, "indexOf", js_std::array::index_of),
            array_is_array: register_function_no_ctor(gc, "isArray", js_std::array::is_array),
            array_join: register_function_no_ctor(gc, "join", js_std::array::join),
            array_last_index_of: register_function_no_ctor(
                gc,
                "lastIndexOf",
                js_std::array::last_index_of
            ),
            array_of: register_function_no_ctor(gc, "of", js_std::array::of),
            array_pop: register_function_no_ctor(gc, "pop", js_std::array::pop),
            array_reduce: register_function_no_ctor(gc, "reduce", js_std::array::reduce),
            array_reduce_right: register_function_no_ctor(
                gc,
                "reduceRight",
                js_std::array::reduce_right
            ),
            array_reverse: register_function_no_ctor(gc, "reverse", js_std::array::reverse),
            array_shift: register_function_no_ctor(gc, "shift", js_std::array::shift),
            array_slice: register_function_no_ctor(gc, "slice", js_std::array::slice),
            array_some: register_function_no_ctor(gc, "some", js_std::array::some),
            array_sort: register_function_no_ctor(gc, "sort", js_std::array::sort),
            array_splice: register_function_no_ctor(gc, "splice", js_std::array::splice),
            array_unshift: register_function_no_ctor(gc, "unshift", js_std::array::unshift),
            string_char_at: register_function_no_ctor(gc, "charAt", js_std::string::char_at),
            string_char_code_at: register_function_no_ctor(
                gc,
                "charCodeAt",
                js_std::string::char_code_at
            ),
            string_ends_with: register_function_no_ctor(gc, "endsWith", js_std::string::ends_with),
            string_anchor: register_function_no_ctor(gc, "anchor", js_std::string::anchor),
            string_big: register_function_no_ctor(gc, "big", js_std::string::big),
            string_blink: register_function_no_ctor(gc, "blink", js_std::string::blink),
            string_bold: register_function_no_ctor(gc, "bold", js_std::string::bold),
            string_fixed: register_function_no_ctor(gc, "fixed", js_std::string::fixed),
            string_fontcolor: register_function_no_ctor(gc, "fontcolor", js_std::string::fontcolor),
            string_fontsize: register_function_no_ctor(gc, "fontsize", js_std::string::fontsize),
            string_italics: register_function_no_ctor(gc, "italics", js_std::string::italics),
            string_link: register_function_no_ctor(gc, "link", js_std::string::link),
            string_small: register_function_no_ctor(gc, "small", js_std::string::small),
            string_strike: register_function_no_ctor(gc, "strike", js_std::string::strike),
            string_sub: register_function_no_ctor(gc, "sub", js_std::string::sub),
            string_sup: register_function_no_ctor(gc, "sup", js_std::string::sup),
            string_includes: register_function_no_ctor(gc, "includes", js_std::string::includes),
            string_index_of: register_function_no_ctor(gc, "indexOf", js_std::string::index_of),
            string_pad_start: register_function_no_ctor(gc, "padStart", js_std::string::pad_start),
            string_pad_end: register_function_no_ctor(gc, "padEnd", js_std::string::pad_end),
            string_repeat: register_function_no_ctor(gc, "repeat", js_std::string::repeat),
            string_to_lowercase: register_function_no_ctor(
                gc,
                "toLowerCase",
                js_std::string::to_lowercase
            ),
            string_to_uppercase: register_function_no_ctor(
                gc,
                "toUpperCase",
                js_std::string::to_uppercase
            ),
            string_replace: register_function_no_ctor(gc, "replace", js_std::string::replace),
            math_pow: register_function_no_ctor(gc, "pow", js_std::math::pow),
            math_abs: register_function_no_ctor(gc, "abs", js_std::math::abs),
            math_ceil: register_function_no_ctor(gc, "ceil", js_std::math::ceil),
            math_floor: register_function_no_ctor(gc, "floor", js_std::math::floor),
            math_max: register_function_no_ctor(gc, "max", js_std::math::max),
            math_random: register_function_no_ctor(gc, "random", js_std::math::random),
            object_define_property: register_function_no_ctor(
                gc,
                "defineProperty",
                js_std::object::define_property
            ),
            object_get_own_property_names: register_function_no_ctor(
                gc,
                "getOwnPropertyNames",
                js_std::object::get_own_property_names
            ),
            object_get_own_property_symbols: register_function_no_ctor(
                gc,
                "getOwnPropertySymbols",
                js_std::object::get_own_property_symbols
            ),
            object_get_prototype_of: register_function_no_ctor(
                gc,
                "getPrototypeOf",
                js_std::object::get_prototype_of
            ),
            object_to_string: register_function_no_ctor(gc, "toString", js_std::object::to_string),
            weakset_has: register_function_no_ctor(gc, "has", js_std::weakset::has),
            weakset_add: register_function_no_ctor(gc, "add", js_std::weakset::add),
            weakset_delete: register_function_no_ctor(gc, "delete", js_std::weakset::delete),
            weakmap_has: register_function_no_ctor(gc, "has", js_std::weakmap::has),
            weakmap_add: register_function_no_ctor(gc, "add", js_std::weakmap::add),
            weakmap_get: register_function_no_ctor(gc, "get", js_std::weakmap::get),
            weakmap_delete: register_function_no_ctor(gc, "delete", js_std::weakmap::delete),
            json_parse: register_function_no_ctor(gc, "parse", js_std::json::parse),
            json_stringify: register_function_no_ctor(gc, "stringify", js_std::json::stringify),
            promise_resolve: register_function_no_ctor(gc, "resolve", js_std::promise::resolve),
            promise_reject: register_function_no_ctor(gc, "reject", js_std::promise::reject),
            // Well known symbols
            symbol_iterator: register_symbol(gc, "iterator"),
            symbol_async_iterator: register_symbol(gc, "asyncIterator"),
            symbol_has_instance: register_symbol(gc, "hasInstance"),
            symbol_is_concat_spreadable: register_symbol(gc, "isConcatSpreadable"),
            symbol_match: register_symbol(gc, "match"),
            symbol_match_all: register_symbol(gc, "matchAll"),
            symbol_replace: register_symbol(gc, "replace"),
            symbol_search: register_symbol(gc, "search"),
            symbol_species: register_symbol(gc, "species"),
            symbol_split: register_symbol(gc, "split"),
            symbol_to_primitive: register_symbol(gc, "toPrimitive"),
            symbol_to_string_tag: register_symbol(gc, "toStringTag"),
            symbol_unscopables: register_symbol(gc, "unscopables"),
            // Other
            identity: register_function_no_ctor(gc, "identity", js_std::identity),
        }
    }
}
