use std::{cell::RefCell, rc::Rc};

use crate::js_std;
use crate::vm::value::function::Constructor;
use crate::vm::value::object::AnyObject;

use super::value::function::NativeFunction;
use super::value::Value;

/// Static data used by the VM
pub struct Statics {
    /// Represents Boolean.prototype
    pub boolean_proto: Rc<RefCell<Value>>,
    /// Represents Number.prototype
    pub number_proto: Rc<RefCell<Value>>,
    /// Represents String.prototype
    pub string_proto: Rc<RefCell<Value>>,
    /// Represents Function.prototype
    pub function_proto: Rc<RefCell<Value>>,
    /// Represents Array.prototype
    pub array_proto: Rc<RefCell<Value>>,
    /// Represents WeakSet.prototype
    pub weakset_proto: Rc<RefCell<Value>>,
    /// Represents WeakMap.prototype
    pub weakmap_proto: Rc<RefCell<Value>>,
    /// Represents Object.prototype
    pub object_proto: Rc<RefCell<Value>>,
    /// Represents Error.prototype
    pub error_proto: Rc<RefCell<Value>>,
    /// Represents the Boolean constructor
    pub boolean_ctor: Rc<RefCell<Value>>,
    /// Represents the Number constructor
    pub number_ctor: Rc<RefCell<Value>>,
    /// Represents the String constructor
    pub string_ctor: Rc<RefCell<Value>>,
    /// Represents the Function constructor
    pub function_ctor: Rc<RefCell<Value>>,
    /// Represents the Array constructor
    pub array_ctor: Rc<RefCell<Value>>,
    /// Represents the WeakSet constructor
    pub weakset_ctor: Rc<RefCell<Value>>,
    /// Represents the WeakMap constructor
    pub weakmap_ctor: Rc<RefCell<Value>>,
    /// Represents the Object constructor
    pub object_ctor: Rc<RefCell<Value>>,
    /// Represents the Error constructor
    pub error_ctor: Rc<RefCell<Value>>,
    /// Represents console.log
    pub console_log: Rc<RefCell<Value>>,
    /// Represents isNaN
    pub isnan: Rc<RefCell<Value>>,
    /// Represents Array.prototype.push
    pub array_push: Rc<RefCell<Value>>,
    /// Represents Array.prototype.concat
    pub array_concat: Rc<RefCell<Value>>,
    /// Represents Array.prototype.map
    pub array_map: Rc<RefCell<Value>>,
    /// Represents Array.prototype.every
    pub array_every: Rc<RefCell<Value>>,
    /// Represents Array.prototype.fill
    pub array_fill: Rc<RefCell<Value>>,
    /// Represents Array.prototype.filter
    pub array_filter: Rc<RefCell<Value>>,
    /// Represents Array.prototype.find
    pub array_find: Rc<RefCell<Value>>,
    /// Represents Array.prototype.findIndex
    pub array_find_index: Rc<RefCell<Value>>,
    /// Represents Array.prototype.flat
    pub array_flat: Rc<RefCell<Value>>,
    /// Represents Array.prototype.forEach
    pub array_for_each: Rc<RefCell<Value>>,
    /// Represents Array.from
    pub array_from: Rc<RefCell<Value>>,
    /// Represents Array.prototype.includes
    pub array_includes: Rc<RefCell<Value>>,
    /// Represents Array.prototype.indexOf
    pub array_index_of: Rc<RefCell<Value>>,
    /// Represents Array.isArray
    pub array_is_array: Rc<RefCell<Value>>,
    /// Represents Array.prototype.join
    pub array_join: Rc<RefCell<Value>>,
    /// Represents Array.prototype.lastIndexOf
    pub array_last_index_of: Rc<RefCell<Value>>,
    /// Represents Array.of
    pub array_of: Rc<RefCell<Value>>,
    /// Represents Array.prototype.pop
    pub array_pop: Rc<RefCell<Value>>,
    /// Represents Array.prototype.reduce
    pub array_reduce: Rc<RefCell<Value>>,
    /// Represents Array.prototype.reduceRight
    pub array_reduce_right: Rc<RefCell<Value>>,
    /// Represents Array.prototype.reverse
    pub array_reverse: Rc<RefCell<Value>>,
    /// Represents Array.prototype.shift
    pub array_shift: Rc<RefCell<Value>>,
    /// Represents Array.prototype.slice
    pub array_slice: Rc<RefCell<Value>>,
    /// Represents Array.prototype.some
    pub array_some: Rc<RefCell<Value>>,
    /// Represents Array.prototype.sort
    pub array_sort: Rc<RefCell<Value>>,
    /// Represents Array.prototype.splice
    pub array_splice: Rc<RefCell<Value>>,
    /// Represents Array.prototype.unshift
    pub array_unshift: Rc<RefCell<Value>>,
    /// Represents String.prototype.charAt
    pub string_char_at: Rc<RefCell<Value>>,
    /// Represents String.prototype.charCodeAt
    pub string_char_code_at: Rc<RefCell<Value>>,
    /// Represents String.prototype.endsWith
    pub string_ends_with: Rc<RefCell<Value>>,
    /// Represents String.prototype.anchor
    pub string_anchor: Rc<RefCell<Value>>,
    /// Represents String.prototype.big
    pub string_big: Rc<RefCell<Value>>,
    /// Represents String.prototype.blink
    pub string_blink: Rc<RefCell<Value>>,
    /// Represents String.prototype.bold
    pub string_bold: Rc<RefCell<Value>>,
    /// Represents String.prototype.fixed
    pub string_fixed: Rc<RefCell<Value>>,
    /// Represents String.prototype.fontcolor
    pub string_fontcolor: Rc<RefCell<Value>>,
    /// Represents String.prototype.fontsize
    pub string_fontsize: Rc<RefCell<Value>>,
    /// Represents String.prototype.italics
    pub string_italics: Rc<RefCell<Value>>,
    /// Represents String.prototype.link
    pub string_link: Rc<RefCell<Value>>,
    /// Represents String.prototype.small
    pub string_small: Rc<RefCell<Value>>,
    /// Represents String.prototype.strike
    pub string_strike: Rc<RefCell<Value>>,
    /// Represents String.prototype.sub
    pub string_sub: Rc<RefCell<Value>>,
    /// Represents String.prototype.sup
    pub string_sup: Rc<RefCell<Value>>,
    /// Represents Math.pow
    pub math_pow: Rc<RefCell<Value>>,
    /// Represents Math.abs
    pub math_abs: Rc<RefCell<Value>>,
    /// Represents Math.ceil
    pub math_ceil: Rc<RefCell<Value>>,
    /// Represents Math.floor
    pub math_floor: Rc<RefCell<Value>>,
    /// Represents Math.max
    pub math_max: Rc<RefCell<Value>>,
    /// Represents Math.random
    pub math_random: Rc<RefCell<Value>>,
    /// Represents Object.defineProperty
    pub object_define_property: Rc<RefCell<Value>>,
    /// Represents Object.getOwnPropertyNames
    pub object_get_own_property_names: Rc<RefCell<Value>>,
    /// Represents Object.getPrototypeOf
    pub object_get_prototype_of: Rc<RefCell<Value>>,
    /// Represents Object.prototype.toString
    pub object_to_string: Rc<RefCell<Value>>,
    /// Represents WeakSet.prototype.has
    pub weakset_has: Rc<RefCell<Value>>,
    /// Represents WeakSet.prototype.add
    pub weakset_add: Rc<RefCell<Value>>,
    /// Represents WeakSet.prototype.delete
    pub weakset_delete: Rc<RefCell<Value>>,
    /// Represents WeakMap.prototype.has
    pub weakmap_has: Rc<RefCell<Value>>,
    /// Represents WeakMap.prototype.add
    pub weakmap_add: Rc<RefCell<Value>>,
    /// Represents WeakMap.prototype.get
    pub weakmap_get: Rc<RefCell<Value>>,
    /// Represents WeakMap.prototype.delete
    pub weakmap_delete: Rc<RefCell<Value>>,
    /// Represents JSON.parse
    pub json_parse: Rc<RefCell<Value>>,
    /// Represents JSON.stringify
    pub json_stringify: Rc<RefCell<Value>>,
}

