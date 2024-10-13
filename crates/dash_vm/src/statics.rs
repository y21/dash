use dash_proc_macro::Trace;

use crate::gc::gc2::Allocator;
use crate::gc::interner::{self, sym};
use crate::gc::ObjectId;
use crate::js_std;
use crate::value::error::{AggregateError, EvalError, RangeError, ReferenceError, SyntaxError, TypeError, URIError};
use crate::value::function::{Function, FunctionKind};
use crate::value::map::Map;
use crate::value::regex::RegExp;
use crate::value::set::Set;
use crate::value::PureBuiltin;

use super::value::array::{Array, ArrayIterator};
use super::value::arraybuffer::ArrayBuffer;
use super::value::boxed::{
    Boolean as BoxedBoolean, Number as BoxedNumber, String as BoxedString, Symbol as BoxedSymbol,
};
use super::value::error::Error;
use super::value::function::generator::GeneratorIterator;
use super::value::function::native::NativeFunction;
use super::value::object::{NamedObject, Object};
use super::value::primitive::Symbol;

#[derive(Trace)]
pub struct Statics {
    pub function_proto: ObjectId,
    pub function_ctor: ObjectId,
    pub function_apply: ObjectId,
    pub function_bind: ObjectId,
    pub function_call: ObjectId,
    pub function_to_string: ObjectId,
    pub is_nan: ObjectId,
    pub eval: ObjectId,
    pub is_finite: ObjectId,
    pub parse_float: ObjectId,
    pub parse_int: ObjectId,
    pub console: ObjectId,
    pub console_log: ObjectId,
    pub math: ObjectId,
    pub math_floor: ObjectId,
    pub math_abs: ObjectId,
    pub math_acos: ObjectId,
    pub math_acosh: ObjectId,
    pub math_asin: ObjectId,
    pub math_asinh: ObjectId,
    pub math_atan: ObjectId,
    pub math_atanh: ObjectId,
    pub math_atan2: ObjectId,
    pub math_cbrt: ObjectId,
    pub math_ceil: ObjectId,
    pub math_clz32: ObjectId,
    pub math_cos: ObjectId,
    pub math_cosh: ObjectId,
    pub math_exp: ObjectId,
    pub math_expm1: ObjectId,
    pub math_log: ObjectId,
    pub math_log1p: ObjectId,
    pub math_log10: ObjectId,
    pub math_log2: ObjectId,
    pub math_round: ObjectId,
    pub math_sin: ObjectId,
    pub math_sinh: ObjectId,
    pub math_sqrt: ObjectId,
    pub math_tan: ObjectId,
    pub math_tanh: ObjectId,
    pub math_trunc: ObjectId,
    pub math_random: ObjectId,
    pub math_max: ObjectId,
    pub math_min: ObjectId,
    pub math_pow: ObjectId,
    pub object_ctor: ObjectId,
    pub object_prototype: ObjectId,
    pub object_create: ObjectId,
    pub object_keys: ObjectId,
    pub object_to_string: ObjectId,
    pub object_get_own_property_descriptor: ObjectId,
    pub object_get_own_property_descriptors: ObjectId,
    pub object_has_own_property: ObjectId,
    pub object_define_property: ObjectId,
    pub object_define_properties: ObjectId,
    pub object_assign: ObjectId,
    pub object_entries: ObjectId,
    pub object_get_prototype_of: ObjectId,
    pub object_set_prototype_of: ObjectId,
    pub object_is_prototype_of: ObjectId,
    pub object_property_is_enumerable: ObjectId,
    pub number_ctor: ObjectId,
    pub number_prototype: ObjectId,
    pub number_tostring: ObjectId,
    pub number_valueof: ObjectId,
    pub number_is_finite: ObjectId,
    pub number_is_nan: ObjectId,
    pub number_is_safe_integer: ObjectId,
    pub number_is_integer: ObjectId,
    pub number_to_fixed: ObjectId,
    pub boolean_ctor: ObjectId,
    pub boolean_tostring: ObjectId,
    pub boolean_prototype: ObjectId,
    pub boolean_valueof: ObjectId,
    pub string_ctor: ObjectId,
    pub string_prototype: ObjectId,
    pub string_tostring: ObjectId,
    pub string_char_at: ObjectId,
    pub string_char_code_at: ObjectId,
    pub string_concat: ObjectId,
    pub string_ends_with: ObjectId,
    pub string_starts_with: ObjectId,
    pub string_includes: ObjectId,
    pub string_index_of: ObjectId,
    pub string_last_index_of: ObjectId,
    pub string_pad_end: ObjectId,
    pub string_pad_start: ObjectId,
    pub string_repeat: ObjectId,
    pub string_replace: ObjectId,
    pub string_replace_all: ObjectId,
    pub string_split: ObjectId,
    pub string_to_uppercase: ObjectId,
    pub string_to_lowercase: ObjectId,
    pub string_big: ObjectId,
    pub string_blink: ObjectId,
    pub string_bold: ObjectId,
    pub string_fixed: ObjectId,
    pub string_italics: ObjectId,
    pub string_strike: ObjectId,
    pub string_sub: ObjectId,
    pub string_sup: ObjectId,
    pub string_fontcolor: ObjectId,
    pub string_fontsize: ObjectId,
    pub string_link: ObjectId,
    pub string_trim: ObjectId,
    pub string_trim_start: ObjectId,
    pub string_trim_end: ObjectId,
    pub string_from_char_code: ObjectId,
    pub string_substr: ObjectId,
    pub string_substring: ObjectId,
    pub string_slice: ObjectId,
    pub string_iterator: ObjectId,
    pub array_ctor: ObjectId,
    pub array_tostring: ObjectId,
    pub array_prototype: ObjectId,
    pub array_join: ObjectId,
    pub array_values: ObjectId,
    pub symbol_ctor: ObjectId,
    pub symbol_prototype: ObjectId,
    pub symbol_async_iterator: Symbol,
    pub symbol_has_instance: Symbol,
    pub symbol_is_concat_spreadable: Symbol,
    pub symbol_iterator: Symbol,
    pub symbol_match: Symbol,
    pub symbol_match_all: Symbol,
    pub symbol_replace: Symbol,
    pub symbol_search: Symbol,
    pub symbol_species: Symbol,
    pub symbol_split: Symbol,
    pub symbol_to_primitive: Symbol,
    pub symbol_to_string_tag: Symbol,
    pub symbol_unscopables: Symbol,
    pub array_iterator_prototype: ObjectId,
    pub array_iterator_next: ObjectId,
    pub identity_this: ObjectId,
    pub array_at: ObjectId,
    pub array_concat: ObjectId,
    pub array_entries: ObjectId,
    pub array_keys: ObjectId,
    pub array_every: ObjectId,
    pub array_some: ObjectId,
    pub array_fill: ObjectId,
    pub array_filter: ObjectId,
    pub array_reduce: ObjectId,
    pub array_find: ObjectId,
    pub array_find_index: ObjectId,
    pub array_flat: ObjectId,
    pub array_for_each: ObjectId,
    pub array_includes: ObjectId,
    pub array_index_of: ObjectId,
    pub array_map: ObjectId,
    pub array_pop: ObjectId,
    pub array_push: ObjectId,
    pub array_reverse: ObjectId,
    pub array_shift: ObjectId,
    pub array_sort: ObjectId,
    pub array_unshift: ObjectId,
    pub array_slice: ObjectId,
    pub array_last_index_of: ObjectId,
    pub array_from: ObjectId,
    pub array_is_array: ObjectId,
    pub generator_iterator_prototype: ObjectId,
    pub generator_iterator_next: ObjectId,
    pub error_ctor: ObjectId,
    pub error_prototype: ObjectId,
    pub error_to_string: ObjectId,
    pub eval_error_ctor: ObjectId,
    pub eval_error_prototype: ObjectId,
    pub range_error_ctor: ObjectId,
    pub range_error_prototype: ObjectId,
    pub reference_error_ctor: ObjectId,
    pub reference_error_prototype: ObjectId,
    pub syntax_error_ctor: ObjectId,
    pub syntax_error_prototype: ObjectId,
    pub type_error_ctor: ObjectId,
    pub type_error_prototype: ObjectId,
    pub uri_error_ctor: ObjectId,
    pub uri_error_prototype: ObjectId,
    pub aggregate_error_ctor: ObjectId,
    pub aggregate_error_prototype: ObjectId,
    pub arraybuffer_ctor: ObjectId,
    pub arraybuffer_prototype: ObjectId,
    pub arraybuffer_byte_length: ObjectId,
    pub uint8array_ctor: ObjectId,
    pub uint8array_prototype: ObjectId,
    pub int8array_ctor: ObjectId,
    pub int8array_prototype: ObjectId,
    pub uint16array_ctor: ObjectId,
    pub uint16array_prototype: ObjectId,
    pub int16array_ctor: ObjectId,
    pub int16array_prototype: ObjectId,
    pub uint32array_ctor: ObjectId,
    pub uint32array_prototype: ObjectId,
    pub int32array_ctor: ObjectId,
    pub int32array_prototype: ObjectId,
    pub float32array_ctor: ObjectId,
    pub float32array_prototype: ObjectId,
    pub float64array_ctor: ObjectId,
    pub float64array_prototype: ObjectId,
    pub typedarray_fill: ObjectId,
    pub promise_ctor: ObjectId,
    pub promise_proto: ObjectId,
    pub promise_resolve: ObjectId,
    pub promise_reject: ObjectId,
    pub promise_then: ObjectId,
    pub set_constructor: ObjectId,
    pub set_prototype: ObjectId,
    pub set_add: ObjectId,
    pub set_has: ObjectId,
    pub set_delete: ObjectId,
    pub set_clear: ObjectId,
    pub set_size: ObjectId,
    pub map_constructor: ObjectId,
    pub map_prototype: ObjectId,
    pub map_set: ObjectId,
    pub map_get: ObjectId,
    pub map_has: ObjectId,
    pub map_delete: ObjectId,
    pub map_clear: ObjectId,
    pub map_size: ObjectId,
    pub regexp_ctor: ObjectId,
    pub regexp_prototype: ObjectId,
    pub regexp_test: ObjectId,
    pub regexp_exec: ObjectId,
    pub date_ctor: ObjectId,
    pub date_prototype: ObjectId,
    pub date_now: ObjectId,
    pub json_ctor: ObjectId,
    pub json_parse: ObjectId,
}

