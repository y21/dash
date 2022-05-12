use std::{convert::TryInto, fmt, ops::RangeBounds, vec::Drain};

use crate::{
    compiler::FunctionCompiler,
    gc::{handle::Handle, trace::Trace, Gc},
    optimizer::{self, consteval::OptLevel},
    parser::parser::Parser,
    vm::value::function::Function,
    EvalError, throw,
};

use self::{
    dispatch::HandleResult,
    external::Externals,
    frame::{Exports, Frame, FrameState, TryBlock},
    local::LocalScope,
    params::VmParams,
    statics::Statics,
    value::{
        object::{NamedObject, Object},
        Value,
    },
};

pub mod dispatch;
pub mod external;
pub mod frame;
pub mod local;
pub mod params;
pub mod statics;
pub mod util;
pub mod value;

pub const MAX_FRAME_STACK_SIZE: usize = 1024;
pub const MAX_STACK_SIZE: usize = 8196;

pub struct Vm {
    frames: Vec<Frame>,
    stack: Vec<Value>,
    gc: Gc<dyn Object>,
    global: Handle<dyn Object>,
    externals: Externals,
    statics: Statics, // TODO: we should box this... maybe?
    try_blocks: Vec<TryBlock>,
    params: VmParams,
}

impl Vm {
    pub fn new(params: VmParams) -> Self {
        let mut gc = Gc::new();
        let statics = Statics::new(&mut gc);
        let global = gc.register(NamedObject::null()); // TODO: set its __proto__ and constructor

        let mut vm = Self {
            frames: Vec::new(),
            stack: Vec::with_capacity(512),
            gc,
            global,
            externals: Externals::new(),
            statics,
            try_blocks: Vec::new(),
            params,
        };
        vm.prepare();
        vm
    }

