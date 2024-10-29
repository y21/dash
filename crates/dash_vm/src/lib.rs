#![cfg_attr(dash_lints, feature(register_tool))]
#![cfg_attr(dash_lints, register_tool(dash_lints))]
#![allow(clippy::clone_on_copy)] // TODO: Requires large amounts of changes
#![warn(clippy::redundant_clone)]
#![deny(clippy::disallowed_methods)]

use std::ops::RangeBounds;
use std::vec::Drain;
use std::fmt;

use crate::util::cold_path;
use crate::value::function::Function;
use crate::value::object::{PropertyDataDescriptor, PropertyValueKind};
use crate::value::primitive::Symbol;
use crate::value::Root;

use self::dispatch::HandleResult;
use self::frame::{Exports, Frame, FrameState, TryBlock};
use self::localscope::LocalScope;
use self::params::VmParams;
use self::statics::Statics;
use self::value::object::{Object, PropertyValue};
use self::value::Value;

use dash_log::{debug, error, span, Level};
use dash_middle::compiler::instruction::Instruction;
use dash_middle::interner::{self, sym, StringInterner};
use gc::trace::{Trace, TraceCtxt};
use gc::{Allocator, ObjectId};
use localscope::{scope, LocalScopeList};
use rustc_hash::FxHashMap;
use value::object::NamedObject;
use value::{ExternalValue, PureBuiltin, Unpack, Unrooted, ValueKind};

#[cfg(feature = "jit")]
mod jit;

pub mod dispatch;
pub mod eval;
pub mod frame;
pub mod gc;
pub mod js_std;
pub mod json;
pub mod localscope;
mod macros;
pub mod params;
pub mod statics;
#[cfg(test)]
mod test;
pub mod util;
pub mod value;

pub const MAX_FRAME_STACK_SIZE: usize = 1024;
pub const MAX_STACK_SIZE: usize = 8192;
const DEFAULT_GC_RSS_THRESHOLD: usize = 1024 * 1024;

#[derive(Clone, Default)]
pub struct ExternalRefs(pub std::rc::Rc<std::cell::RefCell<FxHashMap<ObjectId, u32>>>);

pub struct Vm {
    #[cfg_attr(dash_lints, dash_lints::trusted_no_gc)]
    frames: Vec<Frame>,
    async_tasks: Vec<ObjectId>,
    // TODO: the inner vec of the stack should be private for soundness
    // popping from the stack must return `Unrooted`
    stack: Vec<Value>,
    alloc: Allocator,
    pub interner: StringInterner,
    global: ObjectId,
    // "External refs" currently refers to existing `Persistent<T>`s.
    // Persistent values already manage the reference count when cloning or dropping them
    // and are stored in the Handle itself, but we still need to keep track of them so we can
    // consider them as roots and also **trace** them (to reach their children).
    //
    // We insert into this in `Persistent::new`, and remove from it during the tracing phase.
    // We can't do that in Persistent's Drop code, because we don't have access to the VM there.
    external_refs: ExternalRefs,
    scopes: LocalScopeList,
    statics: Box<Statics>,
    #[cfg_attr(dash_lints, dash_lints::trusted_no_gc)]
    try_blocks: Vec<TryBlock>,
    #[cfg_attr(dash_lints, dash_lints::trusted_no_gc)]
    params: VmParams,
    gc_rss_threshold: usize,
    /// Keeps track of the "purity" of the builtins of this VM.
    /// Purity here refers to whether builtins have been (in one way or another) mutated.
    /// Removing a property from the global object (e.g. `Math`) or any other builtin,
    /// or adding a property to a builtin, will cause this to be set to `false`, which in turn
    /// will disable many optimizations such as specialized intrinsics.
    builtins_pure: bool,
    #[cfg(feature = "jit")]
    jit: jit::Frontend,
}

impl Vm {
    pub fn new(params: VmParams) -> Self {
        debug!("create vm");
        let mut alloc = Allocator::new();
        let statics = Statics::new(&mut alloc);
        // TODO: global __proto__ and constructor
        let global: ObjectId = alloc.alloc_object(PureBuiltin::new(NamedObject::null()));
        let gc_rss_threshold = params
            .initial_gc_rss_threshold
            .unwrap_or(DEFAULT_GC_RSS_THRESHOLD);

        let mut vm = Self {
            frames: Vec::new(),
            async_tasks: Vec::new(),
            stack: Vec::with_capacity(512),
            alloc,
            interner: StringInterner::new(),
            global,
            external_refs: ExternalRefs::default(),
            scopes: LocalScopeList::new(),
            statics: Box::new(statics),
            try_blocks: Vec::new(),
            params,
            gc_rss_threshold,
            builtins_pure: true,

            #[cfg(feature = "jit")]
            jit: jit::Frontend::new(),
        };
        vm.prepare();
        vm
    }