fn builtin_object<O: Object + 'static>(gc: &mut Allocator, obj: O) -> ObjectId {
    // gc.register(PureBuiltin::new(obj))
    gc.alloc_object(PureBuiltin::new(obj))
}

fn empty_object(gc: &mut Allocator) -> ObjectId {
    gc.alloc_object(NamedObject::null())
}

fn function(gc: &mut Allocator, name: interner::Symbol, cb: NativeFunction) -> ObjectId {
    let f = Function::with_obj(Some(name.into()), FunctionKind::Native(cb), NamedObject::null());
    gc.alloc_object(PureBuiltin::new(f))
}

impl Statics {
    pub fn new(gc: &mut Allocator) -> Self {
        Self {
            function_proto: empty_object(gc),
            function_ctor: function(gc, sym::Function, js_std::function::constructor),
            function_apply: function(gc, sym::apply, js_std::function::apply),
            function_bind: function(gc, sym::bind, js_std::function::bind),
            function_call: function(gc, sym::call, js_std::function::call),
            function_to_string: function(gc, sym::toString, js_std::function::to_string),
            console: empty_object(gc),
            console_log: function(gc, sym::log, js_std::global::log),
            math: empty_object(gc),
            math_floor: function(gc, sym::floor, js_std::math::floor),
            object_ctor: function(gc, sym::object, js_std::object::constructor),
            object_create: function(gc, sym::create, js_std::object::create),
            object_keys: function(gc, sym::keys, js_std::object::keys),
            object_prototype: empty_object(gc),
            object_to_string: function(gc, sym::toString, js_std::object::to_string),
            object_get_own_property_descriptor: function(
                gc,
                sym::getOwnPropertyDescriptor,
                js_std::object::get_own_property_descriptor,
            ),
            object_get_own_property_descriptors: function(
                gc,
                sym::getOwnPropertyDescriptors,
                js_std::object::get_own_property_descriptors,
            ),
            object_has_own_property: function(gc, sym::hasOwnProperty, js_std::object::has_own_property),
            object_define_property: function(gc, sym::defineProperty, js_std::object::define_property),
            object_define_properties: function(gc, sym::defineProperties, js_std::object::define_properties),
            object_assign: function(gc, sym::assign, js_std::object::assign),
            object_entries: function(gc, sym::entries, js_std::object::entries),
            object_get_prototype_of: function(gc, sym::getPrototypeOf, js_std::object::get_prototype_of),
            object_set_prototype_of: function(gc, sym::setPrototypeOf, js_std::object::set_prototype_of),
            object_is_prototype_of: function(gc, sym::isPrototypeOf, js_std::object::is_prototype_of),
            object_property_is_enumerable: function(
                gc,
                sym::propertyIsEnumerable,
                js_std::object::property_is_enumerable,
            ),
            number_ctor: function(gc, sym::Number, js_std::number::constructor),
            number_prototype: builtin_object(gc, BoxedNumber::with_obj(0.0, NamedObject::null())),
            number_tostring: function(gc, sym::toString, js_std::number::to_string),
            number_valueof: function(gc, sym::valueOf, js_std::number::value_of),
            boolean_ctor: function(gc, sym::Boolean, js_std::boolean::constructor),
            boolean_tostring: function(gc, sym::toString, js_std::boolean::to_string),
            boolean_prototype: builtin_object(gc, BoxedBoolean::with_obj(false, NamedObject::null())),
            string_ctor: function(gc, sym::String, js_std::string::constructor),
            string_prototype: builtin_object(gc, BoxedString::with_obj(sym::empty.into(), NamedObject::null())),
            is_nan: function(gc, sym::isNaN, js_std::global::is_nan),
            eval: function(gc, sym::eval, js_std::global::eval),
            is_finite: function(gc, sym::isFinite, js_std::global::is_finite),
            parse_float: function(gc, sym::parseFloat, js_std::global::parse_float),
            parse_int: function(gc, sym::parseInt, js_std::global::parse_int),
            math_abs: function(gc, sym::abs, js_std::math::abs),
            math_acos: function(gc, sym::acos, js_std::math::acos),
            math_acosh: function(gc, sym::acosh, js_std::math::acosh),
            math_asin: function(gc, sym::asin, js_std::math::asin),
            math_asinh: function(gc, sym::asinh, js_std::math::asinh),
            math_atan: function(gc, sym::atan, js_std::math::atan),
            math_atanh: function(gc, sym::atanh, js_std::math::atanh),
            math_atan2: function(gc, sym::atan2, js_std::math::atan2),
            math_cbrt: function(gc, sym::cbrt, js_std::math::cbrt),
            math_ceil: function(gc, sym::ceil, js_std::math::ceil),
            math_clz32: function(gc, sym::clz32, js_std::math::clz32),
            math_cos: function(gc, sym::cos, js_std::math::cos),
            math_cosh: function(gc, sym::cosh, js_std::math::cosh),
            math_exp: function(gc, sym::exp, js_std::math::exp),
            math_expm1: function(gc, sym::expm1, js_std::math::expm1),
            math_log: function(gc, sym::log, js_std::math::log),
            math_log1p: function(gc, sym::log1p, js_std::math::log1p),
            math_log10: function(gc, sym::log10, js_std::math::log10),
            math_log2: function(gc, sym::log2, js_std::math::log2),
            math_round: function(gc, sym::round, js_std::math::round),
            math_sin: function(gc, sym::sin, js_std::math::sin),
            math_sinh: function(gc, sym::sinh, js_std::math::sinh),
            math_sqrt: function(gc, sym::sqrt, js_std::math::sqrt),
            math_tan: function(gc, sym::tan, js_std::math::tan),
            math_tanh: function(gc, sym::tanh, js_std::math::tanh),
            math_trunc: function(gc, sym::trunc, js_std::math::trunc),
            math_random: function(gc, sym::random, js_std::math::random),
            math_max: function(gc, sym::max, js_std::math::max),
            math_min: function(gc, sym::min, js_std::math::min),
            math_pow: function(gc, sym::pow, js_std::math::pow),
            number_is_finite: function(gc, sym::isFinite, js_std::number::is_finite),
            number_is_nan: function(gc, sym::isNaN, js_std::number::is_nan),
            number_is_safe_integer: function(gc, sym::isSafeInteger, js_std::number::is_safe_integer),
            number_is_integer: function(gc, sym::isInteger, js_std::number::is_integer),
            number_to_fixed: function(gc, sym::toFixed, js_std::number::to_fixed),
            boolean_valueof: function(gc, sym::valueOf, js_std::boolean::value_of),
            string_tostring: function(gc, sym::toString, js_std::string::to_string),
            string_char_at: function(gc, sym::charAt, js_std::string::char_at),
            string_char_code_at: function(gc, sym::charCodeAt, js_std::string::char_code_at),
            string_concat: function(gc, sym::concat, js_std::string::concat),
            string_ends_with: function(gc, sym::endsWith, js_std::string::ends_with),
            string_starts_with: function(gc, sym::startsWith, js_std::string::starts_with),
            string_includes: function(gc, sym::includes, js_std::string::includes),
            string_index_of: function(gc, sym::indexOf, js_std::string::index_of),
            string_last_index_of: function(gc, sym::lastIndexOf, js_std::string::last_index_of),
            string_pad_end: function(gc, sym::padEnd, js_std::string::pad_end),
            string_pad_start: function(gc, sym::padStart, js_std::string::pad_start),
            string_repeat: function(gc, sym::repeat, js_std::string::repeat),
            string_replace: function(gc, sym::replace, js_std::string::replace),
            string_replace_all: function(gc, sym::replaceAll, js_std::string::replace_all),
            string_split: function(gc, sym::split, js_std::string::split),
            string_to_uppercase: function(gc, sym::toUpperCase, js_std::string::to_uppercase),
            string_to_lowercase: function(gc, sym::toLowerCase, js_std::string::to_lowercase),
            string_big: function(gc, sym::big, js_std::string::big),
            string_blink: function(gc, sym::blink, js_std::string::blink),
            string_bold: function(gc, sym::bold, js_std::string::bold),
            string_fixed: function(gc, sym::fixed, js_std::string::fixed),
            string_italics: function(gc, sym::italics, js_std::string::italics),
            string_strike: function(gc, sym::strike, js_std::string::strike),
            string_sub: function(gc, sym::sub, js_std::string::sub),
            string_sup: function(gc, sym::sup, js_std::string::sup),
            string_fontcolor: function(gc, sym::fontcolor, js_std::string::fontcolor),
            string_fontsize: function(gc, sym::fontsize, js_std::string::fontsize),
            string_link: function(gc, sym::link, js_std::string::link),
            string_trim: function(gc, sym::trim, js_std::string::trim),
            string_trim_start: function(gc, sym::trimStart, js_std::string::trim_start),
            string_trim_end: function(gc, sym::trimEnd, js_std::string::trim_end),
            string_from_char_code: function(gc, sym::fromCharCode, js_std::string::from_char_code),
            string_substr: function(gc, sym::substr, js_std::string::substr),
            string_substring: function(gc, sym::substring, js_std::string::substring),
            string_slice: function(gc, sym::slice, js_std::string::slice),
            string_iterator: function(gc, sym::iterator, js_std::string::iterator),
            array_ctor: function(gc, sym::Array, js_std::array::constructor),
            array_tostring: function(gc, sym::toString, js_std::array::to_string),
            array_prototype: builtin_object(gc, Array::with_obj(NamedObject::null())),
            array_join: function(gc, sym::join, js_std::array::join),
            array_values: function(gc, sym::values, js_std::array::values),
            array_reverse: function(gc, sym::reverse, js_std::array::reverse),
            symbol_ctor: function(gc, sym::JsSymbol, js_std::symbol::constructor),
            symbol_prototype: builtin_object(
                gc,
                BoxedSymbol::with_obj(Symbol::new(sym::empty.into()), NamedObject::null()),
            ),
            symbol_async_iterator: Symbol::new(sym::asyncIterator.into()),
            symbol_has_instance: Symbol::new(sym::hasInstance.into()),
            symbol_is_concat_spreadable: Symbol::new(sym::isConcatSpreadable.into()),
            symbol_iterator: Symbol::new(sym::iterator.into()),
            symbol_match: Symbol::new(sym::match_.into()),
            symbol_match_all: Symbol::new(sym::matchAll.into()),
            symbol_replace: Symbol::new(sym::replace.into()),
            symbol_search: Symbol::new(sym::search.into()),
            symbol_species: Symbol::new(sym::species.into()),
            symbol_split: Symbol::new(sym::split.into()),
            symbol_to_primitive: Symbol::new(sym::toPrimitive.into()),
            symbol_to_string_tag: Symbol::new(sym::toStringTag.into()),
            symbol_unscopables: Symbol::new(sym::unscopables.into()),
            array_iterator_prototype: builtin_object(gc, ArrayIterator::empty()),
            array_iterator_next: function(gc, sym::next, js_std::array_iterator::next),
            identity_this: function(gc, sym::iterator, js_std::identity_this),
            array_at: function(gc, sym::at, js_std::array::at),
            array_concat: function(gc, sym::concat, js_std::array::concat),
            array_entries: function(gc, sym::entries, js_std::array::entries),
            array_keys: function(gc, sym::keys, js_std::array::keys),
            array_every: function(gc, sym::every, js_std::array::every),
            array_some: function(gc, sym::some, js_std::array::some),
            array_fill: function(gc, sym::fill, js_std::array::fill),
            array_filter: function(gc, sym::filter, js_std::array::filter),
            array_reduce: function(gc, sym::reduce, js_std::array::reduce),
            array_find: function(gc, sym::find, js_std::array::find),
            array_find_index: function(gc, sym::findIndex, js_std::array::find_index),
            array_flat: function(gc, sym::flat, js_std::array::flat),
            array_for_each: function(gc, sym::forEach, js_std::array::for_each),
            array_includes: function(gc, sym::includes, js_std::array::includes),
            array_index_of: function(gc, sym::indexOf, js_std::array::index_of),
            array_map: function(gc, sym::map, js_std::array::map),
            array_pop: function(gc, sym::pop, js_std::array::pop),
            array_push: function(gc, sym::push, js_std::array::push),
            array_shift: function(gc, sym::shift, js_std::array::shift),
            array_sort: function(gc, sym::sort, js_std::array::sort),
            array_unshift: function(gc, sym::unshift, js_std::array::unshift),
            array_slice: function(gc, sym::slice, js_std::array::slice),
            array_last_index_of: function(gc, sym::lastIndexOf, js_std::array::last_index_of),
            array_from: function(gc, sym::from, js_std::array::from),
            array_is_array: function(gc, sym::isArray, js_std::array::is_array),
            generator_iterator_prototype: {
                let obj: ObjectId = empty_object(gc);
                builtin_object(gc, GeneratorIterator::empty(obj))
            },
            generator_iterator_next: function(gc, sym::next, js_std::generator::next),
            error_ctor: function(gc, sym::Error, js_std::error::error_constructor),
            error_prototype: builtin_object(gc, Error::empty()),
            error_to_string: function(gc, sym::toString, js_std::error::to_string),
            eval_error_ctor: function(gc, sym::EvalError, js_std::error::eval_error_constructor),
            eval_error_prototype: builtin_object(gc, EvalError::empty()),
            range_error_ctor: function(gc, sym::RangeError, js_std::error::range_error_constructor),
            range_error_prototype: builtin_object(gc, RangeError::empty()),
            reference_error_ctor: function(gc, sym::ReferenceError, js_std::error::reference_error_constructor),
            reference_error_prototype: builtin_object(gc, ReferenceError::empty()),
            syntax_error_ctor: function(gc, sym::SyntaxError, js_std::error::syntax_error_constructor),
            syntax_error_prototype: builtin_object(gc, SyntaxError::empty()),
            type_error_ctor: function(gc, sym::TypeError, js_std::error::type_error_constructor),
            type_error_prototype: builtin_object(gc, TypeError::empty()),
            uri_error_ctor: function(gc, sym::URIError, js_std::error::uri_error_constructor),
            uri_error_prototype: builtin_object(gc, URIError::empty()),
            aggregate_error_ctor: function(gc, sym::AggregateError, js_std::error::aggregate_error_constructor),
            aggregate_error_prototype: builtin_object(gc, AggregateError::empty()),
            arraybuffer_ctor: function(gc, sym::ArrayBuffer, js_std::arraybuffer::constructor),
            arraybuffer_prototype: builtin_object(gc, ArrayBuffer::empty()),
            arraybuffer_byte_length: function(gc, sym::byteLength, js_std::arraybuffer::byte_length),
            uint8array_ctor: function(gc, sym::Uint8Array, js_std::typedarray::u8array::constructor),
            uint8array_prototype: empty_object(gc),
            int8array_ctor: function(gc, sym::Int8Array, js_std::typedarray::i8array::constructor),
            int8array_prototype: empty_object(gc),
            uint16array_ctor: function(gc, sym::Uint16Array, js_std::typedarray::u16array::constructor),
            uint16array_prototype: empty_object(gc),
            int16array_ctor: function(gc, sym::Int16Array, js_std::typedarray::i16array::constructor),
            int16array_prototype: empty_object(gc),
            uint32array_ctor: function(gc, sym::Uint32Array, js_std::typedarray::u32array::constructor),
            uint32array_prototype: empty_object(gc),
            int32array_ctor: function(gc, sym::Int32Array, js_std::typedarray::i32array::constructor),
            int32array_prototype: empty_object(gc),
            float32array_ctor: function(gc, sym::Float32Array, js_std::typedarray::f32array::constructor),
            float32array_prototype: empty_object(gc),
            float64array_ctor: function(gc, sym::Float64Array, js_std::typedarray::f64array::constructor),
            float64array_prototype: empty_object(gc),
            typedarray_fill: function(gc, sym::fill, js_std::typedarray::fill),
            promise_ctor: function(gc, sym::Promise, js_std::promise::constructor),
            promise_proto: empty_object(gc),
            promise_resolve: function(gc, sym::resolve, js_std::promise::resolve),
            promise_reject: function(gc, sym::reject, js_std::promise::reject),
            promise_then: function(gc, sym::then, js_std::promise::then),
            set_constructor: function(gc, sym::Set, js_std::set::constructor),
            set_add: function(gc, sym::add, js_std::set::add),
            set_has: function(gc, sym::has, js_std::set::has),
            set_delete: function(gc, sym::delete, js_std::set::delete),
            set_prototype: builtin_object(gc, Set::with_obj(NamedObject::null())),
            set_clear: function(gc, sym::clear, js_std::set::clear),
            set_size: function(gc, sym::size, js_std::set::size),
            map_constructor: function(gc, sym::Map, js_std::map::constructor),
            map_set: function(gc, sym::set, js_std::map::set),
            map_get: function(gc, sym::get, js_std::map::get),
            map_has: function(gc, sym::has, js_std::map::has),
            map_delete: function(gc, sym::delete, js_std::map::delete),
            map_prototype: builtin_object(gc, Map::with_obj(NamedObject::null())),
            map_clear: function(gc, sym::clear, js_std::map::clear),
            map_size: function(gc, sym::size, js_std::map::size),
            regexp_ctor: function(gc, sym::RegExp, js_std::regex::constructor),
            regexp_prototype: builtin_object(gc, RegExp::empty()),
            regexp_test: function(gc, sym::test, js_std::regex::test),
            regexp_exec: function(gc, sym::exec, js_std::regex::exec),
            date_ctor: function(gc, sym::Date, js_std::date::constructor),
            date_prototype: builtin_object(gc, NamedObject::null()),
            date_now: function(gc, sym::now, js_std::date::now),
            json_ctor: function(gc, sym::JSON, js_std::json::constructor),
            json_parse: function(gc, sym::parse, js_std::json::parse),
        }
    }
}
