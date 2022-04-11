use std::{convert::TryInto, fmt};

use crate::gc::{handle::Handle, trace::Trace, Gc};

use self::{
    dispatch::HandleResult,
    external::Externals,
    frame::{Frame, TryBlock},
    local::LocalScope,
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
pub mod statics;
pub mod util;
pub mod value;

pub const MAX_STACK_SIZE: usize = 8196;

pub struct Vm {
    frames: Vec<Frame>,
    stack: Vec<Value>,
    gc: Gc<dyn Object>,
    global: Handle<dyn Object>,
    externals: Externals,
    statics: Statics,
    try_blocks: Vec<TryBlock>,
}

impl Vm {
    pub fn new() -> Self {
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
        };
        vm.prepare();
        vm
    }

    /// Prepare the VM for execution.
    #[rustfmt::skip]
    fn prepare(&mut self) {
        let mut scope = LocalScope::new(self);

        let global = scope.global.clone();

        let object = {
            let object = scope.statics.object_ctor.clone();
            let object_proto = scope.statics.object_prototype.clone();
            let create = scope.statics.object_create.clone();
            let keys = scope.statics.object_keys.clone();

            object.set_prototype(&mut scope, object_proto.into()).unwrap();
            object.set_property(&mut scope, "create".into(), create.into()).unwrap();
            object.set_property(&mut scope, "keys".into(), keys.into()).unwrap();
            object
        };

        let console = {
            let console = scope.statics.console.clone();
            let log = scope.statics.console_log.clone();
            console.set_property(&mut scope, "log".into(), log.into()).unwrap();
            console
        };

        let math = {
            let math = scope.statics.math.clone();
            let floor = scope.statics.math_floor.clone();
            let abs = scope.statics.math_abs.clone();
            let acos = scope.statics.math_acos.clone();
            let acosh = scope.statics.math_acosh.clone();
            let asin = scope.statics.math_asin.clone();
            let asinh = scope.statics.math_asinh.clone();
            let atan = scope.statics.math_atan.clone();
            let atanh = scope.statics.math_atanh.clone();
            let atan2 = scope.statics.math_atan2.clone();
            let cbrt = scope.statics.math_cbrt.clone();
            let ceil = scope.statics.math_ceil.clone();
            let clz32 = scope.statics.math_clz32.clone();
            let cos = scope.statics.math_cos.clone();
            let cosh = scope.statics.math_cosh.clone();
            let exp = scope.statics.math_exp.clone();
            let expm1 = scope.statics.math_expm1.clone();
            let log = scope.statics.math_log.clone();
            let log1p = scope.statics.math_log1p.clone();
            let log10 = scope.statics.math_log10.clone();
            let log2 = scope.statics.math_log2.clone();
            let round = scope.statics.math_round.clone();
            let sin = scope.statics.math_sin.clone();
            let sinh = scope.statics.math_sinh.clone();
            let sqrt = scope.statics.math_sqrt.clone();
            let tan = scope.statics.math_tan.clone();
            let tanh = scope.statics.math_tanh.clone();
            let trunc = scope.statics.math_trunc.clone();

            math.set_property(&mut scope, "floor".into(), floor.into()).unwrap();
            math.set_property(&mut scope, "abs".into(), abs.into()).unwrap();
            math.set_property(&mut scope, "acos".into(), acos.into()).unwrap();
            math.set_property(&mut scope, "acosh".into(), acosh.into()).unwrap();
            math.set_property(&mut scope, "asin".into(), asin.into()).unwrap();
            math.set_property(&mut scope, "asinh".into(), asinh.into()).unwrap();
            math.set_property(&mut scope, "atan".into(), atan.into()).unwrap();
            math.set_property(&mut scope, "atanh".into(), atanh.into()).unwrap();
            math.set_property(&mut scope, "atan2".into(), atan2.into()).unwrap();
            math.set_property(&mut scope, "cbrt".into(), cbrt.into()).unwrap();
            math.set_property(&mut scope, "ceil".into(), ceil.into()).unwrap();
            math.set_property(&mut scope, "clz32".into(), clz32.into()).unwrap();
            math.set_property(&mut scope, "cos".into(), cos.into()).unwrap();
            math.set_property(&mut scope, "cosh".into(), cosh.into()).unwrap();
            math.set_property(&mut scope, "exp".into(), exp.into()).unwrap();
            math.set_property(&mut scope, "expm1".into(), expm1.into()).unwrap();
            math.set_property(&mut scope, "log".into(), log.into()).unwrap();
            math.set_property(&mut scope, "log1p".into(), log1p.into()).unwrap();
            math.set_property(&mut scope, "log10".into(), log10.into()).unwrap();
            math.set_property(&mut scope, "log2".into(), log2.into()).unwrap();
            math.set_property(&mut scope, "round".into(), round.into()).unwrap();
            math.set_property(&mut scope, "sin".into(), sin.into()).unwrap();
            math.set_property(&mut scope, "sinh".into(), sinh.into()).unwrap();
            math.set_property(&mut scope, "sqrt".into(), sqrt.into()).unwrap();
            math.set_property(&mut scope, "tan".into(), tan.into()).unwrap();
            math.set_property(&mut scope, "tanh".into(), tanh.into()).unwrap();
            math.set_property(&mut scope, "trunc".into(), trunc.into()).unwrap();

            math
        };

        let number = {
            let number = scope.statics.number_ctor.clone();
            let number_prototype = scope.statics.number_prototype.clone();
            let is_finite = scope.statics.number_is_finite.clone();
            let is_nan = scope.statics.number_is_nan.clone();
            let is_safe_integer = scope.statics.number_is_safe_integer.clone();

            number.set_property(&mut scope, "isFinite".into(), is_finite.into()).unwrap();
            number.set_property(&mut scope, "isNaN".into(), is_nan.into()).unwrap();
            number.set_property(&mut scope, "isSafeInteger".into(), is_safe_integer.into()).unwrap();
            number.set_prototype(&mut scope, number_prototype.into()).unwrap();

            number
        };

        let number_proto = {
            let number = scope.statics.number_prototype.clone();
            let tostring = scope.statics.number_tostring.clone();
            let to_fixed = scope.statics.number_to_fixed.clone();
            number.set_property(&mut scope, "toString".into(), tostring.into()).unwrap();
            number.set_property(&mut scope, "toFixed".into(), to_fixed.into()).unwrap();
            number
        };

        let boolean = {
            let boolean = scope.statics.boolean_ctor.clone();
            let boolean_prototype = scope.statics.boolean_prototype.clone();
            boolean.set_prototype(&mut scope, boolean_prototype.into()).unwrap();
            boolean
        };

        let boolean_proto = {
            let boolean = scope.statics.boolean_prototype.clone();
            let tostring = scope.statics.boolean_tostring.clone();
            let valueof = scope.statics.boolean_valueof.clone();
            boolean.set_property(&mut scope, "toString".into(), tostring.into()).unwrap();
            boolean.set_property(&mut scope, "valueOf".into(), valueof.into()).unwrap();
            boolean
        };

        let string = {
            let string = scope.statics.string_ctor.clone();
            let string_prototype = scope.statics.string_prototype.clone();

            string.set_prototype(&mut scope, string_prototype.into()).unwrap();
            string
        };

        let string_prototype = {
            let string = scope.statics.string_prototype.clone();
            let tostring = scope.statics.string_tostring.clone();
            string.set_property(&mut scope, "toString".into(), tostring.into()).unwrap();
            let charat = scope.statics.string_char_at.clone();
            string.set_property(&mut scope, "charAt".into(), charat.into()).unwrap();
            let charcodeat = scope.statics.string_char_code_at.clone();
            string.set_property(&mut scope, "charCodeAt".into(), charcodeat.into()).unwrap();
            let concat = scope.statics.string_concat.clone();
            string.set_property(&mut scope, "concat".into(), concat.into()).unwrap();
            let endswith = scope.statics.string_ends_with.clone();
            string.set_property(&mut scope, "endsWith".into(), endswith.into()).unwrap();
            let startswith = scope.statics.string_starts_with.clone();
            string.set_property(&mut scope, "startsWith".into(), startswith.into()).unwrap();
            let includes = scope.statics.string_includes.clone();
            string.set_property(&mut scope, "includes".into(), includes.into()).unwrap();
            let indexof = scope.statics.string_index_of.clone();
            string.set_property(&mut scope, "indexOf".into(), indexof.into()).unwrap();
            let lastindexof = scope.statics.string_last_index_of.clone();
            string.set_property(&mut scope, "lastIndexOf".into(), lastindexof.into()).unwrap();
            let padend = scope.statics.string_pad_end.clone();
            string.set_property(&mut scope, "padEnd".into(), padend.into()).unwrap();
            let padstart = scope.statics.string_pad_start.clone();
            string.set_property(&mut scope, "padStart".into(), padstart.into()).unwrap();
            let repeat = scope.statics.string_repeat.clone();
            string.set_property(&mut scope, "repeat".into(), repeat.into()).unwrap();
            let replace = scope.statics.string_replace.clone();
            string.set_property(&mut scope, "replace".into(), replace.into()).unwrap();
            let replaceall = scope.statics.string_replace_all.clone();
            string.set_property(&mut scope, "replaceAll".into(), replaceall.into()).unwrap();
            let split = scope.statics.string_split.clone();
            string.set_property(&mut scope, "split".into(), split.into()).unwrap();
            let to_lowercase = scope.statics.string_to_lowercase.clone();
            string.set_property(&mut scope, "toLowerCase".into(), to_lowercase.into()).unwrap();
            let to_uppercase = scope.statics.string_to_uppercase.clone();
            string.set_property(&mut scope, "toUpperCase".into(), to_uppercase.into()).unwrap();


            string
        };

        let array = {
            let array = scope.statics.array_ctor.clone();
            let array_prototype = scope.statics.array_prototype.clone();

            array.set_prototype(&mut scope, array_prototype.into()).unwrap();
            array
        };

        let array_proto = {
            let array = scope.statics.array_prototype.clone();
            let tostring = scope.statics.array_tostring.clone();
            let join = scope.statics.array_join.clone();
            let values = scope.statics.array_values.clone();
            let symbol_iterator = scope.statics.symbol_iterator.clone();

            array.set_property(&mut scope, "toString".into(), tostring.into()).unwrap();
            array.set_property(&mut scope, "join".into(), join.into()).unwrap();
            array.set_property(&mut scope, "values".into(), values.clone().into()).unwrap();
            array.set_property(&mut scope, symbol_iterator.into(), values.into()).unwrap();

            array
        };

        let array_iterator_proto = {
            let array_iterator_proto = scope.statics.array_iterator_prototype.clone();
            let next = scope.statics.array_iterator_next.clone();
            array_iterator_proto.set_property(&mut scope, "next".into(), next.into()).unwrap();
            array_iterator_proto
        };

        let symbol = {
            let symbol = scope.statics.symbol_ctor.clone();

            let async_iterator = scope.statics.symbol_async_iterator.clone();
            let has_instance = scope.statics.symbol_has_instance.clone();
            let iterator = scope.statics.symbol_iterator.clone();
            let match_ = scope.statics.symbol_match.clone();
            let match_all = scope.statics.symbol_match_all.clone();
            let replace = scope.statics.symbol_replace.clone();
            let search = scope.statics.symbol_search.clone();
            let species = scope.statics.symbol_species.clone();
            let split = scope.statics.symbol_split.clone();
            let to_primitive = scope.statics.symbol_to_primitive.clone();
            let to_string_tag = scope.statics.symbol_to_string_tag.clone();
            let unscopables = scope.statics.symbol_unscopables.clone();

            symbol.set_property(&mut scope, "asyncIterator".into(), async_iterator.into()).unwrap();
            symbol.set_property(&mut scope, "hasInstance".into(), has_instance.into()).unwrap();
            symbol.set_property(&mut scope, "iterator".into(), iterator.into()).unwrap();
            symbol.set_property(&mut scope, "match".into(), match_.into()).unwrap();
            symbol.set_property(&mut scope, "matchAll".into(), match_all.into()).unwrap();
            symbol.set_property(&mut scope, "replace".into(), replace.into()).unwrap();
            symbol.set_property(&mut scope, "search".into(), search.into()).unwrap();
            symbol.set_property(&mut scope, "species".into(), species.into()).unwrap();
            symbol.set_property(&mut scope, "split".into(), split.into()).unwrap();
            symbol.set_property(&mut scope, "toPrimitive".into(), to_primitive.into()).unwrap();
            symbol.set_property(&mut scope, "toStringTag".into(), to_string_tag.into()).unwrap();
            symbol.set_property(&mut scope, "unscopables".into(), unscopables.into()).unwrap();

            symbol
        };

        let global = {
            let is_nan = scope.statics.is_nan.clone();
            let is_finite = scope.statics.is_finite.clone();
            let parse_float = scope.statics.parse_float.clone();
            let parse_int = scope.statics.parse_int.clone();
            global.set_property(&mut scope, "isNaN".into(), is_nan.into()).unwrap();
            global.set_property(&mut scope, "isFinite".into(), is_finite.into()).unwrap();
            global.set_property(&mut scope, "parseFloat".into(), parse_float.into()).unwrap();
            global.set_property(&mut scope, "parseInt".into(), parse_int.into()).unwrap();

            global
        };

        global.set_property(&mut scope, "Symbol".into(), symbol.into()).unwrap();
        global.set_property(&mut scope, "Array".into(), array.into()).unwrap();
        global.set_property(&mut scope, "String".into(), string.into()).unwrap();
        global.set_property(&mut scope, "Object".into(), object.into()).unwrap();
        global.set_property(&mut scope, "console".into(), console.into()).unwrap();
        global.set_property(&mut scope, "Math".into(), math.into()).unwrap();
        global.set_property(&mut scope, "Number".into(), number.into()).unwrap();
        global.set_property(&mut scope, "Boolean".into(), boolean.into()).unwrap();
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

    pub(crate) fn try_push_stack(&mut self, value: Value) -> Result<(), Value> {
        if self.stack.len() > MAX_STACK_SIZE {
            panic!("Stack overflow"); // todo: return result
        }
        self.stack.push(value);
        Ok(())
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
    pub fn execute_frame(&mut self, frame: Frame) -> Result<Value, Value> {
        self.stack
            .resize(self.stack.len() + frame.local_count, Value::undefined());

        self.frames.push(frame);

        let fp = self.frames.len();

        loop {
            let instruction = self.fetch_and_inc_ip();

            match dispatch::handle(self, instruction) {
                Ok(HandleResult::Return(value)) => return Ok(value),
                Ok(HandleResult::Continue) => continue,
                Err(e) => self.handle_rt_error(e, fp)?,
            }
        }
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
}

impl fmt::Debug for Vm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Vm")
    }
}

#[test]
fn test_eval() {
    let (vm, value) = crate::eval(
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