    pub fn scope(&mut self) -> LocalScope<'_> {
        scope(self)
    }

    pub fn global(&self) -> ObjectId {
        self.global
    }

    /// Prepare the VM for execution.
    #[rustfmt::skip]
    fn prepare(&mut self) {
        debug!("initialize vm intrinsics");
        fn set_fn_prototype(vm: &Vm, v: &dyn Object, proto: ObjectId, name: interner::Symbol) {
            let fun = v.as_any(vm).downcast_ref::<Function>().unwrap();
            fun.set_name(name.into());
            fun.set_fn_prototype(proto);
        }

        // TODO: we currently recursively call this for each of the registered methods, so a lot of builtins are initialized multiple times
        // we should have some sort of cache to avoid this
        // (though we also populate function prototypes later on this way, so it's not so trivial)
        #[allow(clippy::too_many_arguments)]
        fn register(
            base: ObjectId,
            prototype: impl Into<Value>,
            constructor: ObjectId,
            methods: impl IntoIterator<Item = (interner::Symbol, ObjectId)>,
            symbols: impl IntoIterator<Item = (Symbol, ObjectId)>,
            fields: impl IntoIterator<Item = (interner::Symbol, Value, Option<PropertyDataDescriptor>)>,
            // Contrary to `prototype`, this optionally sets the function prototype. Should only be `Some`
            // when base is a function
            fn_prototype: Option<(interner::Symbol, ObjectId)>,
            // LocalScope needs to be the last parameter because we don't have two phase borrows in user code
            scope: &mut LocalScope<'_>,
        ) -> ObjectId {
            base.set_property(scope, sym::constructor.into(), PropertyValue::static_non_enumerable(constructor.into())).unwrap();
            base.set_prototype(scope, prototype.into()).unwrap();

            for (key, value) in methods {
                register(
                    value.clone(),
                    scope.statics.function_proto.clone(),
                    scope.statics.function_ctor.clone(),
                    [],
                    [],
                    [],
                    None,
                    scope,
                );
                base.set_property(scope, key.into(), PropertyValue::static_non_enumerable(value.into())).unwrap();
            }

            for (key, value) in symbols {
                register(
                    value.clone(),
                    scope.statics.function_proto.clone(),
                    scope.statics.function_ctor.clone(),
                    [],
                    [],
                    [],
                    None,
                    scope,
                );
                base.set_property(scope, key.into(), PropertyValue::static_empty(value.into())).unwrap();
            }

            for (key, value, descriptor) in fields {
                let value = PropertyValue {
                    kind: PropertyValueKind::Static(value),
                    descriptor: descriptor.unwrap_or_default()
                };
                base.set_property(scope, key.into(), value).unwrap();
            }

            if let Some((proto_name, proto_val)) = fn_prototype {
                set_fn_prototype(scope, &base, proto_val, proto_name);
            }

            base
        }

        let mut scope = self.scope();
        let global = scope.global.clone();
        
        let function_ctor = register(
            scope.statics.function_ctor.clone(),
            scope.statics.function_proto.clone(),
            scope.statics.function_ctor.clone(),
            [],
            [],
            [],
            Some((sym::Function, scope.statics.function_proto.clone())),
            &mut scope,
        );
        
        let function_proto = register(
            scope.statics.function_proto.clone(),
            scope.statics.object_prototype.clone(),
            function_ctor.clone(),
            [
                (sym::apply, scope.statics.function_apply.clone()),
                (sym::bind, scope.statics.function_bind.clone()),
                (sym::call, scope.statics.function_call.clone()),
                (sym::toString, scope.statics.function_to_string.clone()),
            ],
            [],
            [],
            None,
            &mut scope,
        );
        
        let object_ctor = register(
            scope.statics.object_ctor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [
                (sym::create, scope.statics.object_create.clone()),
                (sym::keys, scope.statics.object_keys.clone()),
                // FIXME: these are not the same
                (sym::getOwnPropertyNames, scope.statics.object_keys.clone()),
                (sym::getOwnPropertyDescriptor, scope.statics.object_get_own_property_descriptor.clone()),
                (sym::getOwnPropertyDescriptors, scope.statics.object_get_own_property_descriptors.clone()),
                (sym::defineProperty, scope.statics.object_define_property.clone()),
                (sym::defineProperties, scope.statics.object_define_properties.clone()),
                (sym::entries, scope.statics.object_entries.clone()),
                (sym::assign, scope.statics.object_assign.clone()),
                (sym::getPrototypeOf, scope.statics.object_get_prototype_of.clone()),
                (sym::setPrototypeOf, scope.statics.object_set_prototype_of.clone()),
            ],
            [],
            [],
            Some((sym::Object, scope.statics.object_prototype.clone())),
            &mut scope,
        );
        
        let object_proto = register(
            scope.statics.object_prototype.clone(),
            Value::null(),
            object_ctor.clone(),
            [
                (sym::toString, scope.statics.object_to_string.clone()),
                (sym::hasOwnProperty, scope.statics.object_has_own_property.clone()),
                (sym::isPrototypeOf, scope.statics.object_is_prototype_of.clone()),
                (sym::propertyIsEnumerable, scope.statics.object_property_is_enumerable.clone())
            ],
            [],
            [],
            None,
            &mut scope,
        );
        
        let console = register(
            scope.statics.console.clone(),
            object_proto.clone(),
            object_ctor.clone(),
            [
                (sym::log, scope.statics.console_log.clone()),
            ],
            [],
            [],
            None,
            &mut scope,
        );
        
        let math = register(
            scope.statics.math.clone(),
            object_proto.clone(),
            object_ctor.clone(),
            [
                (sym::floor, scope.statics.math_floor.clone()),
                (sym::abs, scope.statics.math_abs.clone()),
                (sym::acos, scope.statics.math_acos.clone()),
                (sym::acosh, scope.statics.math_acosh.clone()),
                (sym::asin, scope.statics.math_asin.clone()),
                (sym::asinh, scope.statics.math_asinh.clone()),
                (sym::atan, scope.statics.math_atan.clone()),
                (sym::atanh, scope.statics.math_atanh.clone()),
                (sym::atan2, scope.statics.math_atan2.clone()),
                (sym::cbrt, scope.statics.math_cbrt.clone()),
                (sym::ceil, scope.statics.math_ceil.clone()),
                (sym::clz32, scope.statics.math_clz32.clone()),
                (sym::cos, scope.statics.math_cos.clone()),
                (sym::cosh, scope.statics.math_cosh.clone()),
                (sym::exp, scope.statics.math_exp.clone()),
                (sym::expm1, scope.statics.math_expm1.clone()),
                (sym::log, scope.statics.math_log.clone()),
                (sym::log1p, scope.statics.math_log1p.clone()),
                (sym::log10, scope.statics.math_log10.clone()),
                (sym::log2, scope.statics.math_log2.clone()),
                (sym::round, scope.statics.math_round.clone()),
                (sym::sin, scope.statics.math_sin.clone()),
                (sym::sinh, scope.statics.math_sinh.clone()),
                (sym::sqrt, scope.statics.math_sqrt.clone()),
                (sym::tan, scope.statics.math_tan.clone()),
                (sym::tanh, scope.statics.math_tanh.clone()),
                (sym::trunc, scope.statics.math_trunc.clone()),
                (sym::random, scope.statics.math_random.clone()),
                (sym::max, scope.statics.math_max.clone()),
                (sym::min, scope.statics.math_min.clone()),
                (sym::pow, scope.statics.math_pow.clone()),
            ],
            [],
            [
                (sym::PI, Value::number(std::f64::consts::PI), Some(PropertyDataDescriptor::empty())),
            ],
            None,
            &mut scope,
        );
        
        let number_ctor = register(
            scope.statics.number_ctor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [
                (sym::isFinite, scope.statics.number_is_finite.clone()),
                (sym::isNaN, scope.statics.number_is_nan.clone()),
                (sym::isSafeInteger, scope.statics.number_is_safe_integer.clone()),
                (sym::isInteger, scope.statics.number_is_integer.clone()),
            ],
            [],
            [
                (sym::EPSILON, Value::number(f64::EPSILON), Some(PropertyDataDescriptor::empty())),
                (sym::MAX_SAFE_INTEGER, Value::number(value::primitive::MAX_SAFE_INTEGERF), Some(PropertyDataDescriptor::empty())),
                (sym::MAX_VALUE, Value::number(f64::MAX), Some(PropertyDataDescriptor::empty())),
                (sym::MIN_SAFE_INTEGER, Value::number(value::primitive::MIN_SAFE_INTEGERF), Some(PropertyDataDescriptor::empty())),
                (sym::MIN_VALUE, Value::number(f64::MIN), Some(PropertyDataDescriptor::empty())),
                (sym::NEGATIVE_INFINITY, Value::number(f64::NEG_INFINITY), Some(PropertyDataDescriptor::empty())),
                (sym::POSITIVE_INFINITY, Value::number(f64::INFINITY), Some(PropertyDataDescriptor::empty())),
                // TODO: this needs to be writable: false, causes test262 language/types/number/S8.5_A14_T1.js to fail
                (sym::NaN, Value::number(f64::NAN), Some(PropertyDataDescriptor::empty()))
            ],
            Some((sym::Number, scope.statics.number_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.number_prototype.clone(),
            object_proto.clone(),
            number_ctor.clone(),
            [
                (sym::toString, scope.statics.number_tostring.clone()),
                (sym::valueOf, scope.statics.number_valueof.clone()),
                (sym::toFixed, scope.statics.number_to_fixed.clone()),
            ],
            [],
            [],
            None,
            &mut scope,
        );
        
        let boolean_ctor = register(
            scope.statics.boolean_ctor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [],
            [],
            [],
            Some((sym::Boolean, scope.statics.boolean_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.boolean_prototype.clone(),
            object_proto.clone(),
            boolean_ctor.clone(),
            [
                (sym::toString, scope.statics.boolean_tostring.clone()),
                (sym::valueOf, scope.statics.boolean_valueof.clone()),
            ],
            [],
            [],
            None,
            &mut scope,
        );
        
        let string_ctor = register(
            scope.statics.string_ctor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [
                (sym::fromCharCode, scope.statics.string_from_char_code.clone()),
            ],
            [],
            [],
            Some((sym::String, scope.statics.string_prototype.clone())),
            &mut scope,
        );
        
        register(
           scope.statics.string_prototype.clone(),
           scope.statics.object_prototype.clone(),
           scope.statics.string_ctor.clone(),
           [
                (sym::toString, scope.statics.string_tostring.clone()),
                (sym::charAt, scope.statics.string_char_at.clone()),
                (sym::charCodeAt, scope.statics.string_char_code_at.clone()),
                (sym::concat, scope.statics.string_concat.clone()),
                (sym::endsWith, scope.statics.string_ends_with.clone()),
                (sym::startsWith, scope.statics.string_starts_with.clone()),
                (sym::includes, scope.statics.string_includes.clone()),
                (sym::indexOf, scope.statics.string_index_of.clone()),
                (sym::lastIndexOf, scope.statics.string_last_index_of.clone()),
                (sym::padEnd, scope.statics.string_pad_end.clone()),
                (sym::padStart, scope.statics.string_pad_start.clone()),
                (sym::repeat, scope.statics.string_repeat.clone()),
                (sym::replace, scope.statics.string_replace.clone()),
                (sym::replaceAll, scope.statics.string_replace_all.clone()),
                (sym::split, scope.statics.string_split.clone()),
                (sym::toLowerCase, scope.statics.string_to_lowercase.clone()),
                (sym::toUpperCase, scope.statics.string_to_uppercase.clone()),
                (sym::big, scope.statics.string_big.clone()),
                (sym::blink, scope.statics.string_blink.clone()),
                (sym::bold, scope.statics.string_bold.clone()),
                (sym::fixed, scope.statics.string_fixed.clone()),
                (sym::italics, scope.statics.string_italics.clone()),
                (sym::strike, scope.statics.string_strike.clone()),
                (sym::sub, scope.statics.string_sub.clone()),
                (sym::sup, scope.statics.string_sup.clone()),
                (sym::fontcolor, scope.statics.string_fontcolor.clone()),
                (sym::fontsize, scope.statics.string_fontsize.clone()),
                (sym::link, scope.statics.string_link.clone()),
                (sym::trim, scope.statics.string_trim.clone()),
                (sym::trimStart, scope.statics.string_trim_start.clone()),
                (sym::trimEnd, scope.statics.string_trim_end.clone()),
                (sym::substr, scope.statics.string_substr.clone()),
                (sym::substring, scope.statics.string_substring.clone()),
                (sym::slice, scope.statics.string_slice.clone()),
            ],
           [(scope.statics.symbol_iterator.clone(), scope.statics.string_iterator.clone())],
           [],
           None,
           &mut scope,
        );
        
        let array_ctor = register(
            scope.statics.array_ctor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [
                (sym::from, scope.statics.array_from.clone()),
                (sym::isArray, scope.statics.array_is_array.clone()),
            ],
            [],
            [],
            Some((sym::Array, scope.statics.array_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.array_prototype.clone(),
            object_proto.clone(),
            array_ctor.clone(),
            [
                (sym::toString, scope.statics.array_tostring.clone()),
                (sym::join, scope.statics.array_join.clone()),
                (sym::values, scope.statics.array_values.clone()),
                (sym::at, scope.statics.array_at.clone()),
                (sym::concat, scope.statics.array_concat.clone()),
                (sym::entries, scope.statics.array_entries.clone()),
                (sym::keys, scope.statics.array_keys.clone()),
                (sym::every, scope.statics.array_every.clone()),
                (sym::some, scope.statics.array_some.clone()),
                (sym::fill, scope.statics.array_fill.clone()),
                (sym::filter, scope.statics.array_filter.clone()),
                (sym::reduce, scope.statics.array_reduce.clone()),
                (sym::find, scope.statics.array_find.clone()),
                (sym::findIndex, scope.statics.array_find_index.clone()),
                (sym::flat, scope.statics.array_flat.clone()),
                (sym::forEach, scope.statics.array_for_each.clone()),
                (sym::includes, scope.statics.array_includes.clone()),
                (sym::indexOf, scope.statics.array_index_of.clone()),
                (sym::map, scope.statics.array_map.clone()),
                (sym::pop, scope.statics.array_pop.clone()),
                (sym::push, scope.statics.array_push.clone()),
                (sym::reverse, scope.statics.array_reverse.clone()),
                (sym::shift, scope.statics.array_shift.clone()),
                (sym::sort, scope.statics.array_sort.clone()),
                (sym::unshift, scope.statics.array_unshift.clone()),
                (sym::slice, scope.statics.array_slice.clone()),
                (sym::lastIndexOf, scope.statics.array_last_index_of.clone()),
            ],
            [(scope.statics.symbol_iterator.clone(), scope.statics.array_values.clone())],
            [],
            None,
            &mut scope,
        );
        
        register(
            scope.statics.array_iterator_prototype.clone(),
            object_proto.clone(), // TODO: wrong
            function_ctor.clone(), // TODO: ^
            [
                (sym::next, scope.statics.array_iterator_next.clone()),
            ],
            [
                (scope.statics.symbol_iterator.clone(), scope.statics.identity_this.clone()),
            ],
            [],
            None,
            &mut scope,
        );
        
        register(
            scope.statics.generator_iterator_prototype.clone(),
            object_proto.clone(), // TODO: wrong
            function_ctor.clone(), // TODO: ^
            [
                (sym::next, scope.statics.generator_iterator_next.clone()),
            ],
            [
                (scope.statics.symbol_iterator.clone(), scope.statics.identity_this.clone()),
            ],
            [],
            None,
            &mut scope,
        );
        
        let symbol_ctor = register(
            scope.statics.symbol_ctor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [],
            [],
            [
                (sym::asyncIterator,Value::symbol(scope.statics.symbol_async_iterator.clone()), Some(PropertyDataDescriptor::empty())),
                (sym::hasInstance, Value::symbol(scope.statics.symbol_has_instance.clone()), Some(PropertyDataDescriptor::empty())),
                (sym::iterator, Value::symbol(scope.statics.symbol_iterator.clone()), Some(PropertyDataDescriptor::empty())),
                (sym::match_, Value::symbol(scope.statics.symbol_match.clone()), Some(PropertyDataDescriptor::empty())),
                (sym::matchAll, Value::symbol(scope.statics.symbol_match_all.clone()), Some(PropertyDataDescriptor::empty())),
                (sym::replace, Value::symbol(scope.statics.symbol_replace.clone()), Some(PropertyDataDescriptor::empty())),
                (sym::search, Value::symbol(scope.statics.symbol_search.clone()), Some(PropertyDataDescriptor::empty())),
                (sym::species, Value::symbol(scope.statics.symbol_species.clone()), Some(PropertyDataDescriptor::empty())),
                (sym::split, Value::symbol(scope.statics.symbol_split.clone()), Some(PropertyDataDescriptor::empty())),
                (sym::toPrimitive, Value::symbol(scope.statics.symbol_to_primitive.clone()), Some(PropertyDataDescriptor::empty())),
                (sym::toStringTag, Value::symbol(scope.statics.symbol_to_string_tag.clone()), Some(PropertyDataDescriptor::empty())),
                (sym::unscopables, Value::symbol(scope.statics.symbol_unscopables.clone()), Some(PropertyDataDescriptor::empty())),
            ],
            Some((sym::JsSymbol, scope.statics.symbol_prototype.clone())),
            &mut scope,
        );
        
        let error_ctor = register(
            scope.statics.error_ctor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [],
            [],
            [],
            Some((sym::Error, scope.statics.error_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.error_prototype.clone(),
            object_proto.clone(),
            error_ctor.clone(),
            [
                (sym::toString, scope.statics.error_to_string.clone()),
            ],
            [],
            [],
            None,
            &mut scope,
        );
        
        let arraybuffer_ctor = register(
            scope.statics.arraybuffer_ctor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [],
            [],
            [],
            Some((sym::ArrayBuffer, scope.statics.arraybuffer_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.arraybuffer_prototype.clone(),
            object_proto.clone(),
            arraybuffer_ctor.clone(),
            [
                (sym::byteLength, scope.statics.arraybuffer_byte_length.clone()) // TODO: should be a getter really
            ],
            [],
            [],
            None,
            &mut scope,
        );
        
        let u8array_ctor = register(
            scope.statics.uint8array_ctor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [],
            [],
            [],
            Some((sym::Uint8Array, scope.statics.uint8array_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.uint8array_prototype.clone(),
            object_proto.clone(),
            u8array_ctor.clone(),
            [
                (sym::fill, scope.statics.typedarray_fill.clone()),
            ],
            [],
            [],
            None,
            &mut scope,
        );
        
        let i8array_ctor = register(
            scope.statics.int8array_ctor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [],
            [],
            [],
            Some((sym::Int8Array, scope.statics.int8array_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.int8array_prototype.clone(),
            object_proto.clone(),
            i8array_ctor.clone(),
            [
                (sym::fill, scope.statics.typedarray_fill.clone()),
            ],
            [],
            [],
            None,
            &mut scope,
        );
        
        let u16array_ctor = register(
            scope.statics.uint16array_ctor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [],
            [],
            [],
            Some((sym::Uint16Array, scope.statics.uint16array_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.uint16array_prototype.clone(),
            object_proto.clone(),
            u16array_ctor.clone(),
            [
                (sym::fill, scope.statics.typedarray_fill.clone()),
            ],
            [],
            [],
            None,
            &mut scope,
        );
        
        let i16array_ctor = register(
            scope.statics.int16array_ctor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [],
            [],
            [],
            Some((sym::Int16Array, scope.statics.int16array_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.int16array_prototype.clone(),
            object_proto.clone(),
            i16array_ctor.clone(),
            [
                (sym::fill, scope.statics.typedarray_fill.clone()),
            ],
            [],
            [],
            None,
            &mut scope,
        );
        
        let u32array_ctor = register(
            scope.statics.uint32array_ctor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [],
            [],
            [],
            Some((sym::Uint32Array, scope.statics.uint32array_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.uint32array_prototype.clone(),
            object_proto.clone(),
            u32array_ctor.clone(),
            [
                (sym::fill, scope.statics.typedarray_fill.clone()),
            ],
            [],
            [],
            None,
            &mut scope,
        );
        
        let i32array_ctor = register(
            scope.statics.int32array_ctor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [],
            [],
            [],
            Some((sym::Int32Array, scope.statics.int32array_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.int32array_prototype.clone(),
            object_proto.clone(),
            i32array_ctor.clone(),
            [
                (sym::fill, scope.statics.typedarray_fill.clone()),
            ],
            [],
            [],
            None,
            &mut scope,
        );
        
        let f32array_ctor = register(
            scope.statics.float32array_ctor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [],
            [],
            [],
            Some((sym::Float32Array, scope.statics.float32array_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.float32array_prototype.clone(),
            object_proto.clone(),
            f32array_ctor.clone(),
            [
                (sym::fill, scope.statics.typedarray_fill.clone()),
            ],
            [],
            [],
            None,
            &mut scope,
        );
        
        let f64array_ctor = register(
            scope.statics.float64array_ctor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [],
            [],
            [],
            Some((sym::Float64Array, scope.statics.float64array_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.float64array_prototype.clone(),
            object_proto.clone(),
            f64array_ctor.clone(),
            [
                (sym::fill, scope.statics.typedarray_fill.clone()),
            ],
            [],
            [],
            None,
            &mut scope,
        );
        
        let promise_ctor = register(
            scope.statics.promise_ctor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [
                (sym::resolve, scope.statics.promise_resolve.clone()),
                (sym::reject, scope.statics.promise_reject.clone()),
            ],
            [],
            [],
            Some((sym::Promise, scope.statics.promise_proto.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.promise_proto.clone(),
            object_proto.clone(),
            promise_ctor.clone(),
            [
                (sym::then, scope.statics.promise_then.clone()),
            ],
            [],
            [],
            None,
            &mut scope,
        );
        
        let set_ctor = register(
            scope.statics.set_constructor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [],
            [],
            [],
            Some((sym::Set, scope.statics.set_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.set_prototype.clone(),
            object_proto.clone(),
            set_ctor.clone(),
            [
                (sym::add, scope.statics.set_add.clone()),
                (sym::has, scope.statics.set_has.clone()),
                (sym::delete, scope.statics.set_delete.clone()),
                (sym::clear, scope.statics.set_clear.clone()),
                (sym::size, scope.statics.set_size.clone()),
            ],
            [],
            [],
            None,
            &mut scope,
        );
        
        let map_ctor = register(
            scope.statics.map_constructor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [],
            [],
            [],
            Some((sym::Map, scope.statics.map_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.map_prototype.clone(),
            object_proto.clone(),
            map_ctor.clone(),
            [
                (sym::set, scope.statics.map_set.clone()),
                (sym::get, scope.statics.map_get.clone()),
                (sym::has, scope.statics.map_has.clone()),
                (sym::delete, scope.statics.map_delete.clone()),
                (sym::clear, scope.statics.map_clear.clone()),
                (sym::size, scope.statics.map_size.clone()), // TODO: this should be a getter
            ],
            [],
            [],
            None,
            &mut scope,
        );
        
        let regexp_ctor = register(
            scope.statics.regexp_ctor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [],
            [],
            [],
            Some((sym::RegExp, scope.statics.regexp_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.regexp_prototype.clone(),
            object_proto.clone(),
            regexp_ctor.clone(),
            [
                (sym::test, scope.statics.regexp_test.clone()),
                (sym::exec, scope.statics.regexp_exec.clone())
            ],
            [],
            [],
            None,
            &mut scope,
        );
        
        let eval_error_ctor = register(
            scope.statics.eval_error_ctor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [],
            [],
            [],
            Some((sym::EvalError, scope.statics.eval_error_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.eval_error_prototype.clone(),
            scope.statics.error_prototype.clone(),
            eval_error_ctor.clone(),
            [
                (sym::toString, scope.statics.error_to_string.clone()),
            ],
            [],
            [],
            None,
            &mut scope,
        );
        
        let range_error_ctor = register(
            scope.statics.range_error_ctor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [],
            [],
            [],
            Some((sym::RangeError, scope.statics.range_error_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.range_error_prototype.clone(),
            scope.statics.error_prototype.clone(),
            range_error_ctor.clone(),
            [
                (sym::toString, scope.statics.error_to_string.clone()),
            ],
            [],
            [],
            None,
            &mut scope,
        );
        
        let reference_error_ctor = register(
            scope.statics.reference_error_ctor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [],
            [],
            [],
            Some((sym::ReferenceError, scope.statics.reference_error_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.reference_error_prototype.clone(),
            scope.statics.error_prototype.clone(),
            reference_error_ctor.clone(),
            [
                (sym::toString, scope.statics.error_to_string.clone()),
            ],
            [],
            [],
            None,
            &mut scope,
        );
        
        let syntax_error_ctor = register(
            scope.statics.syntax_error_ctor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [],
            [],
            [],
            Some((sym::SyntaxError, scope.statics.syntax_error_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.syntax_error_prototype.clone(),
            scope.statics.error_prototype.clone(),
            syntax_error_ctor.clone(),
            [
                (sym::toString, scope.statics.error_to_string.clone()),
            ],
            [],
            [],
            None,
            &mut scope,
        );
        
        let type_error_ctor = register(
            scope.statics.type_error_ctor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [],
            [],
            [],
            Some((sym::TypeError, scope.statics.type_error_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.type_error_prototype.clone(),
            scope.statics.error_prototype.clone(),
            type_error_ctor.clone(),
            [
                (sym::toString, scope.statics.error_to_string.clone()),
            ],
            [],
            [],
            None,
            &mut scope,
        );
        
        let uri_error_ctor = register(
            scope.statics.uri_error_ctor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [],
            [],
            [],
            Some((sym::URIError, scope.statics.uri_error_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.uri_error_prototype.clone(),
            scope.statics.error_prototype.clone(),
            uri_error_ctor.clone(),
            [
                (sym::toString, scope.statics.error_to_string.clone()),
            ],
            [],
            [],
            None,
            &mut scope,
        );
        
        let aggregate_error_ctor = register(
            scope.statics.aggregate_error_ctor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [],
            [],
            [],
            Some((sym::AggregateError, scope.statics.aggregate_error_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.aggregate_error_prototype.clone(),
            scope.statics.error_prototype.clone(),
            aggregate_error_ctor.clone(),
            [
                (sym::toString, scope.statics.error_to_string.clone()),
            ],
            [],
            [],
            None,
            &mut scope,
        );
        
        let date_ctor = register(
            scope.statics.date_ctor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [
                (sym::now, scope.statics.date_now.clone()),
            ],
            [],
            [],
            Some((sym::Date, scope.statics.date_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.date_prototype.clone(),
            object_proto.clone(),
            date_ctor.clone(),
            [],
            [],
            [],
            None,
            &mut scope,
        );
        
        let json_ctor = register(
            scope.statics.json_ctor.clone(),
            function_proto,
            function_ctor,
            [
                (sym::parse, scope.statics.json_parse.clone()),
            ],
            [],
            [],
            None,
            &mut scope,
        );
        
        register(
            global,
            object_proto,
            object_ctor.clone(),
            [
                (sym::isNaN, scope.statics.is_nan.clone()),
                (sym::eval, scope.statics.eval.clone()),
                (sym::isFinite, scope.statics.is_finite.clone()),
                (sym::parseFloat, scope.statics.parse_float.clone()),
                (sym::parseInt, scope.statics.parse_int.clone()),
                (sym::RegExp, regexp_ctor),
                (sym::JsSymbol, symbol_ctor),
                (sym::Date, date_ctor),
                (sym::ArrayBuffer, arraybuffer_ctor),
                (sym::Uint8Array, u8array_ctor),
                (sym::Int8Array, i8array_ctor),
                (sym::Uint16Array, u16array_ctor),
                (sym::Int16Array, i16array_ctor),
                (sym::Uint32Array, u32array_ctor),
                (sym::Int32Array, i32array_ctor),
                (sym::Float32Array, f32array_ctor),
                (sym::Float64Array, f64array_ctor),
                (sym::Array, array_ctor),
                (sym::Error, error_ctor),
                (sym::EvalError, eval_error_ctor),
                (sym::RangeError, range_error_ctor),
                (sym::ReferenceError, reference_error_ctor),
                (sym::SyntaxError, syntax_error_ctor),
                (sym::TypeError, type_error_ctor),
                (sym::URIError, uri_error_ctor),
                (sym::AggregateError, aggregate_error_ctor),
                (sym::String, string_ctor),
                (sym::Object, object_ctor),
                (sym::Set, set_ctor),
                (sym::Map, map_ctor),
                (sym::console, console),
                (sym::Math, math),
                (sym::Number, number_ctor),
                (sym::Boolean, boolean_ctor),
                (sym::Promise, promise_ctor),
                (sym::JSON, json_ctor),
            ],
            [],
            [],
            None,
            &mut scope
        );
    }

    pub(crate) fn active_frame(&self) -> &Frame {
        self.frames.last().expect("frames stack is empty")
    }

    #[cfg_attr(dash_lints, dash_lints::trusted_no_gc)]
    pub(crate) fn active_frame_mut(&mut self) -> &mut Frame {
        self.frames.last_mut().expect("frames stack is empty")
    }

    /// Fetches the current instruction/value in the currently executing frame
    /// and increments the instruction pointer
    #[cfg_attr(dash_lints, dash_lints::trusted_no_gc)]
    pub(crate) fn fetch_and_inc_ip(&mut self) -> u8 {
        let frame = self.active_frame_mut();
        let ip = frame.ip;
        frame.ip += 1;
        frame.function.buffer.with(|buf| buf[ip])
    }

    /// Fetches a wide value (16-bit) in the currently executing frame
    /// and increments the instruction pointer
    #[cfg_attr(dash_lints, dash_lints::trusted_no_gc)]
    pub(crate) fn fetchw_and_inc_ip(&mut self) -> u16 {
        let frame = self.active_frame_mut();
        let value: [u8; 2] = frame.function.buffer.with(|buf| {
            buf[frame.ip..frame.ip + 2]
                .try_into()
                .expect("Failed to get wide instruction")
        });

        frame.ip += 2;
        u16::from_ne_bytes(value)
    }

    #[cfg_attr(dash_lints, dash_lints::trusted_no_gc)]
    pub(crate) fn get_frame_sp(&self) -> usize {
        self.active_frame().sp
    }

    /// Fetches a local, while also preserving external values
    pub(crate) fn get_local_raw(&self, id: usize) -> Option<Value> {
        self.stack.get(self.get_frame_sp() + id).cloned()
    }

    /// Fetches a local and unboxes any externals.
    /// 
    /// This is usually the method you want to use, since handling externals specifically is not
    /// typically useful.
    pub(crate) fn get_local(&self, id: usize) -> Option<Value> {
        self.stack.get(self.get_frame_sp() + id).cloned().map(|v| v.unbox_external(self))
    }

    pub(crate) fn get_external(&self, id: usize) -> Option<&ExternalValue> {
        self.active_frame().externals.get(id)
    }

    #[cfg_attr(dash_lints, dash_lints::trusted_no_gc)]
    pub(crate) fn set_local(&mut self, id: usize, value: Unrooted) {
        let sp = self.get_frame_sp();
        let idx = sp + id;

        // SAFETY: GC cannot trigger here
        // and value will become a root here, therefore this is ok
        let value = unsafe { value.into_value() };

        // TODO: double check this is still right
        if let ValueKind::External(o) = self.stack[idx].unpack() {
            unsafe { ExternalValue::replace(self, &o, value) };
        } else {
            self.stack[idx] = value;
        }
    }

    pub(crate) fn push_stack(&mut self, value: Unrooted) {
        // SAFETY: Value will become a root here, therefore we don't need to root with a scope
        let value = unsafe { value.into_value() };
        self.stack.push(value);
    }

    pub(crate) fn try_push_frame(&mut self, frame: Frame) -> Result<(), Unrooted> {
        if self.frames.len() <= MAX_FRAME_STACK_SIZE {
            self.frames.push(frame);
        } else {
            cold_path();
            // This is a bit sus (we're creating a temporary scope for the error creation and returning it past its scope),
            // but the error type is `Unrooted`, so it needs to be re-rooted at callsite anyway.
            throw!(self.scope(), RangeError, "Maximum call stack size exceeded");
        }
        Ok(())
    }

    pub(crate) fn try_extend_stack<I>(&mut self, other: I) -> Result<(), Unrooted>
    where
        I: IntoIterator<Item = Value>,
        <I as IntoIterator>::IntoIter: ExactSizeIterator,
    {
        let it = other.into_iter();
        let len = it.len();
        if self.stack.len() + len > MAX_STACK_SIZE {
            debug!("vm exceeded stack size");
            // This is a bit sus (we're creating a temporary scope for the error creation and returning it past its scope),
            // but the error type is `Unrooted`, so it needs to be re-rooted at callsite anyway.
            throw!(self.scope(), RangeError, "Maximum stack size exceeded");
        }
        self.stack.extend(it);
        Ok(())
    }

    pub(crate) fn stack_size(&self) -> usize {
        self.stack.len()
    }

    pub(crate) fn pop_frame(&mut self) -> Option<Frame> {
        self.frames.pop()
    }

    // TODO: should actually yield iterator over Unrooted
    pub(crate) fn drain_stack<R>(&mut self, range: R) -> Drain<'_, Value>
    where
        R: RangeBounds<usize>,
    {
        self.stack.drain(range)
    }

    pub fn pop_stack(&mut self) -> Option<Unrooted> {
        let value = self.stack.pop()?;
        Some(Unrooted::new(value))
    }

    pub fn pop_stack_unwrap(&mut self) -> Unrooted {
        let value = self.stack.pop().expect("Expected value on stack");
        Unrooted::new(value)
    }

    fn handle_rt_error(&mut self, err: Unrooted, max_fp: usize) -> Result<(), Unrooted> {
        debug!("handling rt error @{max_fp}");
        // Using .last() here instead of .pop() because there is a possibility that we
        // can't use this block (read the comment above the if statement try_fp < max_fp)
        if let Some(&TryBlock { catch_ip, finally_ip, frame_ip: try_fp }) = self.try_blocks.last() {
            // if we're in a try-catch block, we need to jump to it

            // Do not unwind further than we are allowed to. If the last try block is "outside" of
            // the frame that this execution context was instantiated in, then we can't jump there.
            if try_fp < max_fp {
                // TODO: don't duplicate this code
                self.frames.pop();
                return Err(err);
            }

            // If we've found a suitable try block, actually remove it.
            self.try_blocks.pop();

            // Unwind frames
            drop(self.frames.drain(try_fp..));

            if let Some(catch_ip) = catch_ip {
                self.active_frame_mut().ip = catch_ip;

                let catch_binding = self.fetchw_and_inc_ip();
                if catch_binding != u16::MAX {
                    // u16::MAX is used to indicate that there is no variable binding in the catch block
                    self.set_local(catch_binding as usize, err);
                }
                
                // If we have both a catch_ip and finally_ip, then re-push it but with the catch_ip set to None
                // and then jump to the old catch_ip.
                // Reason: when we then throw an exception within this catch, we correctly jump to the finally.
                if let Some(finally_ip) = finally_ip {
                    self.try_blocks.push(TryBlock{
                        catch_ip: None,
                        finally_ip: Some(finally_ip),
                        frame_ip: try_fp
                    });
                }
            } else if let Some(finally_ip) = finally_ip {
                self.active_frame_mut().delayed_ret = Some(Err(err));
                // `+ 1` because we need to jump over the `TryEnd` instruction since there won't be a try block to pop.
                self.active_frame_mut().ip = finally_ip + 1;
            }

            Ok(())
        } else {
            self.frames.pop();
            Err(err)
        }
    }

    /// Mostly useful for debugging
    pub fn print_stack(&self) {
        for (i, v) in self.stack.iter().enumerate() {
            print!("{i}: ");
            match v.unpack() {
                ValueKind::Object(o) => println!("{:#?}", o),
                ValueKind::External(o) => println!("[[external]]: {:#?}", o.inner(self)),
                v => println!("{v:?}"),
            }
        }
    }

    /// Adds a function to the async task queue.
    pub fn add_async_task(&mut self, fun: ObjectId) {
        self.async_tasks.push(fun);
    }

    pub fn has_async_tasks(&self) -> bool {
        !self.async_tasks.is_empty()
    }

    /// Processes all queued async tasks
    pub fn process_async_tasks(&mut self) {
        debug!("process async tasks");
        debug!(async_task_count = %self.async_tasks.len());

        while let Some(task) = self.async_tasks.pop() {
            let mut scope = self.scope();

            scope.add_ref(task.clone());

            debug!("process task {:?}", task);
            if let Err(ex) = task.apply(&mut scope, Value::undefined(), Vec::new()) {
                if let Some(callback) = scope.params.unhandled_task_exception_callback() {
                    let ex = ex.root(&mut scope);
                    error!("uncaught async task exception");
                    callback(&mut scope, ex);
                }
            }
        }
    }

    /// Executes a frame in this VM and initializes local variables (excluding parameters)
    ///
    /// Parameters must be pushed onto the stack in the correct order by the caller before this function is called.
    pub fn execute_frame(&mut self, frame: Frame) -> Result<HandleResult, Unrooted> {
        debug!("execute frame {:?}", frame.function.name);
        let span = span!(Level::TRACE, "vm frame");
        span.in_scope(|| {
            self.pad_stack_for_frame(&frame);
            self.execute_frame_raw(frame)
        })
    }

    /// Does the necessary stack management that needs to be done before executing a JavaScript frame
    pub(crate) fn pad_stack_for_frame(&mut self, frame: &Frame) {
        let pad_to = self.stack.len() + frame.extra_stack_space;
        debug!(pad_to);
        // TODO: check that the stack space won't exceed our stack frame limit
        self.stack.resize(pad_to, Value::undefined());
    }

    /// Executes a frame in this VM, without doing any sort of stack management
    fn execute_frame_raw(&mut self, frame: Frame) -> Result<HandleResult, Unrooted> {
        // TODO: if this fails, we MUST revert the stack management,
        // like reserving space for undefined values
        self.try_push_frame(frame)?;
        self.handle_instruction_loop()
    }

    fn handle_instruction_loop(&mut self) -> Result<HandleResult, Unrooted> {
        let fp = self.frames.len();

        loop {
            #[cfg(feature = "stress_gc")]
            {
                self.perform_gc();
            }
            #[cfg(not(feature = "stress_gc"))]
            {
                if util::unlikely(self.alloc.rss() > self.gc_rss_threshold) {
                    self.perform_gc();
                }
            }

            let instruction = Instruction::from_repr(self.fetch_and_inc_ip()).unwrap();

            match dispatch::handle(self, instruction) {
                Ok(Some(hr)) => return Ok(hr),
                Ok(None) => continue,
                Err(e) => self.handle_rt_error(e, fp)?, // TODO: pop frame
            }
        }
    }

    pub fn execute_module(&mut self, mut frame: Frame) -> Result<Exports, Unrooted> {
        frame.state = FrameState::Module(Exports::default());
        frame.sp = self.stack.len();
        self.execute_frame(frame)?;

        let frame = self.frames.pop().expect("Missing module frame");
        Ok(match frame.state {
            FrameState::Module(exports) => exports,
            _ => unreachable!(),
        })
    }

    pub fn with_scope<R>(&mut self, f: impl FnOnce(&mut LocalScope<'_>) -> R) -> R {
        let mut scope = self.scope();
        f(&mut scope)
    }

    pub fn perform_gc(&mut self) {
        debug!("gc cycle triggered");

        let trace_roots = span!(Level::TRACE, "gc trace");
        trace_roots.in_scope(|| self.trace_roots());

        // All reachable roots are marked.
        debug!("rss before sweep: {}", self.alloc.rss());
        let sweep = span!(Level::TRACE, "gc sweep");
        sweep.in_scope(|| unsafe { self.alloc.sweep() });
        debug!("rss after sweep: {}", self.alloc.rss());

        debug!("sweep interner");
        self.interner.sweep();

        // Adjust GC threshold
        self.gc_rss_threshold = self.alloc.rss() * 2;
        debug!("new threshold: {}", self.gc_rss_threshold);
    }

    fn trace_roots(&mut self) {
        let mut cx = TraceCtxt::new(&mut self.interner, &mut self.alloc);

        debug!("trace frames");
        self.frames.trace(&mut cx);
        debug!("trace async tasks");
        self.async_tasks.trace(&mut cx);
        debug!("trace stack");
        self.stack.trace(&mut cx);
        debug!("trace globals");
        self.global.trace(&mut cx);
        debug!("trace scopes");
        self.scopes.trace(&mut cx);
        if let Some(state) = self.params.state_raw() {
            debug!("trace state");
            state.trace(&mut cx);
        }

        debug!("trace externals");
        // we do two things here:
        // remove Handles from external refs set that have a zero refcount (implying no active persistent refs)
        // and trace if refcount > 0
        self.external_refs.0.borrow_mut().retain(|&id, &mut refcount| {
            if refcount == 0 {
                false
            } else {
                // Non-zero refcount, retain object and trace
                id.trace(&mut cx);
                true
            }
        });

        debug!("trace statics");
        self.statics.trace(&mut cx);
    }

    pub fn statics(&self) -> &Statics {
        &self.statics
    }

    pub fn params(&self) -> &VmParams {
        &self.params
    }

    pub fn params_mut(&mut self) -> &mut VmParams {
        &mut self.params
    }

    pub(crate) fn builtins_purity(&self) -> bool {
        self.builtins_pure
    }

    pub(crate) fn impure_builtins(&mut self) {
        self.builtins_pure = false;
    }

    // -- JIT specific methods --

    /// Marks an instruction pointer (i.e. code region) as JIT-"poisoned".
    /// It will replace the instruction with one that does not attempt to trigger a trace.
    #[cfg(feature = "jit")]
    pub(crate) fn poison_ip(&mut self, ip: usize) {
        dash_log::warn!("ip poisoned: {}", ip);
        self.active_frame().function.poison_ip(ip);
    }

    // TODO: move these to DispatchContext.
    #[cfg(feature = "jit")]
    pub(crate) fn record_conditional_jump(&mut self, ip: usize, did_jump: bool) {
        use dash_typed_cfg::passes::bb_generation::ConditionalBranchAction;

        if let Some(trace) = self.jit.recording_trace_mut() {
            trace.record_conditional_jump(
                ip,
                match did_jump {
                    true => ConditionalBranchAction::Taken,
                    false => ConditionalBranchAction::NotTaken,
                },
            );
        }
    }
}

pub enum PromiseAction {
    Resolve,
    Reject,
}

impl fmt::Debug for Vm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Vm")
    }
}
