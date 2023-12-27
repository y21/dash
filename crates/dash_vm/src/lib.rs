#![warn(clippy::redundant_clone)]

use std::ops::RangeBounds;
use std::vec::Drain;
use std::{fmt, mem};

use crate::gc::interner::{self, sym};
use crate::gc::trace::{Trace, TraceCtxt};
use crate::util::cold_path;
use crate::value::function::Function;
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
use gc::handle::Handle;
use gc::interner::StringInterner;
use gc::Gc;
use localscope::{scope, LocalScopeList};
use rustc_hash::FxHashSet;
use value::object::NamedObject;
use value::{ExternalValue, PureBuiltin, Unrooted};

#[cfg(feature = "jit")]
mod jit;

pub mod dispatch;
pub mod eval;
pub mod external;
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
const DEFAULT_GC_OBJECT_COUNT_THRESHOLD: usize = 8192;

pub struct Vm {
    frames: Vec<Frame>,
    async_tasks: Vec<Handle<dyn Object>>,
    // TODO: the inner vec of the stack should be private for soundness
    // popping from the stack must return `Unrooted`
    stack: Vec<Value>,
    gc: Gc,
    pub interner: StringInterner,
    global: Handle<dyn Object>,
    // "External refs" currently refers to existing `Persistent<T>`s.
    // Persistent values already manage the reference count when cloning or dropping them
    // and are stored in the Handle itself, but we still need to keep track of them so we can
    // consider them as roots and also **trace** them (to reach their children).
    //
    // We insert into this in `Persistent::new`, and remove from it during the tracing phase.
    // We can't do that in Persistent's Drop code, because we don't have access to the VM there.
    external_refs: FxHashSet<Handle<dyn Object>>,
    scopes: LocalScopeList,
    statics: Box<Statics>, // TODO: we should box this... maybe?
    try_blocks: Vec<TryBlock>,
    params: VmParams,
    gc_object_threshold: usize,
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
        let mut gc = Gc::default();
        let statics = Statics::new(&mut gc);
        // TODO: global __proto__ and constructor
        let global = gc.register(PureBuiltin::new(NamedObject::null()));
        let gc_object_threshold = params
            .initial_gc_object_threshold()
            .unwrap_or(DEFAULT_GC_OBJECT_COUNT_THRESHOLD);

