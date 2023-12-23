#![warn(clippy::redundant_clone)]

use std::ops::RangeBounds;
use std::vec::Drain;
use std::{fmt, mem};

use crate::gc::trace::Trace;
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
        fn set_fn_prototype(v: &dyn Object, proto: &Handle<dyn Object>, name: &str) {
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
            methods: impl IntoIterator<Item = (&'static str, Handle<dyn Object>)>,
            symbols: impl IntoIterator<Item = (Symbol, Handle<dyn Object>)>,
            fields: impl IntoIterator<Item = (&'static str, Value)>,
            // Contrary to `prototype`, this optionally sets the function prototype. Should only be `Some`
            // when base is a function
            fn_prototype: Option<(&'static str, Handle<dyn Object>)>,

            // LocalScope needs to be the last parameter because we don't have two phase borrows in user code
            scope: &mut LocalScope<'_>,
        ) -> Handle<dyn Object> {
            base.set_property(scope, "constructor".into(), PropertyValue::static_default(constructor.into())).unwrap();
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
            Some(("Function", scope.statics.function_proto.clone())),
            &mut scope,
        );

        let function_proto = register(
            scope.statics.function_proto.clone(),
            scope.statics.object_prototype.clone(),
            function_ctor.clone(),
            [
                ("bind", scope.statics.function_bind.clone()),
                ("call", scope.statics.function_call.clone()),
                ("toString", scope.statics.function_to_string.clone()),
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
                ("create", scope.statics.object_create.clone()),
                ("keys", scope.statics.object_keys.clone()),
                ("getOwnPropertyDescriptor", scope.statics.object_get_own_property_descriptor.clone()),
                ("getOwnPropertyDescriptors", scope.statics.object_get_own_property_descriptors.clone()),
                ("defineProperty", scope.statics.object_define_property.clone()),
                ("entries", scope.statics.object_entries.clone()),
                ("assign", scope.statics.object_assign.clone()),
            ],
            [],
            [],
            Some(("Object", scope.statics.object_prototype.clone())),
            &mut scope,
        );

        let object_proto = register(
            scope.statics.object_prototype.clone(),
            Value::null(),
            object_ctor.clone(),
            [
                ("toString", scope.statics.object_to_string.clone()),
                ("hasOwnProperty", scope.statics.object_has_own_property.clone()),
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
                ("log", scope.statics.console_log.clone()),
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
                ("floor", scope.statics.math_floor.clone()),
                ("abs", scope.statics.math_abs.clone()),
                ("acos", scope.statics.math_acos.clone()),
                ("acosh", scope.statics.math_acosh.clone()),
                ("asin", scope.statics.math_asin.clone()),
                ("asinh", scope.statics.math_asinh.clone()),
                ("atan", scope.statics.math_atan.clone()),
                ("atanh", scope.statics.math_atanh.clone()),
                ("atan2", scope.statics.math_atan2.clone()),
                ("cbrt", scope.statics.math_cbrt.clone()),
                ("ceil", scope.statics.math_ceil.clone()),
                ("clz32", scope.statics.math_clz32.clone()),
                ("cos", scope.statics.math_cos.clone()),
                ("cosh", scope.statics.math_cosh.clone()),
                ("exp", scope.statics.math_exp.clone()),
                ("expm1", scope.statics.math_expm1.clone()),
                ("log", scope.statics.math_log.clone()),
                ("log1p", scope.statics.math_log1p.clone()),
                ("log10", scope.statics.math_log10.clone()),
                ("log2", scope.statics.math_log2.clone()),
                ("round", scope.statics.math_round.clone()),
                ("sin", scope.statics.math_sin.clone()),
                ("sinh", scope.statics.math_sinh.clone()),
                ("sqrt", scope.statics.math_sqrt.clone()),
                ("tan", scope.statics.math_tan.clone()),
                ("tanh", scope.statics.math_tanh.clone()),
                ("trunc", scope.statics.math_trunc.clone()),
                ("random", scope.statics.math_random.clone()),
                ("max", scope.statics.math_max.clone()),
                ("min", scope.statics.math_min.clone()),
            ],
            [],
            [
                ("PI", Value::number(std::f64::consts::PI)),
            ],
            None,
            &mut scope,
        );

        let number_ctor = register(
            scope.statics.number_ctor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [
                ("isFinite", scope.statics.number_is_finite.clone()),
                ("isNaN", scope.statics.number_is_nan.clone()),
                ("isSafeInteger", scope.statics.number_is_safe_integer.clone()),
            ],
            [],
            [],
            Some(("Number", scope.statics.number_prototype.clone())),
            &mut scope,
        );

        register(
            scope.statics.number_prototype.clone(),
            object_proto.clone(),
            number_ctor.clone(),
            [
                ("toString", scope.statics.number_tostring.clone()),
                ("toFixed", scope.statics.number_to_fixed.clone()),
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
            Some(("Boolean", scope.statics.boolean_prototype.clone())),
            &mut scope,
        );

        register(
            scope.statics.boolean_prototype.clone(),
            object_proto.clone(),
            boolean_ctor.clone(),
            [
                ("toString", scope.statics.boolean_tostring.clone()),
                ("valueOf", scope.statics.boolean_valueof.clone()),
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
                ("fromCharCode", scope.statics.string_from_char_code.clone()),
            ],
            [],
            [],
            Some(("String", scope.statics.string_prototype.clone())),
            &mut scope,
        );

        register(
           scope.statics.string_prototype.clone(),
           scope.statics.object_prototype.clone(),
           scope.statics.string_ctor.clone(),
           [
                ("toString", scope.statics.string_tostring.clone()),
                ("charAt", scope.statics.string_char_at.clone()),
                ("charCodeAt", scope.statics.string_char_code_at.clone()),
                ("concat", scope.statics.string_concat.clone()),
                ("endsWith", scope.statics.string_ends_with.clone()),
                ("startsWith", scope.statics.string_starts_with.clone()),
                ("includes", scope.statics.string_includes.clone()),
                ("indexOf", scope.statics.string_index_of.clone()),
                ("lastIndexOf", scope.statics.string_last_index_of.clone()),
                ("padEnd", scope.statics.string_pad_end.clone()),
                ("padStart", scope.statics.string_pad_start.clone()),
                ("repeat", scope.statics.string_repeat.clone()),
                ("replace", scope.statics.string_replace.clone()),
                ("replaceAll", scope.statics.string_replace_all.clone()),
                ("split", scope.statics.string_split.clone()),
                ("toLowerCase", scope.statics.string_to_lowercase.clone()),
                ("toUpperCase", scope.statics.string_to_uppercase.clone()),
                ("big", scope.statics.string_big.clone()),
                ("blink", scope.statics.string_blink.clone()),
                ("bold", scope.statics.string_bold.clone()),
                ("fixed", scope.statics.string_fixed.clone()),
                ("italics", scope.statics.string_italics.clone()),
                ("strike", scope.statics.string_strike.clone()),
                ("sub", scope.statics.string_sub.clone()),
                ("sup", scope.statics.string_sup.clone()),
                ("fontcolor", scope.statics.string_fontcolor.clone()),
                ("fontsize", scope.statics.string_fontsize.clone()),
                ("link", scope.statics.string_link.clone()),
                ("trim", scope.statics.string_trim.clone()),
                ("trimStart", scope.statics.string_trim_start.clone()),
                ("trimEnd", scope.statics.string_trim_end.clone()),
                ("substr", scope.statics.string_substr.clone()),
                ("substring", scope.statics.string_substring.clone()),
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
                ("from", scope.statics.array_from.clone()),
                ("isArray", scope.statics.array_is_array.clone()),
            ],
            [],
            [],
            Some(("Array", scope.statics.array_prototype.clone())),
            &mut scope,
        );

        register(
            scope.statics.array_prototype.clone(),
            object_proto.clone(),
            array_ctor.clone(),
            [
                ("toString", scope.statics.array_tostring.clone()),
                ("join", scope.statics.array_join.clone()),
                ("values", scope.statics.array_values.clone()),
                ("at", scope.statics.array_at.clone()),
                ("concat", scope.statics.array_concat.clone()),
                ("entries", scope.statics.array_entries.clone()),
                ("keys", scope.statics.array_keys.clone()),
                ("every", scope.statics.array_every.clone()),
                ("some", scope.statics.array_some.clone()),
                ("fill", scope.statics.array_fill.clone()),
                ("filter", scope.statics.array_filter.clone()),
                ("reduce", scope.statics.array_reduce.clone()),
                ("find", scope.statics.array_find.clone()),
                ("findIndex", scope.statics.array_find_index.clone()),
                ("flat", scope.statics.array_flat.clone()),
                ("forEach", scope.statics.array_for_each.clone()),
                ("includes", scope.statics.array_includes.clone()),
                ("indexOf", scope.statics.array_index_of.clone()),
                ("map", scope.statics.array_map.clone()),
                ("pop", scope.statics.array_pop.clone()),
                ("push", scope.statics.array_push.clone()),
                ("reverse", scope.statics.array_reverse.clone()),
                ("shift", scope.statics.array_shift.clone()),
                ("sort", scope.statics.array_sort.clone()),
                ("unshift", scope.statics.array_unshift.clone()),
                ("slice", scope.statics.array_slice.clone()),
                ("lastIndexOf", scope.statics.array_last_index_of.clone()),
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
                ("next", scope.statics.array_iterator_next.clone()),
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
                ("next", scope.statics.generator_iterator_next.clone()),
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
                ("asyncIterator",Value::Symbol( scope.statics.symbol_async_iterator.clone())),
                ("hasInstance", Value::Symbol(scope.statics.symbol_has_instance.clone())),
                ("iterator", Value::Symbol(scope.statics.symbol_iterator.clone())),
                ("match", Value::Symbol(scope.statics.symbol_match.clone())),
                ("matchAll", Value::Symbol(scope.statics.symbol_match_all.clone())),
                ("replace", Value::Symbol(scope.statics.symbol_replace.clone())),
                ("search", Value::Symbol(scope.statics.symbol_search.clone())),
                ("species", Value::Symbol(scope.statics.symbol_species.clone())),
                ("split", Value::Symbol(scope.statics.symbol_split.clone())),
                ("toPrimitive", Value::Symbol(scope.statics.symbol_to_primitive.clone())),
                ("toStringTag", Value::Symbol(scope.statics.symbol_to_string_tag.clone())),
                ("unscopables", Value::Symbol(scope.statics.symbol_unscopables.clone())),
            ],
            Some(("Symbol", scope.statics.symbol_prototype.clone())),
            &mut scope,
        );

        let error_ctor = register(
            scope.statics.error_ctor.clone(),
            function_proto.clone(),
            function_ctor.clone(),
            [],
            [],
            [],
            Some(("Error", scope.statics.error_prototype.clone())),
            &mut scope,
        );

        register(
            scope.statics.error_prototype.clone(),
            object_proto.clone(),
            error_ctor.clone(),
            [
                ("toString", scope.statics.error_to_string.clone()),
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
            Some(("ArrayBuffer", scope.statics.arraybuffer_prototype.clone())),
            &mut scope,
        );

        register(
            scope.statics.arraybuffer_prototype.clone(),
            object_proto.clone(),
            arraybuffer_ctor.clone(),
            [
                ("byteLength", scope.statics.arraybuffer_byte_length.clone()) // TODO: should be a getter really
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
            Some(("Uint8Array", scope.statics.uint8array_prototype.clone())),
            &mut scope,
        );

        register(
            scope.statics.uint8array_prototype.clone(),
            object_proto.clone(),
            u8array_ctor.clone(),
            [
                ("fill", scope.statics.typedarray_fill.clone()),
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
            Some(("Int8Array", scope.statics.int8array_prototype.clone())),
            &mut scope,
        );

        register(
            scope.statics.int8array_prototype.clone(),
            object_proto.clone(),
            i8array_ctor.clone(),
            [
                ("fill", scope.statics.typedarray_fill.clone()),
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
            Some(("Uint16Array", scope.statics.uint16array_prototype.clone())),
            &mut scope,
        );

        register(
            scope.statics.uint16array_prototype.clone(),
            object_proto.clone(),
            u16array_ctor.clone(),
            [
                ("fill", scope.statics.typedarray_fill.clone()),
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
            Some(("Int16Array", scope.statics.int16array_prototype.clone())),
            &mut scope,
        );

        register(
            scope.statics.int16array_prototype.clone(),
            object_proto.clone(),
            i16array_ctor.clone(),
            [
                ("fill", scope.statics.typedarray_fill.clone()),
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
            Some(("Uint32Array", scope.statics.uint32array_prototype.clone())),
            &mut scope,
        );

        register(
            scope.statics.uint32array_prototype.clone(),
            object_proto.clone(),
            u32array_ctor.clone(),
            [
                ("fill", scope.statics.typedarray_fill.clone()),
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
            Some(("Int32Array", scope.statics.int32array_prototype.clone())),
            &mut scope,
        );

        register(
            scope.statics.int32array_prototype.clone(),
            object_proto.clone(),
            i32array_ctor.clone(),
            [
                ("fill", scope.statics.typedarray_fill.clone()),
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
            Some(("Float32Array", scope.statics.float32array_prototype.clone())),
            &mut scope,
        );

        register(
            scope.statics.float32array_prototype.clone(),
            object_proto.clone(),
            f32array_ctor.clone(),
            [
                ("fill", scope.statics.typedarray_fill.clone()),
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
            Some(("Float64Array", scope.statics.float64array_prototype.clone())),
            &mut scope,
        );

        register(
            scope.statics.float64array_prototype.clone(),
            object_proto.clone(),
            f64array_ctor.clone(),
            [
                ("fill", scope.statics.typedarray_fill.clone()),
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
                ("resolve", scope.statics.promise_resolve.clone()),
                ("reject", scope.statics.promise_reject.clone()),
            ],
            [],
            [],
            Some(("Promise", scope.statics.promise_proto.clone())),
            &mut scope,
        );

        register(
            scope.statics.promise_proto.clone(),
            object_proto.clone(),
            promise_ctor.clone(),
            [
                ("then", scope.statics.promise_then.clone()),
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
            Some(("Set", scope.statics.set_prototype.clone())),
            &mut scope,
        );

        register(
            scope.statics.set_prototype.clone(),
            object_proto.clone(),
            set_ctor.clone(),
            [
                ("add", scope.statics.set_add.clone()),
                ("has", scope.statics.set_has.clone()),
                ("delete", scope.statics.set_delete.clone()),
                ("clear", scope.statics.set_clear.clone()),
                ("size", scope.statics.set_size.clone()),
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
            Some(("Map", scope.statics.map_prototype.clone())),
            &mut scope,
        );

        register(
            scope.statics.map_prototype.clone(),
            object_proto.clone(),
            map_ctor.clone(),
            [
                ("set", scope.statics.map_set.clone()),
                ("get", scope.statics.map_get.clone()),
                ("has", scope.statics.map_has.clone()),
                ("delete", scope.statics.map_delete.clone()),
                ("clear", scope.statics.map_clear.clone()),
                ("size", scope.statics.map_size.clone()), // TODO: this should be a getter
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
            Some(("RegExp", scope.statics.regexp_prototype.clone())),
            &mut scope,
        );

        register(
            scope.statics.regexp_prototype.clone(),
            object_proto.clone(),
            regexp_ctor.clone(),
            [
                ("test", scope.statics.regexp_test.clone()),
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
            Some(("EvalError", scope.statics.eval_error_prototype.clone())),
            &mut scope,
        );

        register(
            scope.statics.eval_error_prototype.clone(),
            scope.statics.error_prototype.clone(),
            eval_error_ctor.clone(),
            [
                ("toString", scope.statics.error_to_string.clone()),
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
            Some(("RangeError", scope.statics.range_error_prototype.clone())),
            &mut scope,
        );

        register(
            scope.statics.range_error_prototype.clone(),
            scope.statics.error_prototype.clone(),
            range_error_ctor.clone(),
            [
                ("toString", scope.statics.error_to_string.clone()),
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
            Some(("ReferenceError", scope.statics.reference_error_prototype.clone())),
            &mut scope,
        );

        register(
            scope.statics.reference_error_prototype.clone(),
            scope.statics.error_prototype.clone(),
            reference_error_ctor.clone(),
            [
                ("toString", scope.statics.error_to_string.clone()),
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
            Some(("SyntaxError", scope.statics.syntax_error_prototype.clone())),
            &mut scope,
        );

        register(
            scope.statics.syntax_error_prototype.clone(),
            scope.statics.error_prototype.clone(),
            syntax_error_ctor.clone(),
            [
                ("toString", scope.statics.error_to_string.clone()),
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
            Some(("TypeError", scope.statics.type_error_prototype.clone())),
            &mut scope,
        );

        register(
            scope.statics.type_error_prototype.clone(),
            scope.statics.error_prototype.clone(),
            type_error_ctor.clone(),
            [
                ("toString", scope.statics.error_to_string.clone()),
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
            Some(("URIError", scope.statics.uri_error_prototype.clone())),
            &mut scope,
        );

        register(
            scope.statics.uri_error_prototype.clone(),
            scope.statics.error_prototype.clone(),
            uri_error_ctor.clone(),
            [
                ("toString", scope.statics.error_to_string.clone()),
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
            Some(("AggregateError", scope.statics.aggregate_error_prototype.clone())),
            &mut scope,
        );

        register(
            scope.statics.aggregate_error_prototype.clone(),
            scope.statics.error_prototype.clone(),
            aggregate_error_ctor.clone(),
            [
                ("toString", scope.statics.error_to_string.clone()),
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
                ("now", scope.statics.date_now.clone()),
            ],
            [],
            [],
            Some(("Date", scope.statics.date_prototype.clone())),
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
                ("parse", scope.statics.json_parse.clone()),
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
                ("isNaN", scope.statics.is_nan.clone()),
                ("isFinite", scope.statics.is_finite.clone()),
                ("parseFloat", scope.statics.parse_float.clone()),
                ("parseInt", scope.statics.parse_int.clone()),
                ("RegExp", regexp_ctor.clone()),
                ("Symbol", symbol_ctor.clone()),
                ("Date", date_ctor.clone()),
                ("ArrayBuffer", arraybuffer_ctor.clone()),
                ("Uint8Array", u8array_ctor.clone()),
                ("Int8Array", i8array_ctor.clone()),
                ("Uint16Array", u16array_ctor.clone()),
                ("Int16Array", i16array_ctor.clone()),
                ("Uint32Array", u32array_ctor.clone()),
                ("Int32Array", i32array_ctor.clone()),
                ("Float32Array", f32array_ctor.clone()),
                ("Float64Array", f64array_ctor.clone()),
                ("Array", array_ctor.clone()),
                ("Error", error_ctor.clone()),
                ("EvalError", eval_error_ctor.clone()),
                ("RangeError", range_error_ctor.clone()),
                ("ReferenceError", reference_error_ctor.clone()),
                ("SyntaxError", syntax_error_ctor.clone()),
                ("TypeError", type_error_ctor.clone()),
                ("URIError", uri_error_ctor.clone()),
                ("AggregateError", aggregate_error_ctor.clone()),
                ("String", string_ctor.clone()),
                ("Object", object_ctor.clone()),
                ("Set", set_ctor.clone()),
                ("Map", map_ctor.clone()),
                ("console", console.clone()),
                ("Math", math.clone()),
                ("Number", number_ctor.clone()),
                ("Boolean", boolean_ctor.clone()),
                ("Promise", promise_ctor.clone()),
                ("JSON", json_ctor.clone()),
            ],
            [],
            [],
            None,
            &mut scope
        );

        scope.builtins_pure = true;
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
            throw!(self, RangeError, "Maximum call stack size exceeded");
        }
        Ok(())
    }

    pub(crate) fn try_extend_stack<I>(&mut self, other: I) -> Result<(), Value>
    where
        I: IntoIterator<Item = Value>,
        <I as IntoIterator>::IntoIter: ExactSizeIterator,
    {
        let it = other.into_iter();
        let len = it.len();
        if self.stack.len() + len > MAX_STACK_SIZE {
            debug!("vm exceeded stack size");
            throw!(self, RangeError, "Maximum stack size exceeded");
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
        debug!("trace frames");
        self.frames.trace();
        debug!("trace async tasks");
        self.async_tasks.trace();
        debug!("trace stack");
        self.stack.trace();
        debug!("trace globals");
        self.global.trace();
        debug!("trace scopes");
        self.scopes.trace();

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
                e.trace();
                true
            }
        });

        debug!("trace statics");
        self.statics.trace();
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