macro_rules! register_glob_method {
    ($name:expr, $path:expr) => {
        Value::from(NativeFunction::new($name, $path, None, Constructor::NoCtor)).into()
    };
}

macro_rules! register_ctor {
    ($name:expr, $path:expr) => {
        Value::from(NativeFunction::new($name, $path, None, Constructor::Ctor)).into()
    };
}

impl Statics {
    /// Creates a new global data object
    pub fn new() -> Self {
        Self {
            // Proto
            boolean_proto: Value::from(AnyObject {}).into(),
            number_proto: Value::from(AnyObject {}).into(),
            string_proto: Value::from(AnyObject {}).into(),
            function_proto: Value::from(AnyObject {}).into(),
            array_proto: Value::from(AnyObject {}).into(),
            weakset_proto: Value::from(AnyObject {}).into(),
            weakmap_proto: Value::from(AnyObject {}).into(),
            object_proto: Value::from(AnyObject {}).into(),
            error_proto: Value::from(AnyObject {}).into(),
            // Ctor
            error_ctor: register_ctor!("Error", js_std::error::error_constructor),
            weakset_ctor: register_ctor!("WeakSet", js_std::weakset::weakset_constructor),
            weakmap_ctor: register_ctor!("WeakMap", js_std::weakmap::weakmap_constructor),
            boolean_ctor: register_ctor!("Boolean", js_std::boolean::boolean_constructor),
            number_ctor: register_ctor!("Number", js_std::number::number_constructor),
            string_ctor: register_ctor!("String", js_std::string::string_constructor),
            function_ctor: register_ctor!("Function", js_std::function::function_constructor),
            array_ctor: register_ctor!("Array", js_std::array::array_constructor),
            object_ctor: register_ctor!("Object", js_std::object::object_constructor),
            // Methods
            console_log: register_glob_method!("log", js_std::console::log),
            isnan: register_glob_method!("isNaN", js_std::functions::is_nan),
            array_push: register_glob_method!("push", js_std::array::push),
            array_concat: register_glob_method!("concat", js_std::array::concat),
            array_map: register_glob_method!("map", js_std::array::map),
            array_every: register_glob_method!("every", js_std::array::every),
            array_fill: register_glob_method!("fill", js_std::array::fill),
            array_filter: register_glob_method!("filter", js_std::array::filter),
            array_find: register_glob_method!("find", js_std::array::find),
            array_find_index: register_glob_method!("findIndex", js_std::array::find_index),
            array_flat: register_glob_method!("flat", js_std::array::flat),
            array_for_each: register_glob_method!("forEach", js_std::array::for_each),
            array_from: register_glob_method!("from", js_std::array::from),
            array_includes: register_glob_method!("includes", js_std::array::includes),
            array_index_of: register_glob_method!("indexOf", js_std::array::index_of),
            array_is_array: register_glob_method!("isArray", js_std::array::is_array),
            array_join: register_glob_method!("join", js_std::array::join),
            array_last_index_of: register_glob_method!("lastIndexOf", js_std::array::last_index_of),
            array_of: register_glob_method!("of", js_std::array::of),
            array_pop: register_glob_method!("pop", js_std::array::pop),
            array_reduce: register_glob_method!("reduce", js_std::array::reduce),
            array_reduce_right: register_glob_method!("reduceRight", js_std::array::reduce_right),
            array_reverse: register_glob_method!("reverse", js_std::array::reverse),
            array_shift: register_glob_method!("shift", js_std::array::shift),
            array_slice: register_glob_method!("slice", js_std::array::slice),
            array_some: register_glob_method!("some", js_std::array::some),
            array_sort: register_glob_method!("sort", js_std::array::sort),
            array_splice: register_glob_method!("splice", js_std::array::splice),
            array_unshift: register_glob_method!("unshift", js_std::array::unshift),
            string_char_at: register_glob_method!("charAt", js_std::string::char_at),
            string_char_code_at: register_glob_method!("charCodeAt", js_std::string::char_code_at),
            string_ends_with: register_glob_method!("endsWith", js_std::string::ends_with),
            string_anchor: register_glob_method!("anchor", js_std::string::anchor),
            string_big: register_glob_method!("big", js_std::string::big),
            string_blink: register_glob_method!("blink", js_std::string::blink),
            string_bold: register_glob_method!("bold", js_std::string::bold),
            string_fixed: register_glob_method!("fixed", js_std::string::fixed),
            string_fontcolor: register_glob_method!("fontcolor", js_std::string::fontcolor),
            string_fontsize: register_glob_method!("fontsize", js_std::string::fontsize),
            string_italics: register_glob_method!("italics", js_std::string::italics),
            string_link: register_glob_method!("link", js_std::string::link),
            string_small: register_glob_method!("small", js_std::string::small),
            string_strike: register_glob_method!("strike", js_std::string::strike),
            string_sub: register_glob_method!("sub", js_std::string::sub),
            string_sup: register_glob_method!("sup", js_std::string::sup),
            math_pow: register_glob_method!("pow", js_std::math::pow),
            math_abs: register_glob_method!("abs", js_std::math::abs),
            math_ceil: register_glob_method!("ceil", js_std::math::ceil),
            math_floor: register_glob_method!("floor", js_std::math::floor),
            math_max: register_glob_method!("max", js_std::math::max),
            math_random: register_glob_method!("random", js_std::math::random),
            object_define_property: register_glob_method!(
                "defineProperty",
                js_std::object::define_property
            ),
            object_get_own_property_names: register_glob_method!(
                "getOwnPropertyNames",
                js_std::object::get_own_property_names
            ),
            object_get_prototype_of: register_glob_method!(
                "getPrototypeOf",
                js_std::object::get_prototype_of
            ),
            object_to_string: register_glob_method!("toString", js_std::object::to_string),
            weakset_has: register_glob_method!("has", js_std::weakset::has),
            weakset_add: register_glob_method!("add", js_std::weakset::add),
            weakset_delete: register_glob_method!("delete", js_std::weakset::delete),
            weakmap_has: register_glob_method!("has", js_std::weakmap::has),
            weakmap_add: register_glob_method!("add", js_std::weakmap::add),
            weakmap_get: register_glob_method!("get", js_std::weakmap::get),
            weakmap_delete: register_glob_method!("delete", js_std::weakmap::delete),
            json_parse: register_glob_method!("parse", js_std::json::parse),
            json_stringify: register_glob_method!("stringify", js_std::json::stringify),
        }
    }
}