    pub fn eval<'a>(&mut self, input: &'a str, opt: OptLevel) -> Result<Value, EvalError<'a>> {
        let mut ast = Parser::from_str(input)
            .map_err(EvalError::LexError)?
            .parse_all()
            .map_err(EvalError::ParseError)?;

        optimizer::optimize_ast(&mut ast, opt);

        let compiled = FunctionCompiler::new()
            .compile_ast(ast)
            .map_err(EvalError::CompileError)?;

        let frame = Frame::from_compile_result(compiled);
        let val = self.execute_frame(frame).map_err(EvalError::VmError)?;
        Ok(val.into_value())
    }

    /// Prepare the VM for execution.
    #[rustfmt::skip]
    fn prepare(&mut self) {
        fn set_fn_prototype(v: &dyn Object, proto: &Handle<dyn Object>) {
            let fun = v.as_any().downcast_ref::<Function>().unwrap();
            fun.set_fn_prototype(proto.clone());
        }

        let mut scope = LocalScope::new(self);

        let global = scope.global.clone();

        /// #[prototype] - Internal [[Prototype]] field for this value
        /// #[fn_prototype] - Only valid on function values
        ///                   This will set the [[Prototype]] field of the function
        /// #[properties] - "Reference" properties (i.e. object of some kind)
        /// #[symbols] - Symbol properties (e.g. @@iterator)
        /// #[fields] - Primitive fields (e.g. PI: 3.1415)
        macro_rules! register_builtin_type {
            (
                $base:expr, {
                    #[prototype] $prototype:expr;
                    $( #[fn_prototype] $fnprototype:expr; )?
                    $( #[properties] $( $prop:ident: $prop_path:expr; )+ )?
                    $( #[symbols] $( $symbol:expr => $symbol_path:expr; )+ )?
                    $( #[fields] $( $field:ident: $value:expr; )+ )?
                }
            ) => {{
                let base = $base.clone();

                // Prototype
                {
                    let proto = $prototype.clone();
                    base.set_prototype(&mut scope, proto.into()).unwrap();
                }

                // Properties
                $(
                    $({
                        let method = stringify!($prop);
                        let path = $prop_path.clone();
                        register_builtin_type!(path, { #[prototype] scope.statics.function_proto; });
                        base.set_property(&mut scope, method.into(), path.into()).unwrap();
                    })+
                )?

                // Symbols
                $(
                    $({
                        let method = $symbol.clone();
                        let path = $symbol_path.clone();
                        register_builtin_type!(path, { #[prototype] scope.statics.function_proto; });
                        base.set_property(&mut scope, method.into(), path.into()).unwrap();
                    })+
                )?

                // Fields
                $(
                    $({
                        let method = stringify!($field);
                        let value = $value.clone();
                        base.set_property(&mut scope, method.into(), value.into()).unwrap();
                    })+
                )?

                // Function prototype
                $(
                    set_fn_prototype(&base, &$fnprototype);
                )?

                base
            }}            
        }

        register_builtin_type!(scope.statics.function_proto, {
            #[prototype] scope.statics.object_prototype.clone();
        });

        let object_ctor = register_builtin_type!(scope.statics.object_ctor, {
            #[prototype] scope.statics.function_proto;
            #[fn_prototype] scope.statics.object_prototype;

            #[properties]
            create: scope.statics.object_create;
            keys: scope.statics.object_keys;
        });

        let object_proto = register_builtin_type!(scope.statics.object_prototype, {
            #[prototype] Value::null();

            #[properties]
            toString: scope.statics.object_to_string;
        });

        let console = register_builtin_type!(scope.statics.console, {
            #[prototype] scope.statics.object_prototype;

            #[properties]
            log: scope.statics.console_log;
        });

        let math = register_builtin_type!(scope.statics.math, {
            #[prototype] scope.statics.object_prototype;

            #[properties]
            floor: scope.statics.math_floor;
            abs: scope.statics.math_abs;
            acos: scope.statics.math_acos;
            acosh: scope.statics.math_acosh;
            asin: scope.statics.math_asin;
            asinh: scope.statics.math_asinh;
            atan: scope.statics.math_atan;
            atanh: scope.statics.math_atanh;
            atan2: scope.statics.math_atan2;
            cbrt: scope.statics.math_cbrt;
            ceil: scope.statics.math_ceil;
            clz32: scope.statics.math_clz32;
            cos: scope.statics.math_cos;
            cosh: scope.statics.math_cosh;
            exp: scope.statics.math_exp;
            expm1: scope.statics.math_expm1;
            log: scope.statics.math_log;
            log1p: scope.statics.math_log1p;
            log10: scope.statics.math_log10;
            log2: scope.statics.math_log2;
            round: scope.statics.math_round;
            sin: scope.statics.math_sin;
            sinh: scope.statics.math_sinh;
            sqrt: scope.statics.math_sqrt;
            tan: scope.statics.math_tan;
            tanh: scope.statics.math_tanh;
            trunc: scope.statics.math_trunc;
            random: scope.statics.math_random;

            #[fields]
            PI: Value::Number(std::f64::consts::PI);
        });

        let number_ctor = register_builtin_type!(scope.statics.number_ctor, {
            #[prototype] scope.statics.function_proto;
            #[fn_prototype] scope.statics.number_prototype;

            #[properties]
            isFinite: scope.statics.number_is_finite;
            isNaN: scope.statics.number_is_nan;
            isSafeInteger: scope.statics.number_is_safe_integer;
        });

        let number_proto = register_builtin_type!(scope.statics.number_prototype, {
            #[prototype] scope.statics.object_prototype;
            
            #[properties]
            toString: scope.statics.number_tostring;
            toFixed: scope.statics.number_to_fixed;
        });

        let boolean_ctor = register_builtin_type!(scope.statics.boolean_ctor, {
            #[prototype] scope.statics.function_proto;
            #[fn_prototype] scope.statics.boolean_prototype;
        });

        let boolean_proto = register_builtin_type!(scope.statics.boolean_prototype, {
            #[prototype] scope.statics.object_prototype;

            #[properties]
            toString: scope.statics.boolean_tostring;
            valueOf: scope.statics.boolean_valueof;
        });

        let string_ctor = register_builtin_type!(scope.statics.string_ctor, {
            #[prototype] scope.statics.function_proto;
            #[fn_prototype] scope.statics.string_prototype;
        });
        
        let string_prototype = register_builtin_type!(scope.statics.string_prototype, {
            #[prototype] scope.statics.object_prototype;

            #[properties]
            toString: scope.statics.string_tostring;
            charAt: scope.statics.string_char_at;
            charCodeAt: scope.statics.string_char_code_at;
            concat: scope.statics.string_concat;
            endsWith: scope.statics.string_ends_with;
            startsWith: scope.statics.string_starts_with;
            includes: scope.statics.string_includes;
            indexOf: scope.statics.string_index_of;
            lastIndexOf: scope.statics.string_last_index_of;
            padEnd: scope.statics.string_pad_end;
            padStart: scope.statics.string_pad_start;
            repeat: scope.statics.string_repeat;
            replace: scope.statics.string_replace;
            replaceAll: scope.statics.string_replace_all;
            split: scope.statics.string_split;
            toLowerCase: scope.statics.string_to_lowercase;
            toUpperCase: scope.statics.string_to_uppercase;
        });

        let array_ctor = register_builtin_type!(scope.statics.array_ctor, {
            #[prototype] scope.statics.function_proto;
            #[fn_prototype] scope.statics.array_prototype;
        });
        
        let array_proto = register_builtin_type!(scope.statics.array_prototype, {
            #[prototype] scope.statics.object_prototype;

            #[properties]
            toString: scope.statics.array_tostring;
            join: scope.statics.array_join;
            values: scope.statics.array_values;
            at: scope.statics.array_at;
            concat: scope.statics.array_concat;
            entries: scope.statics.array_entries;
            keys: scope.statics.array_keys;
            every: scope.statics.array_every;
            fill: scope.statics.array_fill;
            filter: scope.statics.array_filter;
            find: scope.statics.array_find;
            findIndex: scope.statics.array_find_index;
            flat: scope.statics.array_flat;
            forEach: scope.statics.array_for_each;
            includes: scope.statics.array_includes;
            indexOf: scope.statics.array_index_of;
            map: scope.statics.array_map;
            pop: scope.statics.array_pop;
            push: scope.statics.array_push;

            #[symbols]
            scope.statics.symbol_iterator => scope.statics.array_values;
        });

        let array_iterator_proto = register_builtin_type!(scope.statics.array_iterator_prototype, {
            #[prototype] scope.statics.object_prototype; // TODO: this is incorrect

            #[properties]
            next: scope.statics.array_iterator_next;
        });

        let generator_iterator_proto = register_builtin_type!(scope.statics.generator_iterator_prototype, {
            #[prototype] scope.statics.object_prototype; // TODO: this is incorrect

            #[properties]
            next: scope.statics.generator_iterator_next;
        });

        let symbol_ctor = register_builtin_type!(scope.statics.symbol_ctor, {
            #[prototype] scope.statics.function_proto;
            #[fn_prototype] scope.statics.symbol_prototype;

            #[properties]
            asyncIterator: scope.statics.symbol_async_iterator;
            hasInstance: scope.statics.symbol_has_instance;
            iterator: scope.statics.symbol_iterator;
            match: scope.statics.symbol_match;
            matchAll: scope.statics.symbol_match_all;
            replace: scope.statics.symbol_replace;
            search: scope.statics.symbol_search;
            species: scope.statics.symbol_species;
            split: scope.statics.symbol_split;
            toPrimitive: scope.statics.symbol_to_primitive;
            toStringTag: scope.statics.symbol_to_string_tag;
            unscopables: scope.statics.symbol_unscopables;
        });

        let error_ctor = register_builtin_type!(scope.statics.error_ctor, {
            #[prototype] scope.statics.function_proto;
            #[fn_prototype] scope.statics.error_prototype;
        });

        let error_proto = register_builtin_type!(scope.statics.error_prototype, {
            #[prototype] scope.statics.object_prototype;
            #[properties]
            toString: scope.statics.error_to_string;
        });

        let global = register_builtin_type!(global, {
            #[prototype] scope.statics.object_prototype;

            #[properties]
            isNaN: scope.statics.is_nan;
            isFinite: scope.statics.is_finite;
            parseFloat: scope.statics.parse_float;
            parseInt: scope.statics.parse_int;

            Symbol: symbol_ctor;
            Array: array_ctor;
            Error: error_ctor;
            String: string_ctor;
            Object: object_ctor;
            console: console;
            Math: math;
            Number: number_ctor;
            Boolean: boolean_ctor;
        });
    }

    /// Fetches the current instruction/value in the currently executing frame
    /// and increments the instruction pointer
    pub(crate) fn fetch_and_inc_ip(&mut self) -> u8 {
        let frame = self.frames.last_mut().expect("No frame");
        let ip = frame.ip;
        frame.ip += 1;
        frame.buffer[ip]
    }

    /// Fetches a wide value (16-bit) in the currently executing frame
    /// and increments the instruction pointer
    pub(crate) fn fetchw_and_inc_ip(&mut self) -> u16 {
        let frame = self.frames.last_mut().expect("No frame");
        let value: [u8; 2] = frame.buffer[frame.ip..frame.ip + 2]
            .try_into()
            .expect("Failed to get wide instruction");

        frame.ip += 2;
        u16::from_ne_bytes(value)
    }

    /// Pushes a constant at the given index in the current frame on the top of the stack
    pub(crate) fn push_constant(&mut self, idx: usize) -> Result<(), Value> {
        let frame = self.frames.last().expect("No frame");
        let value = Value::from_constant(frame.constants[idx].clone(), self);
        self.try_push_stack(value)?;
        Ok(())
    }

    pub(crate) fn get_frame_sp(&self) -> usize {
        self.frames.last().map(|frame| frame.sp).expect("No frame")
    }

    pub(crate) fn get_local(&self, id: usize) -> Option<Value> {
        self.stack.get(self.get_frame_sp() + id).cloned()
    }

    pub(crate) fn get_external(&self, id: usize) -> Option<&Handle<dyn Object>> {
        self.frames.last()?.externals.get(id)
    }

    pub(crate) fn set_local(&mut self, id: usize, value: Value) {
        let sp = self.get_frame_sp();
        let idx = sp + id;

        if let Value::External(o) = &mut self.stack[idx] {
            o.replace(value.into_boxed());
        } else {
            self.stack[idx] = value;
        }
    }

    pub(crate) fn try_push_frame(&mut self, frame: Frame) -> Result<(), Value> {
        if self.frames.len() > MAX_FRAME_STACK_SIZE {
            throw!(self, "Maximum call stack size exceeded");
        }

        self.frames.push(frame);
        Ok(())
    }

    pub(crate) fn try_push_stack(&mut self, value: Value) -> Result<(), Value> {
        if self.stack.len() > MAX_STACK_SIZE {
            throw!(self, "Maximum stack size exceeded");
        }
        
        self.stack.push(value);
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
            throw!(self, "Maximum stack size exceeded");
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

    pub(crate) fn drain_stack<R>(&mut self, range: R) -> Drain<'_, Value>
    where
        R: RangeBounds<usize>,
    {
        self.stack.drain(range)
    }

    fn handle_rt_error(&mut self, err: Value, max_fp: usize) -> Result<(), Value> {
        if let Some(last) = self.try_blocks.last() {
            // if we're in a try-catch block, we need to jump to it
            let try_fp = last.frame_ip;
            let catch_ip = last.catch_ip;
            let frame_ip = self.frames.len();

            // Do not unwind further than we are allowed to. If the last try block is "outside" of
            // the frame that this execution context was instantiated in, then we can't jump there.
            if try_fp < max_fp {
                return Err(err);
            }

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
            Err(err)
        }
    }

    /// Executes a frame in this VM
    pub fn execute_frame(&mut self, frame: Frame) -> Result<HandleResult, Value> {
        self.stack
            .resize(self.stack.len() + frame.reserved_stack_size, Value::undefined());

        self.try_push_frame(frame)?;

        let fp = self.frames.len();

        loop {
            let instruction = self.fetch_and_inc_ip();

            match dispatch::handle(self, instruction) {
                Ok(Some(hr)) => return Ok(hr),
                Ok(None) => continue,
                Err(e) => self.handle_rt_error(e, fp)?, // TODO: pop frame
            }
        }
    }

    pub fn execute_module(&mut self, mut frame: Frame) -> Result<Exports, Value> {
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
        self.trace_roots();

        // All reachable roots are marked.
        unsafe { self.gc.sweep() };
    }

    fn trace_roots(&mut self) {
        self.frames.trace();
        self.stack.trace();
        self.global.trace();
        self.externals.trace();
        self.statics.trace();
    }

    pub fn statics(&self) -> &Statics {
        &self.statics
    }

    pub fn gc_mut(&mut self) -> &mut Gc<dyn Object> {
        &mut self.gc
    }

    pub fn register<O: Object + 'static>(&mut self, obj: O) -> Handle<dyn Object> {
        self.gc.register(obj)
    }

    pub fn params(&self) -> &VmParams {
        &self.params
    }
}

impl fmt::Debug for Vm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Vm")
    }
}

#[test]
fn test_eval() {
    let mut vm = Vm::new(Default::default());
    let value = vm
        .eval(
            r#"
        // console.log(1337); 18
        function add(a,b) {
            return a +b
        }
        add(10, 7) + 1
    "#,
            Default::default(),
        )
        .unwrap();

    assert_eq!(vm.stack.len(), 0);
    assert_eq!(vm.frames.len(), 0);
    match value {
        Value::Number(n) => assert_eq!(n, 18.0),
        _ => unreachable!("{:?}", value),
    }
}