        let mut vm = Self {
            frames: Vec::new(),
            async_tasks: Vec::new(),
            stack: Vec::with_capacity(512),
            gc,
            interner: StringInterner::new(),
            global,
            external_refs: FxHashSet::default(),
            scopes: LocalScopeList::new(),
            statics: Box::new(statics),
            try_blocks: Vec::new(),
            params,
            gc_object_threshold,
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

    pub fn global(&self) -> Handle<dyn Object> {
        self.global.clone()
    }

    /// Prepare the VM for execution.
    #[rustfmt::skip]
    fn prepare(&mut self) {
        debug!("initialize vm intrinsics");
        fn set_fn_prototype(v: &dyn Object, proto: &Handle<dyn Object>, name: interner::Symbol) {
            let fun = v.as_any().downcast_ref::<Function>().unwrap();
            fun.set_name(name.into());
            fun.set_fn_prototype(proto.clone());
        }

        // TODO: we currently recursively call this for each of the registered methods, so a lot of builtins are initialized multiple times
        // we should have some sort of cache to avoid this
        // (though we also populate function prototypes later on this way, so it's not so trivial)
        fn register(
            base: Handle<dyn Object>,
            prototype: impl Into<Value>,
            constructor: Handle<dyn Object>,
            methods: impl IntoIterator<Item = (interner::Symbol, Handle<dyn Object>)>,
            symbols: impl IntoIterator<Item = (Symbol, Handle<dyn Object>)>,
            fields: impl IntoIterator<Item = (interner::Symbol, Value)>,
            // Contrary to `prototype`, this optionally sets the function prototype. Should only be `Some`
            // when base is a function
            fn_prototype: Option<(interner::Symbol, Handle<dyn Object>)>,

            // LocalScope needs to be the last parameter because we don't have two phase borrows in user code
            scope: &mut LocalScope<'_>,
        ) -> Handle<dyn Object> {
            base.set_property(scope, sym::CONSTRUCTOR.into(), PropertyValue::static_default(constructor.into())).unwrap();
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
                base.set_property(scope, key.into(), PropertyValue::static_default(value.into())).unwrap();
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
                base.set_property(scope, key.into(), PropertyValue::static_default(value.into())).unwrap();
            }

            for (key, value) in fields {
                base.set_property(scope, key.into(), PropertyValue::static_default(value.into())).unwrap();
            }

            if let Some((proto_name, proto_val)) = fn_prototype {
                set_fn_prototype(&base, &proto_val, proto_name);
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
            Some((sym::FUNCTION, scope.statics.function_proto.clone())),
            &mut scope,
        );
        
        let function_proto = register(
            scope.statics.function_proto.clone(),
            scope.statics.object_prototype.clone(),
            function_ctor.clone(),
            [
                (sym::BIND, scope.statics.function_bind.clone()),
                (sym::CALL, scope.statics.function_call.clone()),
                (sym::TO_STRING, scope.statics.function_to_string.clone()),
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
                (sym::CREATE, scope.statics.object_create.clone()),
                (sym::KEYS, scope.statics.object_keys.clone()),
                (sym::GET_OWN_PROPERTY_DESCRIPTOR, scope.statics.object_get_own_property_descriptor.clone()),
                (sym::GET_OWN_PROPERTY_DESCRIPTORS, scope.statics.object_get_own_property_descriptors.clone()),
                (sym::DEFINE_PROPERTY, scope.statics.object_define_property.clone()),
                (sym::ENTRIES, scope.statics.object_entries.clone()),
                (sym::ASSIGN, scope.statics.object_assign.clone()),
            ],
            [],
            [],
            Some((sym::OBJECT, scope.statics.object_prototype.clone())),
            &mut scope,
        );
        
        let object_proto = register(
            scope.statics.object_prototype.clone(),
            Value::null(),
            object_ctor.clone(),
            [
                (sym::TO_STRING, scope.statics.object_to_string.clone()),
                (sym::HAS_OWN_PROPERTY, scope.statics.object_has_own_property.clone()),
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
                (sym::LOG, scope.statics.console_log.clone()),
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
                (sym::FLOOR, scope.statics.math_floor.clone()),
                (sym::ABS, scope.statics.math_abs.clone()),
                (sym::ACOS, scope.statics.math_acos.clone()),
                (sym::ACOSH, scope.statics.math_acosh.clone()),
                (sym::ASIN, scope.statics.math_asin.clone()),
                (sym::ASINH, scope.statics.math_asinh.clone()),
                (sym::ATAN, scope.statics.math_atan.clone()),
                (sym::ATANH, scope.statics.math_atanh.clone()),
                (sym::ATAN2, scope.statics.math_atan2.clone()),
                (sym::CBRT, scope.statics.math_cbrt.clone()),
                (sym::CEIL, scope.statics.math_ceil.clone()),
                (sym::CLZ32, scope.statics.math_clz32.clone()),
                (sym::COS, scope.statics.math_cos.clone()),
                (sym::COSH, scope.statics.math_cosh.clone()),
                (sym::EXP, scope.statics.math_exp.clone()),
                (sym::EXPM1, scope.statics.math_expm1.clone()),
                (sym::LOG, scope.statics.math_log.clone()),
                (sym::LOG1P, scope.statics.math_log1p.clone()),
                (sym::LOG10, scope.statics.math_log10.clone()),
                (sym::LOG2, scope.statics.math_log2.clone()),
                (sym::ROUND, scope.statics.math_round.clone()),
                (sym::SIN, scope.statics.math_sin.clone()),
                (sym::SINH, scope.statics.math_sinh.clone()),
                (sym::SQRT, scope.statics.math_sqrt.clone()),
                (sym::TAN, scope.statics.math_tan.clone()),
                (sym::TANH, scope.statics.math_tanh.clone()),
                (sym::TRUNC, scope.statics.math_trunc.clone()),
                (sym::RANDOM, scope.statics.math_random.clone()),
                (sym::MAX, scope.statics.math_max.clone()),
                (sym::MIN, scope.statics.math_min.clone()),
            ],
            [],
            [
                (sym::PI, Value::number(std::f64::consts::PI)),
            ],
            None,
            &mut scope,
        );
        
        let number_ctor = register(
            scope.statics.number_ctor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [
                (sym::IS_FINITE, scope.statics.number_is_finite.clone()),
                (sym::IS_NA_N, scope.statics.number_is_nan.clone()),
                (sym::IS_SAFE_INTEGER, scope.statics.number_is_safe_integer.clone()),
            ],
            [],
            [],
            Some((sym::NUMBER, scope.statics.number_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.number_prototype.clone(),
            object_proto.clone(),
            number_ctor.clone(),
            [
                (sym::TO_STRING, scope.statics.number_tostring.clone()),
                (sym::TO_FIXED, scope.statics.number_to_fixed.clone()),
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
            Some((sym::BOOLEAN, scope.statics.boolean_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.boolean_prototype.clone(),
            object_proto.clone(),
            boolean_ctor.clone(),
            [
                (sym::TO_STRING, scope.statics.boolean_tostring.clone()),
                (sym::VALUE_OF, scope.statics.boolean_valueof.clone()),
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
                (sym::FROM_CHAR_CODE, scope.statics.string_from_char_code.clone()),
            ],
            [],
            [],
            Some((sym::STRING, scope.statics.string_prototype.clone())),
            &mut scope,
        );
        
        register(
           scope.statics.string_prototype.clone(),
           scope.statics.object_prototype.clone(),
           scope.statics.string_ctor.clone(),
           [
                (sym::TO_STRING, scope.statics.string_tostring.clone()),
                (sym::CHAR_AT, scope.statics.string_char_at.clone()),
                (sym::CHAR_CODE_AT, scope.statics.string_char_code_at.clone()),
                (sym::CONCAT, scope.statics.string_concat.clone()),
                (sym::ENDS_WITH, scope.statics.string_ends_with.clone()),
                (sym::STARTS_WITH, scope.statics.string_starts_with.clone()),
                (sym::INCLUDES, scope.statics.string_includes.clone()),
                (sym::INDEX_OF, scope.statics.string_index_of.clone()),
                (sym::LAST_INDEX_OF, scope.statics.string_last_index_of.clone()),
                (sym::PAD_END, scope.statics.string_pad_end.clone()),
                (sym::PAD_START, scope.statics.string_pad_start.clone()),
                (sym::REPEAT, scope.statics.string_repeat.clone()),
                (sym::REPLACE, scope.statics.string_replace.clone()),
                (sym::REPLACE_ALL, scope.statics.string_replace_all.clone()),
                (sym::SPLIT, scope.statics.string_split.clone()),
                (sym::TO_LOWER_CASE, scope.statics.string_to_lowercase.clone()),
                (sym::TO_UPPER_CASE, scope.statics.string_to_uppercase.clone()),
                (sym::BIG, scope.statics.string_big.clone()),
                (sym::BLINK, scope.statics.string_blink.clone()),
                (sym::BOLD, scope.statics.string_bold.clone()),
                (sym::FIXED, scope.statics.string_fixed.clone()),
                (sym::ITALICS, scope.statics.string_italics.clone()),
                (sym::STRIKE, scope.statics.string_strike.clone()),
                (sym::SUB, scope.statics.string_sub.clone()),
                (sym::SUP, scope.statics.string_sup.clone()),
                (sym::FONTCOLOR, scope.statics.string_fontcolor.clone()),
                (sym::FONTSIZE, scope.statics.string_fontsize.clone()),
                (sym::LINK, scope.statics.string_link.clone()),
                (sym::TRIM, scope.statics.string_trim.clone()),
                (sym::TRIM_START, scope.statics.string_trim_start.clone()),
                (sym::TRIM_END, scope.statics.string_trim_end.clone()),
                (sym::SUBSTR, scope.statics.string_substr.clone()),
                (sym::SUBSTRING, scope.statics.string_substring.clone()),
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
                (sym::FROM, scope.statics.array_from.clone()),
                (sym::IS_ARRAY, scope.statics.array_is_array.clone()),
            ],
            [],
            [],
            Some((sym::ARRAY, scope.statics.array_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.array_prototype.clone(),
            object_proto.clone(),
            array_ctor.clone(),
            [
                (sym::TO_STRING, scope.statics.array_tostring.clone()),
                (sym::JOIN, scope.statics.array_join.clone()),
                (sym::VALUES, scope.statics.array_values.clone()),
                (sym::AT, scope.statics.array_at.clone()),
                (sym::CONCAT, scope.statics.array_concat.clone()),
                (sym::ENTRIES, scope.statics.array_entries.clone()),
                (sym::KEYS, scope.statics.array_keys.clone()),
                (sym::EVERY, scope.statics.array_every.clone()),
                (sym::SOME, scope.statics.array_some.clone()),
                (sym::FILL, scope.statics.array_fill.clone()),
                (sym::FILTER, scope.statics.array_filter.clone()),
                (sym::REDUCE, scope.statics.array_reduce.clone()),
                (sym::FIND, scope.statics.array_find.clone()),
                (sym::FIND_INDEX, scope.statics.array_find_index.clone()),
                (sym::FLAT, scope.statics.array_flat.clone()),
                (sym::FOR_EACH, scope.statics.array_for_each.clone()),
                (sym::INCLUDES, scope.statics.array_includes.clone()),
                (sym::INDEX_OF, scope.statics.array_index_of.clone()),
                (sym::LO_MAP, scope.statics.array_map.clone()),
                (sym::POP, scope.statics.array_pop.clone()),
                (sym::PUSH, scope.statics.array_push.clone()),
                (sym::REVERSE, scope.statics.array_reverse.clone()),
                (sym::SHIFT, scope.statics.array_shift.clone()),
                (sym::SORT, scope.statics.array_sort.clone()),
                (sym::UNSHIFT, scope.statics.array_unshift.clone()),
                (sym::SLICE, scope.statics.array_slice.clone()),
                (sym::LAST_INDEX_OF, scope.statics.array_last_index_of.clone()),
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
                (sym::NEXT, scope.statics.array_iterator_next.clone()),
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
                (sym::NEXT, scope.statics.generator_iterator_next.clone()),
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
                (sym::ASYNC_ITERATOR,Value::Symbol( scope.statics.symbol_async_iterator.clone())),
                (sym::HAS_INSTANCE, Value::Symbol(scope.statics.symbol_has_instance.clone())),
                (sym::ITERATOR, Value::Symbol(scope.statics.symbol_iterator.clone())),
                (sym::MATCH, Value::Symbol(scope.statics.symbol_match.clone())),
                (sym::MATCH_ALL, Value::Symbol(scope.statics.symbol_match_all.clone())),
                (sym::REPLACE, Value::Symbol(scope.statics.symbol_replace.clone())),
                (sym::SEARCH, Value::Symbol(scope.statics.symbol_search.clone())),
                (sym::SPECIES, Value::Symbol(scope.statics.symbol_species.clone())),
                (sym::SPLIT, Value::Symbol(scope.statics.symbol_split.clone())),
                (sym::TO_PRIMITIVE, Value::Symbol(scope.statics.symbol_to_primitive.clone())),
                (sym::TO_STRING_TAG, Value::Symbol(scope.statics.symbol_to_string_tag.clone())),
                (sym::UNSCOPABLES, Value::Symbol(scope.statics.symbol_unscopables.clone())),
            ],
            Some((sym::SYMBOL, scope.statics.symbol_prototype.clone())),
            &mut scope,
        );
        
        let error_ctor = register(
            scope.statics.error_ctor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [],
            [],
            [],
            Some((sym::ERROR, scope.statics.error_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.error_prototype.clone(),
            object_proto.clone(),
            error_ctor.clone(),
            [
                (sym::TO_STRING, scope.statics.error_to_string.clone()),
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
            Some((sym::ARRAY_BUFFER, scope.statics.arraybuffer_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.arraybuffer_prototype.clone(),
            object_proto.clone(),
            arraybuffer_ctor.clone(),
            [
                (sym::BYTE_LENGTH, scope.statics.arraybuffer_byte_length.clone()) // TODO: should be a getter really
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
            Some((sym::UINT8ARRAY, scope.statics.uint8array_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.uint8array_prototype.clone(),
            object_proto.clone(),
            u8array_ctor.clone(),
            [
                (sym::FILL, scope.statics.typedarray_fill.clone()),
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
            Some((sym::INT8ARRAY, scope.statics.int8array_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.int8array_prototype.clone(),
            object_proto.clone(),
            i8array_ctor.clone(),
            [
                (sym::FILL, scope.statics.typedarray_fill.clone()),
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
            Some((sym::UINT16ARRAY, scope.statics.uint16array_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.uint16array_prototype.clone(),
            object_proto.clone(),
            u16array_ctor.clone(),
            [
                (sym::FILL, scope.statics.typedarray_fill.clone()),
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
            Some((sym::INT16ARRAY, scope.statics.int16array_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.int16array_prototype.clone(),
            object_proto.clone(),
            i16array_ctor.clone(),
            [
                (sym::FILL, scope.statics.typedarray_fill.clone()),
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
            Some((sym::UINT32ARRAY, scope.statics.uint32array_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.uint32array_prototype.clone(),
            object_proto.clone(),
            u32array_ctor.clone(),
            [
                (sym::FILL, scope.statics.typedarray_fill.clone()),
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
            Some((sym::INT32ARRAY, scope.statics.int32array_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.int32array_prototype.clone(),
            object_proto.clone(),
            i32array_ctor.clone(),
            [
                (sym::FILL, scope.statics.typedarray_fill.clone()),
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
            Some((sym::FLOAT32ARRAY, scope.statics.float32array_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.float32array_prototype.clone(),
            object_proto.clone(),
            f32array_ctor.clone(),
            [
                (sym::FILL, scope.statics.typedarray_fill.clone()),
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
            Some((sym::FLOAT64ARRAY, scope.statics.float64array_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.float64array_prototype.clone(),
            object_proto.clone(),
            f64array_ctor.clone(),
            [
                (sym::FILL, scope.statics.typedarray_fill.clone()),
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
                (sym::RESOLVE, scope.statics.promise_resolve.clone()),
                (sym::REJECT, scope.statics.promise_reject.clone()),
            ],
            [],
            [],
            Some((sym::PROMISE, scope.statics.promise_proto.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.promise_proto.clone(),
            object_proto.clone(),
            promise_ctor.clone(),
            [
                (sym::THEN, scope.statics.promise_then.clone()),
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
            Some((sym::SET, scope.statics.set_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.set_prototype.clone(),
            object_proto.clone(),
            set_ctor.clone(),
            [
                (sym::ADD, scope.statics.set_add.clone()),
                (sym::HAS, scope.statics.set_has.clone()),
                (sym::DELETE, scope.statics.set_delete.clone()),
                (sym::CLEAR, scope.statics.set_clear.clone()),
                (sym::SIZE, scope.statics.set_size.clone()),
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
            Some((sym::MAP, scope.statics.map_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.map_prototype.clone(),
            object_proto.clone(),
            map_ctor.clone(),
            [
                (sym::LO_SET, scope.statics.map_set.clone()),
                (sym::GET, scope.statics.map_get.clone()),
                (sym::HAS, scope.statics.map_has.clone()),
                (sym::DELETE, scope.statics.map_delete.clone()),
                (sym::CLEAR, scope.statics.map_clear.clone()),
                (sym::SIZE, scope.statics.map_size.clone()), // TODO: this should be a getter
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
            Some((sym::REG_EXP, scope.statics.regexp_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.regexp_prototype.clone(),
            object_proto.clone(),
            regexp_ctor.clone(),
            [
                (sym::TEST, scope.statics.regexp_test.clone()),
                (sym::EXEC, scope.statics.regexp_exec.clone())
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
            Some((sym::EVAL_ERROR, scope.statics.eval_error_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.eval_error_prototype.clone(),
            scope.statics.error_prototype.clone(),
            eval_error_ctor.clone(),
            [
                (sym::TO_STRING, scope.statics.error_to_string.clone()),
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
            Some((sym::RANGE_ERROR, scope.statics.range_error_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.range_error_prototype.clone(),
            scope.statics.error_prototype.clone(),
            range_error_ctor.clone(),
            [
                (sym::TO_STRING, scope.statics.error_to_string.clone()),
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
            Some((sym::REFERENCE_ERROR, scope.statics.reference_error_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.reference_error_prototype.clone(),
            scope.statics.error_prototype.clone(),
            reference_error_ctor.clone(),
            [
                (sym::TO_STRING, scope.statics.error_to_string.clone()),
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
            Some((sym::SYNTAX_ERROR, scope.statics.syntax_error_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.syntax_error_prototype.clone(),
            scope.statics.error_prototype.clone(),
            syntax_error_ctor.clone(),
            [
                (sym::TO_STRING, scope.statics.error_to_string.clone()),
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
            Some((sym::TYPE_ERROR, scope.statics.type_error_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.type_error_prototype.clone(),
            scope.statics.error_prototype.clone(),
            type_error_ctor.clone(),
            [
                (sym::TO_STRING, scope.statics.error_to_string.clone()),
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
            Some((sym::URI_ERROR, scope.statics.uri_error_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.uri_error_prototype.clone(),
            scope.statics.error_prototype.clone(),
            uri_error_ctor.clone(),
            [
                (sym::TO_STRING, scope.statics.error_to_string.clone()),
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
            Some((sym::AGGREGATE_ERROR, scope.statics.aggregate_error_prototype.clone())),
            &mut scope,
        );
        
        register(
            scope.statics.aggregate_error_prototype.clone(),
            scope.statics.error_prototype.clone(),
            aggregate_error_ctor.clone(),
            [
                (sym::TO_STRING, scope.statics.error_to_string.clone()),
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
                (sym::NOW, scope.statics.date_now.clone()),
            ],
            [],
            [],
            Some((sym::DATE, scope.statics.date_prototype.clone())),
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
            function_proto.clone(),
            function_ctor.clone(),
            [
                (sym::PARSE, scope.statics.json_parse.clone()),
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
                (sym::IS_NA_N, scope.statics.is_nan.clone()),
                (sym::IS_FINITE, scope.statics.is_finite.clone()),
                (sym::PARSE_FLOAT, scope.statics.parse_float.clone()),
                (sym::PARSE_INT, scope.statics.parse_int.clone()),
                (sym::REG_EXP, regexp_ctor.clone()),
                (sym::SYMBOL, symbol_ctor.clone()),
                (sym::DATE, date_ctor.clone()),
                (sym::ARRAY_BUFFER, arraybuffer_ctor.clone()),
                (sym::UINT8ARRAY, u8array_ctor.clone()),
                (sym::INT8ARRAY, i8array_ctor.clone()),
                (sym::UINT16ARRAY, u16array_ctor.clone()),
                (sym::INT16ARRAY, i16array_ctor.clone()),
                (sym::UINT32ARRAY, u32array_ctor.clone()),
                (sym::INT32ARRAY, i32array_ctor.clone()),
                (sym::FLOAT32ARRAY, f32array_ctor.clone()),
                (sym::FLOAT64ARRAY, f64array_ctor.clone()),
                (sym::ARRAY, array_ctor.clone()),
                (sym::ERROR, error_ctor.clone()),
                (sym::EVAL_ERROR, eval_error_ctor.clone()),
                (sym::RANGE_ERROR, range_error_ctor.clone()),
                (sym::REFERENCE_ERROR, reference_error_ctor.clone()),
                (sym::SYNTAX_ERROR, syntax_error_ctor.clone()),
                (sym::TYPE_ERROR, type_error_ctor.clone()),
                (sym::URI_ERROR, uri_error_ctor.clone()),
                (sym::AGGREGATE_ERROR, aggregate_error_ctor.clone()),
                (sym::STRING, string_ctor.clone()),
                (sym::OBJECT, object_ctor.clone()),
                (sym::SET, set_ctor.clone()),
                (sym::MAP, map_ctor.clone()),
                (sym::CONSOLE, console.clone()),
                (sym::MATH, math.clone()),
                (sym::NUMBER, number_ctor.clone()),
                (sym::BOOLEAN, boolean_ctor.clone()),
                (sym::PROMISE, promise_ctor.clone()),
                (sym::JSON, json_ctor.clone()),
            ],
            [],
            [],
            None,
            &mut scope
        );
    }

    /// Fetches the current instruction/value in the currently executing frame
    /// and increments the instruction pointer
    pub(crate) fn fetch_and_inc_ip(&mut self) -> u8 {
        let frame = self.frames.last_mut().expect("No frame");
        let ip = frame.ip;
        frame.ip += 1;
        frame.function.buffer.with(|buf| buf[ip])
    }

    /// Fetches a wide value (16-bit) in the currently executing frame
    /// and increments the instruction pointer
    pub(crate) fn fetchw_and_inc_ip(&mut self) -> u16 {
        let frame = self.frames.last_mut().expect("No frame");
        let value: [u8; 2] = frame.function.buffer.with(|buf| {
            buf[frame.ip..frame.ip + 2]
                .try_into()
                .expect("Failed to get wide instruction")
        });

        frame.ip += 2;
        u16::from_ne_bytes(value)
    }

    pub(crate) fn get_frame_sp(&self) -> usize {
        self.frames.last().map(|frame| frame.sp).expect("No frame")
    }

    pub(crate) fn get_local(&self, id: usize) -> Option<Value> {
        self.stack.get(self.get_frame_sp() + id).cloned()
    }

    pub(crate) fn get_external(&self, id: usize) -> Option<&Handle<ExternalValue>> {
        self.frames.last()?.externals.get(id)
    }

    pub(crate) fn set_local(&mut self, id: usize, value: Unrooted) {
        let sp = self.get_frame_sp();
        let idx = sp + id;

        // SAFETY: GC cannot trigger here
        // and value will become a root here, therefore this is ok
        let value = unsafe { value.into_value() };

        if let Value::External(o) = self.stack[idx].clone() {
            let value = value.into_gc_vm(self);
            unsafe { ExternalValue::replace(&o, value) };
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
            throw!(&mut self.scope(), RangeError, "Maximum call stack size exceeded");
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
            throw!(&mut self.scope(), RangeError, "Maximum stack size exceeded");
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
        if let Some(last) = self.try_blocks.last() {
            // if we're in a try-catch block, we need to jump to it
            let try_fp = last.frame_ip;
            let catch_ip = last.catch_ip;

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

            let frame = self.frames.last_mut().expect("No frame");
            frame.ip = catch_ip;

            let catch_ip = self.fetchw_and_inc_ip();
            if catch_ip != u16::MAX {
                // u16::MAX is used to indicate that there is no variable binding in the catch block
                self.set_local(catch_ip as usize, err);
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
            match v {
                Value::Object(o) => println!("{:#?}", &**o),
                Value::External(o) => println!("[[external]]: {:#?}", &*o.inner),
                _ => println!("{v:?}"),
            }
        }
    }

    /// Adds a function to the async task queue.
    pub fn add_async_task(&mut self, fun: Handle<dyn Object>) {
        self.async_tasks.push(fun);
    }

    pub fn has_async_tasks(&self) -> bool {
        !self.async_tasks.is_empty()
    }

    /// Processes all queued async tasks
    pub fn process_async_tasks(&mut self) {
        debug!("process async tasks");
        debug!(async_task_count = %self.async_tasks.len());

        while !self.async_tasks.is_empty() {
            let tasks = mem::take(&mut self.async_tasks);

            let mut scope = self.scope();

            for task in tasks {
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
                if util::unlikely(self.gc.node_count() > self.gc_object_threshold) {
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

    pub fn perform_gc(&mut self) {
        debug!("gc cycle triggered");

        let trace_roots = span!(Level::TRACE, "gc trace");
        trace_roots.in_scope(|| self.trace_roots());

        // All reachable roots are marked.
        debug!("object count before sweep: {}", self.gc.node_count());
        let sweep = span!(Level::TRACE, "gc sweep");
        sweep.in_scope(|| unsafe { self.gc.sweep() });
        debug!("object count after sweep: {}", self.gc.node_count());

        // Adjust GC threshold
        let new_object_count = self.gc.node_count();
        self.gc_object_threshold = new_object_count * 2;
        debug!("new threshold: {}", self.gc_object_threshold);
    }

    fn trace_roots(&mut self) {
        let mut cx = TraceCtxt::new(&mut self.interner);
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

        debug!("trace externals");
        // we do two things here:
        // remove Handles from external refs set that have a zero refcount (implying no active persistent refs)
        // and trace if refcount > 0
        self.external_refs.retain(|e| {
            let refcount = unsafe { (*e.as_ptr()).refcount.get() };
            if refcount == 0 {
                false
            } else {
                // Non-zero refcount, retain object and trace
                e.trace(&mut cx);
                true
            }
        });

        debug!("trace statics");
        self.statics.trace(&mut cx);
    }

    pub fn statics(&self) -> &Statics {
        &self.statics
    }

    pub fn gc_mut(&mut self) -> &mut Gc {
        &mut self.gc
    }

    // TODO: remove this function at all costs, this should never be called.
    // Always call `register` on local scope
    // Or, rather, return Unrooted
    pub fn register<O: Object + 'static>(&mut self, obj: O) -> Handle<dyn Object> {
        self.gc.register(obj)
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
        self.frames.last().unwrap().function.poison_ip(ip);
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
