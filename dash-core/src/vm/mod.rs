use std::{convert::TryInto, fmt};

use crate::gc::{handle::Handle, Gc};

use self::{
    dispatch::HandleResult,
    external::Externals,
    frame::Frame,
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
            object.set_prototype(&mut scope, object_proto.into()).unwrap();
            object
        };

        let console = {
            let console = scope.statics.console.clone();
            let log = scope.statics.console_log.clone();
            console.set_property(&mut scope, "log", log.into()).unwrap();
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

            math.set_property(&mut scope, "floor", floor.into()).unwrap();
            math.set_property(&mut scope, "abs", abs.into()).unwrap();
            math.set_property(&mut scope, "acos", acos.into()).unwrap();
            math.set_property(&mut scope, "acosh", acosh.into()).unwrap();
            math.set_property(&mut scope, "asin", asin.into()).unwrap();
            math.set_property(&mut scope, "asinh", asinh.into()).unwrap();
            math.set_property(&mut scope, "atan", atan.into()).unwrap();
            math.set_property(&mut scope, "atanh", atanh.into()).unwrap();
            math.set_property(&mut scope, "atan2", atan2.into()).unwrap();
            math.set_property(&mut scope, "cbrt", cbrt.into()).unwrap();
            math.set_property(&mut scope, "ceil", ceil.into()).unwrap();
            math.set_property(&mut scope, "clz32", clz32.into()).unwrap();
            math.set_property(&mut scope, "cos", cos.into()).unwrap();
            math.set_property(&mut scope, "cosh", cosh.into()).unwrap();
            math.set_property(&mut scope, "exp", exp.into()).unwrap();
            math.set_property(&mut scope, "expm1", expm1.into()).unwrap();
            math.set_property(&mut scope, "log", log.into()).unwrap();
            math.set_property(&mut scope, "log1p", log1p.into()).unwrap();
            math.set_property(&mut scope, "log10", log10.into()).unwrap();
            math.set_property(&mut scope, "log2", log2.into()).unwrap();
            math.set_property(&mut scope, "round", round.into()).unwrap();
            math.set_property(&mut scope, "sin", sin.into()).unwrap();
            math.set_property(&mut scope, "sinh", sinh.into()).unwrap();
            math.set_property(&mut scope, "sqrt", sqrt.into()).unwrap();
            math.set_property(&mut scope, "tan", tan.into()).unwrap();
            math.set_property(&mut scope, "tanh", tanh.into()).unwrap();
            math.set_property(&mut scope, "trunc", trunc.into()).unwrap();

            math
        };

        let number = {
            let number = scope.statics.number_ctor.clone();
            let number_prototype = scope.statics.number_prototype.clone();
            let is_finite = scope.statics.number_is_finite.clone();
            let is_nan = scope.statics.number_is_nan.clone();
            let is_safe_integer = scope.statics.number_is_safe_integer.clone();

            number.set_property(&mut scope, "isFinite", is_finite.into()).unwrap();
            number.set_property(&mut scope, "isNaN", is_nan.into()).unwrap();
            number.set_property(&mut scope, "isSafeInteger", is_safe_integer.into()).unwrap();
            number.set_prototype(&mut scope, number_prototype.into()).unwrap();

            number
        };

        let number_proto = {
            let number = scope.statics.number_prototype.clone();
            let tostring = scope.statics.number_tostring.clone();
            let to_fixed = scope.statics.number_to_fixed.clone();
            number.set_property(&mut scope, "toString", tostring.into()).unwrap();
            number.set_property(&mut scope, "toFixed", to_fixed.into()).unwrap();
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
            boolean.set_property(&mut scope, "toString", tostring.into()).unwrap();
            boolean.set_property(&mut scope, "valueOf", valueof.into()).unwrap();
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
            string.set_property(&mut scope, "toString", tostring.into()).unwrap();
            string
        };

        let global = {
            let is_nan = scope.statics.is_nan.clone();
            let is_finite = scope.statics.is_finite.clone();
            let parse_float = scope.statics.parse_float.clone();
            let parse_int = scope.statics.parse_int.clone();
            global.set_property(&mut scope, "isNaN", is_nan.into()).unwrap();
            global.set_property(&mut scope, "isFinite", is_finite.into()).unwrap();
            global.set_property(&mut scope, "parseFloat", parse_float.into()).unwrap();
            global.set_property(&mut scope, "parseInt", parse_int.into()).unwrap();

            global
        };

        global.set_property(&mut scope, "String", string.into()).unwrap();
        global.set_property(&mut scope, "Object", object.into()).unwrap();
        global.set_property(&mut scope, "console", console.into()).unwrap();
        global.set_property(&mut scope, "Math", math.into()).unwrap();
        global.set_property(&mut scope, "Number", number.into()).unwrap();
        global.set_property(&mut scope, "Boolean", boolean.into()).unwrap();
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

    /// Executes a frame in this VM
    pub fn execute_frame(&mut self, frame: Frame) -> Result<Value, Value> {
        self.stack
            .resize(self.stack.len() + frame.local_count, Value::undefined());

        self.frames.push(frame);

        loop {
            let instruction = self.fetch_and_inc_ip();

            match dispatch::handle(self, instruction) {
                Ok(HandleResult::Return(value)) => return Ok(value),
                Ok(HandleResult::Continue) => continue,
                Err(e) => return Err(e),
            }
        }
    }

    pub fn statics(&self) -> &Statics {
        &self.statics
    }

    pub fn gc_mut(&mut self) -> &mut Gc<dyn Object> {
        &mut self.gc
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
    )
    .unwrap();

    assert_eq!(vm.stack.len(), 0);
    assert_eq!(vm.frames.len(), 0);
    match value {
        Value::Number(n) => assert_eq!(n, 18.0),
        _ => unreachable!("{:?}", value),
    }
}
