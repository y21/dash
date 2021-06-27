pub mod abstractions;
pub mod conversions;
pub mod environment;
pub mod frame;
pub mod instruction;
pub mod stack;
pub mod statics;
pub mod upvalue;
pub mod value;

use std::{any::Any, borrow::Cow, cell::RefCell, collections::HashMap, rc::Rc};

use instruction::{Instruction, Opcode};
use value::Value;

use crate::{
    agent::Agent,
    js_std::{self, error::MaybeRc},
    vm::{
        frame::{NativeResume, UnwindHandler},
        upvalue::Upvalue,
        value::{
            array::Array,
            function::{
                CallContext, CallResult, CallState, Closure, FunctionKind, Receiver, UserFunction,
            },
            ops::compare::Compare,
            ValueKind,
        },
    },
};

use self::{
    frame::{Frame, Loop},
    instruction::Constant,
    stack::Stack,
    statics::Statics,
    value::object::{AnyObject, Object},
};

#[derive(Debug)]
pub enum VMError {
    UncaughtError(Rc<RefCell<Value>>),
}

impl VMError {
    pub fn to_string(&self) -> Cow<str> {
        match self {
            Self::UncaughtError(err_cell) => {
                let err = err_cell.borrow();
                let stack_cell = err.get_field("stack").unwrap();
                let stack_ref = stack_cell.borrow();
                let stack_string = stack_ref.as_string().unwrap();
                Cow::Owned(String::from(stack_string))
            }
        }
    }
}

pub struct VM {
    /// Call stack
    pub(crate) frames: Stack<Frame, 256>,
    /// Stack
    pub(crate) stack: Stack<Rc<RefCell<Value>>, 512>,
    /// Global namespace
    pub(crate) global: Rc<RefCell<Value>>,
    /// Static values created once when the VM is initialized
    pub(crate) statics: Statics,
    /// Embedder specific slot data
    pub(crate) slot: Option<Box<dyn Any>>,
    /// Unwind (try/catch) handlers
    pub(crate) unwind_handlers: Stack<UnwindHandler, 128>,
    /// Loops
    pub(crate) loops: Stack<Loop, 32>,
    /// Agent
    pub(crate) agent: Box<dyn Agent>,
}

impl VM {
    pub fn new_with_agent(func: UserFunction, agent: Box<dyn Agent>) -> Self {
        let mut frames = Stack::new();
        frames.push(Frame {
            buffer: func.buffer.clone(),
            func: Value::from(Closure::new(func)).into(),
            ip: 0,
            sp: 0,
            state: None,
            resume: None,
        });

        let mut vm = Self {
            frames,
            stack: Stack::new(),
            global: Value::from(AnyObject {}).into(),
            statics: Statics::new(),
            unwind_handlers: Stack::new(),
            loops: Stack::new(),
            slot: None,
            agent,
        };
        vm.prepare_stdlib();
        vm
    }

    pub fn new(func: UserFunction) -> Self {
        Self::new_with_agent(func, Box::new(()))
    }

    pub fn global(&self) -> &Rc<RefCell<Value>> {
        &self.global
    }

    pub fn set_slot<T: 'static>(&mut self, value: T) {
        self.slot.insert(Box::new(value) as Box<dyn Any>);
    }

    pub fn get_slot<T: 'static>(&self) -> Option<&T> {
        let slot = self.slot.as_ref()?;
        slot.downcast_ref::<T>()
    }

    pub fn get_slot_mut<T: 'static>(&mut self) -> Option<&mut T> {
        let slot = self.slot.as_mut()?;
        slot.downcast_mut::<T>()
    }

    fn frame(&self) -> &Frame {
        unsafe { self.frames.get_unchecked() }
    }

    fn frame_mut(&mut self) -> &mut Frame {
        unsafe { self.frames.get_mut_unchecked() }
    }

    fn ip(&self) -> usize {
        self.frame().ip
    }

    fn buffer(&self) -> &[Instruction] {
        &self.frame().buffer
    }

    fn is_eof(&self) -> bool {
        self.ip() >= self.buffer().len()
    }

    fn next(&mut self) -> Option<&Instruction> {
        if self.is_eof() {
            return None;
        }

        self.frame_mut().ip += 1;

        Some(&self.buffer()[self.ip() - 1])
    }

    fn read_constant(&mut self) -> Option<Constant> {
        self.next().cloned().map(|x| x.into_operand())
    }

    fn read_op(&mut self) -> Option<Opcode> {
        self.next().cloned().map(|x| x.into_op())
    }

    fn read_user_function(&mut self) -> Option<UserFunction> {
        self.read_constant()
            .and_then(|c| c.into_value())
            .and_then(|v| v.into_object())
            .and_then(|o| match o {
                Object::Function(FunctionKind::User(f)) => Some(f),
                _ => None,
            })
    }

    fn read_number(&mut self) -> f64 {
        self.stack.pop().borrow().as_number()
    }

    fn read_index(&mut self) -> Option<usize> {
        self.stack
            .pop()
            .borrow()
            .as_constant()
            .and_then(|c| c.as_index())
    }

    fn pop_owned(&mut self) -> Option<Value> {
        Value::try_into_inner(self.stack.pop())
    }

    fn read_lhs_rhs(&mut self) -> (Rc<RefCell<Value>>, Rc<RefCell<Value>>) {
        let rhs = self.stack.pop();
        let lhs = self.stack.pop();
        (lhs, rhs)
    }

    fn with_lhs_borrowed<F, T>(&mut self, func: F) -> T
    where
        F: Fn(&Value) -> T,
    {
        let lhs_cell = self.stack.pop();
        let lhs = lhs_cell.borrow();
        func(&*lhs)
    }

    fn with_lhs_rhs_borrowed<F, T>(&mut self, func: F) -> T
    where
        F: Fn(&Value, &Value) -> T,
    {
        let (lhs_cell, rhs_cell) = self.read_lhs_rhs();
        let lhs = lhs_cell.borrow();
        let rhs = rhs_cell.borrow();
        func(&*lhs, &*rhs)
    }

    pub fn create_object(&self) -> Value {
        self.create_js_value(AnyObject {})
    }

    pub fn create_null_object(&self) -> Value {
        let mut o = Value::from(AnyObject {});
        o.detect_internal_properties(self);
        // Override [[Prototype]]
        o.proto = None;
        o
    }

    pub fn create_object_with_fields(
        &self,
        fields: impl Into<HashMap<Box<str>, Rc<RefCell<Value>>>>,
    ) -> Value {
        let mut o = self.create_object();
        o.fields = fields.into();
        o
    }

    pub fn create_js_value(&self, value: impl Into<Value>) -> Value {
        let mut value = value.into();
        value.detect_internal_properties(self);
        value
    }

    pub fn create_array(&self, arr: Array) -> Value {
        let mut o = Value::from(arr);
        o.proto = Some(Rc::downgrade(&self.statics.array_proto));
        o.constructor = Some(Rc::downgrade(&self.statics.array_ctor));
        o
    }

    fn prepare_stdlib(&mut self) {
        let mut global = self.global.borrow_mut();
        global.detect_internal_properties(self);
        // TODO: attaching globalThis causes a reference cycle and memory leaks
        // We somehow need to have
        // global.set_property("globalThis", self.global.clone());

        self.statics.boolean_proto = self.create_object().into();
        self.statics.number_proto = self.create_object().into();
        self.statics.string_proto = self.create_object().into();
        self.statics.function_proto = self.create_object().into();
        self.statics.array_proto = {
            let mut o = self.create_object();
            o.set_property("push", Rc::clone(&self.statics.array_push));
            o.set_property("concat", Rc::clone(&self.statics.array_concat));
            o.set_property("map", Rc::clone(&self.statics.array_map));
            o.set_property("every", Rc::clone(&self.statics.array_every));
            o.set_property("fill", Rc::clone(&self.statics.array_fill));
            o.set_property("filter", Rc::clone(&self.statics.array_filter));
            o.set_property("find", Rc::clone(&self.statics.array_find));
            o.set_property("findIndex", Rc::clone(&self.statics.array_find_index));
            o.set_property("flat", Rc::clone(&self.statics.array_flat));
            o.set_property("forEach", Rc::clone(&self.statics.array_for_each));
            o.set_property("from", Rc::clone(&self.statics.array_from));
            o.set_property("includes", Rc::clone(&self.statics.array_includes));
            o.set_property("indexOf", Rc::clone(&self.statics.array_index_of));
            o.set_property("join", Rc::clone(&self.statics.array_join));
            o.set_property("lastIndexOf", Rc::clone(&self.statics.array_last_index_of));
            o.set_property("of", Rc::clone(&self.statics.array_of));
            o.set_property("pop", Rc::clone(&self.statics.array_pop));
            o.set_property("reduce", Rc::clone(&self.statics.array_reduce));
            o.set_property("reduceRight", Rc::clone(&self.statics.array_reduce_right));
            o.set_property("reverse", Rc::clone(&self.statics.array_reverse));
            o.set_property("shift", Rc::clone(&self.statics.array_shift));
            o.set_property("slice", Rc::clone(&self.statics.array_slice));
            o.set_property("some", Rc::clone(&self.statics.array_some));
            o.set_property("sort", Rc::clone(&self.statics.array_sort));
            o.set_property("splice", Rc::clone(&self.statics.array_splice));
            o.set_property("unshift", Rc::clone(&self.statics.array_unshift));
            o.into()
        };

        {
            let mut array_ctor = self.statics.array_ctor.borrow_mut();
            array_ctor.set_property("isArray", Rc::clone(&self.statics.array_is_array));
        }

        self.statics.weakset_proto = self.create_object().into();
        self.statics.weakmap_proto = self.create_object().into();
        self.statics.error_proto = self.create_object().into();

        let mut object_proto = self.statics.object_proto.borrow_mut();
        object_proto.constructor = Some(Rc::downgrade(&self.statics.object_ctor));
        object_proto.proto = Some(Rc::downgrade(&Value::new(ValueKind::Null).into()));
        object_proto.set_property("toString", Rc::clone(&self.statics.object_to_string));

        // All functions that live in self.statics do not have a [[Prototype]] set
        // so we do it here
        fn patch_function_value(this: &VM, func: &Rc<RefCell<Value>>) {
            func.borrow_mut().detect_internal_properties(this);
        }

        fn patch_constructor(this: &VM, func: &Rc<RefCell<Value>>, prototype: &Rc<RefCell<Value>>) {
            let mut func_ref = func.borrow_mut();
            let real_func = func_ref.as_function_mut().unwrap();
            real_func.set_prototype(Rc::downgrade(prototype));
            func_ref.detect_internal_properties(this);
        }

        // Constructors
        patch_constructor(
            self,
            &self.statics.boolean_ctor,
            &self.statics.boolean_proto,
        );
        patch_constructor(self, &self.statics.number_ctor, &self.statics.number_proto);
        patch_constructor(self, &self.statics.string_ctor, &self.statics.string_proto);
        patch_constructor(
            self,
            &self.statics.function_ctor,
            &self.statics.function_proto,
        );
        patch_constructor(self, &self.statics.array_ctor, &self.statics.array_proto);
        patch_constructor(
            self,
            &self.statics.weakset_ctor,
            &self.statics.weakset_proto,
        );
        patch_constructor(
            self,
            &self.statics.weakmap_ctor,
            &self.statics.weakmap_proto,
        );
        patch_constructor(self, &self.statics.object_ctor, &self.statics.object_proto);
        patch_constructor(self, &self.statics.error_ctor, &self.statics.error_proto);
        // Other functions/methods
        patch_function_value(self, &self.statics.isnan);
        patch_function_value(self, &self.statics.object_define_property);
        patch_function_value(self, &self.statics.object_get_own_property_names);
        patch_function_value(self, &self.statics.object_to_string);
        patch_function_value(self, &self.statics.isnan);
        patch_function_value(self, &self.statics.console_log);
        patch_function_value(self, &self.statics.array_push);
        patch_function_value(self, &self.statics.math_pow);
        patch_function_value(self, &self.statics.math_abs);
        patch_function_value(self, &self.statics.math_ceil);
        patch_function_value(self, &self.statics.math_floor);
        patch_function_value(self, &self.statics.math_max);
        patch_function_value(self, &self.statics.math_random);
        patch_function_value(self, &self.statics.weakset_has);
        patch_function_value(self, &self.statics.weakset_add);
        patch_function_value(self, &self.statics.weakset_delete);
        patch_function_value(self, &self.statics.weakmap_has);
        patch_function_value(self, &self.statics.weakmap_add);
        patch_function_value(self, &self.statics.weakmap_get);
        patch_function_value(self, &self.statics.weakmap_delete);
        patch_function_value(self, &self.statics.json_parse);
        patch_function_value(self, &self.statics.json_stringify);

        global.set_property("NaN", self.create_js_value(f64::NAN).into());
        global.set_property("Infinity", self.create_js_value(f64::INFINITY).into());

        global.set_property("isNaN", self.statics.isnan.clone());

        let mut object_ctor = self.statics.object_ctor.borrow_mut();
        object_ctor.set_property(
            "defineProperty",
            self.statics.object_define_property.clone(),
        );
        object_ctor.set_property(
            "getOwnPropertyNames",
            self.statics.object_get_own_property_names.clone(),
        );
        object_ctor.set_property(
            "getPrototypeOf",
            self.statics.object_get_prototype_of.clone(),
        );
        global.set_property("Object", Rc::clone(&self.statics.object_ctor));

        let mut math_obj = self.create_object();
        math_obj.set_property("pow", Rc::clone(&self.statics.math_pow));
        math_obj.set_property("abs", Rc::clone(&self.statics.math_abs));
        math_obj.set_property("ceil", Rc::clone(&self.statics.math_ceil));
        math_obj.set_property("floor", Rc::clone(&self.statics.math_floor));
        math_obj.set_property("max", Rc::clone(&self.statics.math_max));
        math_obj.set_property("random", Rc::clone(&self.statics.math_random));

        math_obj.set_property("PI", self.create_js_value(std::f64::consts::PI).into());
        math_obj.set_property("E", self.create_js_value(std::f64::consts::E).into());
        math_obj.set_property("LN10", self.create_js_value(std::f64::consts::LN_10).into());
        math_obj.set_property("LN2", self.create_js_value(std::f64::consts::LN_2).into());
        math_obj.set_property(
            "LOG10E",
            self.create_js_value(std::f64::consts::LOG10_E).into(),
        );
        math_obj.set_property(
            "LOG2E",
            self.create_js_value(std::f64::consts::LOG2_E).into(),
        );
        math_obj.set_property(
            "SQRT2",
            self.create_js_value(std::f64::consts::SQRT_2).into(),
        );
        global.set_property("Math", math_obj.into());

        let mut json_obj = self.create_object();
        json_obj.set_property("parse", self.statics.json_parse.clone());
        json_obj.set_property("stringify", self.statics.json_stringify.clone());
        global.set_property("JSON", json_obj.into());

        let mut console_obj = self.create_object();
        console_obj.set_property("log", self.statics.console_log.clone());
        global.set_property("console", console_obj.into());

        global.set_property("Error", self.statics.error_ctor.clone());
        global.set_property("Boolean", self.statics.boolean_ctor.clone());
        global.set_property("Number", self.statics.number_ctor.clone());
        global.set_property("String", self.statics.string_ctor.clone());
        global.set_property("Function", self.statics.function_ctor.clone());
        global.set_property("Array", self.statics.array_ctor.clone());
        global.set_property("WeakSet", self.statics.weakset_ctor.clone());
        global.set_property("WeakMap", self.statics.weakmap_ctor.clone());
    }

    fn unwind(&mut self, value: Rc<RefCell<Value>>) -> Result<(), Rc<RefCell<Value>>> {
        // TODO: clean up resources caused by this unwind
        if self.unwind_handlers.get_stack_pointer() == 0 {
            return Err(value);
        }

        let handler = self.unwind_handlers.pop();
        if let Some(catch_value_sp) = handler.catch_value_sp {
            self.stack
                .set_relative(self.frame().sp, catch_value_sp, value);
        }
        self.frame_mut().ip = handler.catch_ip;
        Ok(())
    }

    pub fn generate_stack_trace(&self, message: Option<&str>) -> String {
        let mut stack = format!("Error: {}\n", message.unwrap_or(""));

        // Iterate over frames and add it to the stack string
        for frame in self.frames.as_array_bottom() {
            let frame = unsafe { &*frame.as_ptr() };
            stack.push_str("  at ");

            // Get reference to function
            let func = frame.func.borrow();
            let func_name = func
                .as_function()
                .and_then(FunctionKind::as_closure)
                .and_then(|c| c.func.name.as_ref());

            // Add function name to string (or <anonymous> if it's an anonymous function)
            stack.push_str(func_name.map(|x| &**x).unwrap_or("<anonymous>"));
            stack.push('\n');
        }

        stack
    }

    fn handle_native_return(
        &mut self,
        old_func: Rc<RefCell<Value>>,
        new_func: Rc<RefCell<Value>>,
        old_args: Vec<Rc<RefCell<Value>>>,
        new_args: Vec<Rc<RefCell<Value>>>,
        state: CallState<Box<dyn Any>>,
        receiver: Option<Rc<RefCell<Value>>>,
    ) {
        let func_ref = new_func.borrow();
        match func_ref.as_function().unwrap() {
            FunctionKind::Closure(closure) => {
                let sp = self.stack.get_stack_pointer();
                self.frame_mut().sp = sp;

                let frame = Frame {
                    buffer: closure.func.buffer.clone(),
                    ip: 0,
                    func: Rc::clone(&new_func),
                    sp,
                    state: Some(state),
                    resume: Some(NativeResume {
                        args: old_args,
                        ctor: false,
                        func: old_func,
                        receiver,
                    }),
                };

                self.frames.push(frame);
                for param in new_args.into_iter() {
                    self.stack.push(param);
                }
            }
            _ => todo!(),
        };
    }

    fn begin_function_call(
        &mut self,
        func_cell: Rc<RefCell<Value>>,
        mut params: Vec<Rc<RefCell<Value>>>,
    ) -> Result<(), Rc<RefCell<Value>>> {
        let func_cell_ref = func_cell.borrow();
        let func = match func_cell_ref.as_function().unwrap() {
            FunctionKind::Native(f) => {
                let receiver = f.receiver.as_ref().map(|rx| rx.get().clone());
                let mut state = CallState::default();
                let ctx = CallContext {
                    vm: self,
                    args: &mut params,
                    ctor: false,
                    receiver: receiver.clone(),
                    state: &mut state,
                    function_call_response: None,
                };

                let result = (f.func)(ctx)?;

                match result {
                    CallResult::Ready(ret) => self.stack.push(ret),
                    CallResult::UserFunction(new_func_cell, args) => self.handle_native_return(
                        Rc::clone(&func_cell),
                        new_func_cell,
                        params,
                        args,
                        state,
                        receiver,
                    ),
                };
                return Ok(());
            }
            FunctionKind::Closure(u) => u,
            // There should never be raw user functions
            _ => unreachable!(),
        };

        // By this point we know func_cell is a UserFunction

        let current_sp = self.stack.get_stack_pointer();

        let frame = Frame {
            buffer: func.func.buffer.clone(),
            ip: 0,
            func: func_cell.clone(),
            sp: current_sp,
            state: self.frame_mut().state.take(),
            resume: None,
        };
        self.frames.push(frame);

        let origin_param_count = func.func.params as usize;
        let param_count = params.len();

        for param in params.into_iter().rev() {
            self.stack.push(param);
        }

        for _ in 0..(origin_param_count.saturating_sub(param_count)) {
            self.stack.push(Value::new(ValueKind::Undefined).into());
        }

        Ok(())
    }

    pub fn interpret(&mut self) -> Result<Option<Rc<RefCell<Value>>>, VMError> {
        macro_rules! unwrap_or_unwind {
            ($e:expr, $err:expr) => {
                if let Some(v) = $e {
                    v
                } else {
                    unwind_abort_if_uncaught!($err)
                }
            };
        }

        macro_rules! unwind_abort_if_uncaught {
            ($e:expr) => {
                if let Err(e) = self.unwind($e) {
                    return Err(VMError::UncaughtError(e));
                } else {
                    continue;
                }
            };
        }

        while !self.is_eof() {
            let instruction = self.buffer()[self.ip()].as_op();

            self.frame_mut().ip += 1;

            match instruction {
                Opcode::Eof => return Ok(None),
                Opcode::Constant => {
                    let mut constant = self.read_constant().map(|c| c.try_into_value()).unwrap();

                    // Values emitted by the compiler do not have a [[Prototype]] set
                    // so we need to do that here when pushing a value onto the stack
                    constant.detect_internal_properties(self);

                    self.stack.push(constant.into());
                }
                Opcode::Closure => {
                    let func = self.read_user_function().unwrap();

                    let upvalue_count = func.upvalues as usize;

                    let mut closure =
                        Closure::with_upvalues(func, Vec::with_capacity(upvalue_count));

                    for _ in 0..closure.func.upvalues {
                        let is_local =
                            matches!(self.next().unwrap(), Instruction::Op(Opcode::UpvalueLocal));
                        let stack_idx = self.read_constant().and_then(|c| c.into_index()).unwrap();
                        if is_local {
                            let value =
                                unsafe { self.stack.peek_unchecked(self.frame().sp + stack_idx) };
                            closure.upvalues.push(Upvalue(value.clone()));
                        } else {
                            todo!("Resolve upvalues")
                        }
                    }

                    self.stack
                        .push(self.create_js_value(FunctionKind::Closure(closure)).into());
                }
                Opcode::Negate => {
                    let maybe_number = self.read_number();

                    self.stack.push(self.create_js_value(-maybe_number).into());
                }
                Opcode::Positive => {
                    let maybe_number = self.read_number();

                    self.stack.push(self.create_js_value(maybe_number).into());
                }
                Opcode::LogicalNot => {
                    let is_truthy = self.stack.pop().borrow().is_truthy();

                    self.stack.push(self.create_js_value(!is_truthy).into());
                }
                Opcode::Add => {
                    let result = self.with_lhs_rhs_borrowed(Value::add).into();
                    self.stack.push(result);
                }
                Opcode::Sub => {
                    let result = self.with_lhs_rhs_borrowed(Value::sub).into();
                    self.stack.push(result);
                }
                Opcode::Mul => {
                    let result = self.with_lhs_rhs_borrowed(Value::mul).into();
                    self.stack.push(result);
                }
                Opcode::Div => {
                    let result = self.with_lhs_rhs_borrowed(Value::div).into();
                    self.stack.push(result);
                }
                Opcode::Rem => {
                    let result = self.with_lhs_rhs_borrowed(Value::rem).into();
                    self.stack.push(result);
                }
                Opcode::Exponentiation => {
                    let result = self.with_lhs_rhs_borrowed(Value::pow).into();
                    self.stack.push(result);
                }
                Opcode::LeftShift => {
                    let result = self.with_lhs_rhs_borrowed(Value::left_shift).into();
                    self.stack.push(result);
                }
                Opcode::RightShift => {
                    let result = self.with_lhs_rhs_borrowed(Value::right_shift).into();
                    self.stack.push(result);
                }
                Opcode::UnsignedRightShift => {
                    let result = self
                        .with_lhs_rhs_borrowed(Value::unsigned_right_shift)
                        .into();
                    self.stack.push(result);
                }
                Opcode::BitwiseAnd => {
                    let result = self.with_lhs_rhs_borrowed(Value::bitwise_and).into();
                    self.stack.push(result);
                }
                Opcode::BitwiseOr => {
                    let result = self.with_lhs_rhs_borrowed(Value::bitwise_or).into();
                    self.stack.push(result);
                }
                Opcode::BitwiseXor => {
                    let result = self.with_lhs_rhs_borrowed(Value::bitwise_xor).into();
                    self.stack.push(result);
                }
                Opcode::BitwiseNot => {
                    let result = self.with_lhs_borrowed(Value::bitwise_not).into();
                    self.stack.push(result);
                }
                Opcode::SetGlobal => {
                    let name = self.pop_owned().unwrap().into_ident().unwrap();
                    let value = self.stack.pop();

                    let mut global = self.global.borrow_mut();
                    global.set_property(name, value);
                }
                Opcode::SetGlobalNoValue => {
                    let name = self.pop_owned().unwrap().into_ident().unwrap();

                    let mut global = self.global.borrow_mut();
                    global.set_property(name, Value::new(ValueKind::Undefined).into());
                }
                Opcode::GetGlobal => {
                    let name = self.pop_owned().unwrap().into_ident().unwrap();

                    let value = unwrap_or_unwind!(
                        Value::get_property(self, &self.global, &name, None),
                        js_std::error::create_error(
                            MaybeRc::Owned(&format!("{} is not defined", name)),
                            self
                        )
                    );

                    self.stack.push(value)
                }
                Opcode::SetLocal => {
                    let stack_idx = self.read_index().unwrap();
                    let value = self.stack.pop();
                    self.stack.set_relative(self.frame().sp, stack_idx, value);
                }
                Opcode::SetLocalNoValue => {
                    let stack_idx = self.read_index().unwrap();
                    self.stack.set_relative(
                        self.frame().sp,
                        stack_idx,
                        Value::new(ValueKind::Undefined).into(),
                    );
                }
                Opcode::GetLocal => {
                    let stack_idx = self.read_index().unwrap();

                    unsafe {
                        self.stack.push(
                            self.stack
                                .peek_relative_unchecked(self.frame().sp, stack_idx)
                                .clone(),
                        )
                    };
                }
                Opcode::GetUpvalue => {
                    let upvalue_idx = self.read_index().unwrap();

                    let value = {
                        let closure_cell = self.frame().func.borrow();
                        let closure = match closure_cell.as_function().unwrap() {
                            FunctionKind::Closure(c) => c,
                            _ => unreachable!(),
                        };
                        closure.upvalues[upvalue_idx].0.clone()
                    };

                    self.stack.push(value);
                }
                Opcode::ShortJmpIfFalse => {
                    let instruction_count = self.read_index().unwrap();

                    let condition_cell = unsafe { self.stack.get_unchecked() };
                    let condition = condition_cell.borrow().is_truthy();

                    if !condition {
                        self.frame_mut().ip += instruction_count;
                    }
                }
                Opcode::ShortJmpIfTrue => {
                    let instruction_count = self.read_index().unwrap();

                    let condition_cell = unsafe { self.stack.get_unchecked() };
                    let condition = condition_cell.borrow().is_truthy();

                    if condition {
                        self.frame_mut().ip += instruction_count;
                    }
                }
                Opcode::ShortJmpIfNullish => {
                    let instruction_count = self.read_index().unwrap();

                    let condition_cell = unsafe { self.stack.get_unchecked() };
                    let condition = condition_cell.borrow().is_nullish();

                    if !condition {
                        self.frame_mut().ip += instruction_count;
                    }
                }
                Opcode::ShortJmp => {
                    let instruction_count = self.read_index().unwrap();
                    self.frame_mut().ip += instruction_count;
                }
                Opcode::BackJmp => {
                    let instruction_count = self.read_index().unwrap();
                    self.frame_mut().ip -= instruction_count;
                }
                Opcode::Pop => {
                    self.stack.pop();
                }
                Opcode::PopUnwindHandler => {
                    self.unwind_handlers.pop();
                }
                Opcode::AdditionAssignment => {
                    let value_cell = self.stack.pop();
                    let target_cell = self.stack.pop();
                    let value = value_cell.borrow();
                    target_cell.borrow_mut().add_assign(&*value);
                    self.stack.push(target_cell);
                }
                Opcode::SubtractionAssignment => {
                    let value_cell = self.stack.pop();
                    let target_cell = self.stack.pop();
                    let value = value_cell.borrow();
                    target_cell.borrow_mut().sub_assign(&*value);
                    self.stack.push(target_cell);
                }
                Opcode::MultiplicationAssignment => {
                    let value_cell = self.stack.pop();
                    let target_cell = self.stack.pop();
                    let value = value_cell.borrow();
                    target_cell.borrow_mut().mul_assign(&*value);
                    self.stack.push(target_cell);
                }
                Opcode::DivisionAssignment => {
                    let value_cell = self.stack.pop();
                    let target_cell = self.stack.pop();
                    let value = value_cell.borrow();
                    target_cell.borrow_mut().div_assign(&*value);
                    self.stack.push(target_cell);
                }
                Opcode::RemainderAssignment => {
                    let value_cell = self.stack.pop();
                    let target_cell = self.stack.pop();
                    let value = value_cell.borrow();
                    target_cell.borrow_mut().rem_assign(&*value);
                    self.stack.push(target_cell);
                }
                Opcode::ExponentiationAssignment => {
                    let value_cell = self.stack.pop();
                    let target_cell = self.stack.pop();
                    let value = value_cell.borrow();
                    target_cell.borrow_mut().pow_assign(&*value);
                    self.stack.push(target_cell);
                }
                Opcode::LeftShiftAssignment => {
                    let value_cell = self.stack.pop();
                    let target_cell = self.stack.pop();
                    let value = value_cell.borrow();
                    target_cell.borrow_mut().left_shift_assign(&*value);
                    self.stack.push(target_cell);
                }
                Opcode::RightShiftAssignment => {
                    let value_cell = self.stack.pop();
                    let target_cell = self.stack.pop();
                    let value = value_cell.borrow();
                    target_cell.borrow_mut().right_shift_assign(&*value);
                    self.stack.push(target_cell);
                }
                Opcode::UnsignedRightShiftAssignment => {
                    let value_cell = self.stack.pop();
                    let target_cell = self.stack.pop();
                    let value = value_cell.borrow();
                    target_cell
                        .borrow_mut()
                        .unsigned_right_shift_assign(&*value);
                    self.stack.push(target_cell);
                }
                Opcode::BitwiseAndAssignment => {
                    let value_cell = self.stack.pop();
                    let target_cell = self.stack.pop();
                    let value = value_cell.borrow();
                    target_cell.borrow_mut().bitwise_and_assign(&*value);
                    self.stack.push(target_cell);
                }
                Opcode::BitwiseOrAssignment => {
                    let value_cell = self.stack.pop();
                    let target_cell = self.stack.pop();
                    let value = value_cell.borrow();
                    target_cell.borrow_mut().bitwise_or_assign(&*value);
                    self.stack.push(target_cell);
                }
                Opcode::BitwiseXorAssignment => {
                    let value_cell = self.stack.pop();
                    let target_cell = self.stack.pop();
                    let value = value_cell.borrow();
                    target_cell.borrow_mut().bitwise_xor_assign(&*value);
                    self.stack.push(target_cell);
                }
                Opcode::LogicalAndAssignment => {
                    let value_cell = self.stack.pop();
                    let target_cell = self.stack.pop();
                    let value = value_cell.borrow();
                    target_cell.borrow_mut().logical_and_assign(&*value);
                    self.stack.push(target_cell);
                }
                Opcode::LogicalOrAssignment => {
                    let value_cell = self.stack.pop();
                    let target_cell = self.stack.pop();
                    let value = value_cell.borrow();
                    target_cell.borrow_mut().logical_or_assign(&*value);
                    self.stack.push(target_cell);
                }
                Opcode::LogicalNullishAssignment => {
                    let value_cell = self.stack.pop();
                    let target_cell = self.stack.pop();
                    let value = value_cell.borrow();
                    target_cell.borrow_mut().nullish_coalescing_assign(&*value);
                    self.stack.push(target_cell);
                }
                Opcode::ConstructorCall => {
                    let param_count = self.read_index().unwrap();
                    let mut params = Vec::new();
                    for _ in 0..param_count {
                        params.push(self.stack.pop());
                    }

                    let func_cell = self.stack.pop();
                    let mut func_cell_ref = func_cell.borrow_mut();
                    let func_cell_kind = func_cell_ref.as_function_mut().unwrap();
                    let this = func_cell_kind.construct(&func_cell);
                    let func = match func_cell_kind {
                        FunctionKind::Native(f) => {
                            if !f.ctor.constructable() {
                                // User tried to invoke non-constructor as a constructor
                                unwind_abort_if_uncaught!(js_std::error::create_error(
                                    MaybeRc::Owned(&format!("{} is not a constructor", f.name)),
                                    self
                                ));
                            }

                            let mut state = CallState::default();
                            let ctx = CallContext {
                                vm: self,
                                args: &mut params,
                                ctor: true,
                                receiver: Some(this.into()),
                                state: &mut state,
                                function_call_response: None,
                            };
                            let result = (f.func)(ctx);

                            match result {
                                Ok(CallResult::Ready(res)) => self.stack.push(res),
                                Err(e) => unwind_abort_if_uncaught!(e),
                                _ => todo!(),
                            }

                            continue;
                        }
                        FunctionKind::Closure(closure) => {
                            closure.func.receiver = Some(Receiver::Bound(this.into()));
                            closure
                        }
                        // There should never be raw user functions
                        _ => unreachable!(),
                    };

                    // By this point we know func_cell is a UserFunction
                    // TODO: get rid of this copy paste and share code with Opcode::FunctionCall

                    let current_sp = self.stack.get_stack_pointer();

                    let state = self.frame_mut().state.take();
                    let frame = Frame {
                        buffer: func.func.buffer.clone(),
                        ip: 0,
                        func: Rc::clone(&func_cell),
                        sp: current_sp,
                        state,
                        resume: None,
                    };

                    self.frames.push(frame);

                    let origin_param_count = func.func.params as usize;
                    let param_count = params.len();

                    for param in params.into_iter().rev() {
                        self.stack.push(param);
                    }

                    for _ in 0..(origin_param_count.saturating_sub(param_count)) {
                        self.stack.push(Value::new(ValueKind::Undefined).into());
                    }
                }
                Opcode::GetThis => {
                    let this = {
                        let frame = self.frame();
                        let func = frame.func.borrow();
                        let raw_func = func
                            .as_function()
                            .and_then(FunctionKind::as_closure)
                            .unwrap();

                        let receiver = raw_func.func.receiver.as_ref().unwrap();
                        receiver.get().clone()
                    };
                    self.stack.push(this);
                }
                Opcode::GetGlobalThis => {
                    self.stack.push(self.global.clone());
                }
                Opcode::EvaluateModule => {
                    let (value_cell, buffer) = {
                        let mut module =
                            self.read_constant().and_then(Constant::into_value).unwrap();

                        let buffer = module
                            .as_function_mut()
                            .unwrap()
                            .as_module_mut()
                            .unwrap()
                            .buffer
                            .take()
                            .unwrap();

                        (module.into(), buffer)
                    };

                    let current_sp = self.stack.get_stack_pointer();
                    self.frame_mut().sp = current_sp;

                    let frame = Frame {
                        func: value_cell,
                        buffer,
                        ip: 0,
                        sp: current_sp,
                        state: None,
                        resume: None,
                    };

                    self.frames.push(frame);
                }
                Opcode::FunctionCall => {
                    let param_count = self.read_index().unwrap();
                    let mut params = Vec::new();
                    for _ in 0..param_count {
                        params.push(self.stack.pop());
                    }

                    let func_cell = self.stack.pop();
                    if let Err(e) = self.begin_function_call(func_cell, params) {
                        unwind_abort_if_uncaught!(e);
                    }
                }
                Opcode::Try => {
                    let catch_idx = self.read_constant().and_then(Constant::into_index).unwrap();
                    let should_capture_error = self.read_op().unwrap() == Opcode::SetLocal;

                    let error_catch_idx = if should_capture_error {
                        Some(self.read_constant().and_then(Constant::into_index).unwrap())
                    } else {
                        None
                    };

                    let current_ip = self.ip();
                    let handler = UnwindHandler {
                        catch_ip: current_ip + catch_idx,
                        catch_value_sp: error_catch_idx,
                        finally_ip: None, // TODO: support finally
                    };
                    self.unwind_handlers.push(handler)
                }
                Opcode::Throw => {
                    let value = self.stack.pop();

                    unwind_abort_if_uncaught!(value);
                }
                Opcode::ReturnModule => {
                    let frame = self.frames.pop();
                    let func_ref = frame.func.borrow();
                    let func = func_ref
                        .as_function()
                        .and_then(FunctionKind::as_module)
                        .unwrap();

                    let exports = if let Some(default) = &func.exports.default {
                        Rc::clone(default)
                    } else {
                        self.create_object().into()
                    };

                    {
                        let mut exports_mut = exports.borrow_mut();
                        for (key, value) in &func.exports.named {
                            exports_mut.set_property(&**key, Rc::clone(value));
                        }
                    }

                    self.stack
                        .discard_multiple(self.stack.get_stack_pointer() - frame.sp);

                    self.stack.set_stack_pointer(frame.sp);
                    self.stack.push(exports);
                }
                Opcode::Return => {
                    // Restore VM state to where we were before the function call happened
                    let mut this = self.frames.pop();

                    if self.frames.get_stack_pointer() == 0 {
                        if self.stack.get_stack_pointer() == 0 {
                            return Ok(None);
                        } else {
                            let value = self.stack.pop();
                            return Ok(Some(value));
                        }
                    }

                    let ret = self.stack.pop();

                    self.stack
                        .discard_multiple(self.stack.get_stack_pointer() - this.sp);

                    self.stack.set_stack_pointer(this.sp);

                    if let Some(mut resume) = this.resume.take() {
                        let mut state = this.state.take().unwrap_or_else(CallState::default);
                        let func_ref = resume.func.borrow();
                        let f = func_ref
                            .as_function()
                            .and_then(FunctionKind::as_native)
                            .unwrap();

                        let context = CallContext {
                            args: &mut resume.args,
                            ctor: resume.ctor,
                            function_call_response: Some(ret),
                            receiver: resume.receiver.clone(),
                            state: &mut state,
                            vm: self,
                        };

                        let ret = (f.func)(context);

                        match ret {
                            Ok(CallResult::Ready(ret)) => self.stack.push(ret),
                            Ok(CallResult::UserFunction(new_func_cell, args)) => self
                                .handle_native_return(
                                    Rc::clone(&resume.func),
                                    new_func_cell,
                                    resume.args,
                                    args,
                                    state,
                                    resume.receiver,
                                ),
                            Err(e) => unwind_abort_if_uncaught!(e),
                        };
                        continue;
                    }

                    let func_ref = this.func.borrow();
                    if let Some(this) = func_ref
                        .as_function()
                        .and_then(FunctionKind::as_closure)
                        .and_then(|c| c.func.receiver.as_ref())
                    {
                        self.stack.push(Rc::clone(this.get()));
                    } else {
                        self.stack.push(ret);
                    }
                }
                Opcode::Less => {
                    let rhs_cell = self.stack.pop();
                    let rhs = rhs_cell.borrow();
                    let lhs_cell = self.stack.pop();
                    let lhs = lhs_cell.borrow();

                    let is_less = matches!(lhs.compare(&rhs), Some(Compare::Less));
                    self.stack.push(self.create_js_value(is_less).into());
                }
                Opcode::LessEqual => {
                    let rhs_cell = self.stack.pop();
                    let rhs = rhs_cell.borrow();
                    let lhs_cell = self.stack.pop();
                    let lhs = lhs_cell.borrow();

                    let is_less_eq = matches!(
                        lhs.compare(&rhs),
                        Some(Compare::Less) | Some(Compare::Equal)
                    );
                    self.stack.push(self.create_js_value(is_less_eq).into());
                }
                Opcode::Greater => {
                    let rhs_cell = self.stack.pop();
                    let rhs = rhs_cell.borrow();
                    let lhs_cell = self.stack.pop();
                    let lhs = lhs_cell.borrow();

                    let is_greater = matches!(lhs.compare(&rhs), Some(Compare::Greater));
                    self.stack.push(self.create_js_value(is_greater).into());
                }
                Opcode::GreaterEqual => {
                    let rhs_cell = self.stack.pop();
                    let rhs = rhs_cell.borrow();
                    let lhs_cell = self.stack.pop();
                    let lhs = lhs_cell.borrow();

                    let is_greater_eq = matches!(
                        lhs.compare(&rhs),
                        Some(Compare::Greater) | Some(Compare::Equal)
                    );
                    self.stack.push(self.create_js_value(is_greater_eq).into());
                }
                Opcode::StaticPropertyAccess => {
                    let property = self.pop_owned().unwrap().into_ident().unwrap();
                    let is_assignment = self.read_index().unwrap() == 1;
                    let target_cell = self.stack.pop();

                    let value = if is_assignment {
                        let maybe_value = Value::get_property(self, &target_cell, &property, None);
                        maybe_value.unwrap_or_else(|| {
                            let mut target = target_cell.borrow_mut();
                            let value = Value::new(ValueKind::Undefined).into();
                            target.set_property(property, Rc::clone(&value));
                            value
                        })
                    } else {
                        Value::unwrap_or_undefined(Value::get_property(
                            self,
                            &target_cell,
                            &property,
                            None,
                        ))
                    };
                    self.stack.push(value);
                }
                Opcode::Equality => {
                    let eq = self.with_lhs_rhs_borrowed(Value::lossy_equal);
                    self.stack.push(self.create_js_value(eq).into());
                }
                Opcode::Inequality => {
                    let eq = self.with_lhs_rhs_borrowed(Value::lossy_equal);
                    self.stack.push(self.create_js_value(!eq).into());
                }
                Opcode::StrictEquality => {
                    let eq = self.with_lhs_rhs_borrowed(Value::strict_equal);
                    self.stack.push(self.create_js_value(eq).into());
                }
                Opcode::StrictInequality => {
                    let eq = self.with_lhs_rhs_borrowed(Value::strict_equal);
                    self.stack.push(self.create_js_value(!eq).into());
                }
                Opcode::Typeof => {
                    let value = self.stack.pop().borrow()._typeof().to_owned();

                    self.stack
                        .push(self.create_js_value(Object::String(value)).into());
                }
                Opcode::PostfixIncrement | Opcode::PostfixDecrement => {
                    let value_cell = self.stack.pop();
                    let mut value = value_cell.borrow_mut();
                    let one = self.create_js_value(1f64);
                    let result = if instruction == Opcode::PostfixIncrement {
                        value.add_assign(&one);
                        value.sub(&one)
                    } else {
                        todo!()
                    };
                    self.stack.push(result.into());
                }
                Opcode::Assignment => {
                    let value_cell = self.stack.pop();
                    let target_cell = self.stack.pop();

                    let value = value_cell.borrow();
                    // TODO: cloning might not be the right thing to do
                    let value = value.clone();

                    let mut target = target_cell.borrow_mut();
                    *target = value;
                    self.stack.push(target_cell.clone());
                }
                Opcode::Void => {
                    self.stack.pop();
                    self.stack.push(Value::new(ValueKind::Undefined).into());
                }
                Opcode::ArrayLiteral => {
                    let element_count = self.read_index().unwrap();
                    let mut elements = Vec::with_capacity(element_count);
                    for _ in 0..element_count {
                        elements.push(self.stack.pop());
                    }
                    self.stack
                        .push(self.create_array(Array::new(elements)).into());
                }
                Opcode::ObjectLiteral => {
                    let property_count = self.read_index().unwrap();

                    let mut fields = HashMap::new();
                    let mut raw_fields = Vec::new();

                    for _ in 0..property_count {
                        let value = self.stack.pop();
                        raw_fields.push(value);
                    }

                    for value in raw_fields.into_iter().rev() {
                        let key = self.read_constant().unwrap().into_ident().unwrap();
                        fields.insert(key.into_boxed_str(), value);
                    }

                    self.stack
                        .push(self.create_object_with_fields(fields).into());
                }
                Opcode::ComputedPropertyAccess => {
                    let property_cell = self.stack.pop();
                    let is_assignment = self.read_index().unwrap() == 1;
                    let target_cell = self.stack.pop();
                    let property = property_cell.borrow();
                    let property_s = property.to_string();

                    let value = if is_assignment {
                        let maybe_value =
                            Value::get_property(self, &target_cell, &*property_s, None);
                        maybe_value.unwrap_or_else(|| {
                            let mut target = target_cell.borrow_mut();
                            let value = Value::new(ValueKind::Undefined).into();
                            target.set_property(property_s.to_string(), Rc::clone(&value));
                            value
                        })
                    } else {
                        Value::unwrap_or_undefined(Value::get_property(
                            self,
                            &target_cell,
                            &*property_s,
                            None,
                        ))
                    };

                    self.stack.push(value);
                }
                Opcode::Continue => {
                    let this = unsafe { self.loops.get_unchecked() };
                    self.frame_mut().ip = this.condition_ip;
                }
                Opcode::Break => {
                    let this = unsafe { self.loops.get_unchecked() };
                    self.frame_mut().ip = this.end_ip;
                }
                Opcode::LoopStart => {
                    let condition_offset =
                        self.read_constant().and_then(Constant::into_index).unwrap();
                    let end_offset = self.read_constant().and_then(Constant::into_index).unwrap();
                    let ip = self.ip();
                    let info = Loop {
                        condition_ip: (ip + condition_offset),
                        end_ip: (ip + end_offset),
                    };
                    self.loops.push(info);
                }
                Opcode::LoopEnd => {
                    self.loops.pop();
                }
                Opcode::ExportDefault => {
                    let export_status = {
                        let value = self.stack.pop();
                        let mut func_ref = self.frame().func.borrow_mut();

                        let maybe_module = func_ref
                            .as_function_mut()
                            .and_then(FunctionKind::as_module_mut);

                        if let Some(module) = maybe_module {
                            module.exports.default = Some(value);
                            true
                        } else {
                            false
                        }
                    };

                    if !export_status {
                        unwind_abort_if_uncaught!(js_std::error::create_error(
                            MaybeRc::Owned("Can only export at the top level in a module"),
                            self
                        ))
                    }
                }
                Opcode::ToPrimitive => {
                    let obj_cell = self.stack.pop();

                    {
                        // If this is already a primitive value, we do not need to try to convert it
                        let obj = obj_cell.borrow();
                        if obj.is_primitive() {
                            self.stack.push(Rc::clone(&obj_cell));
                            continue;
                        }
                    }

                    let to_prim = unwrap_or_unwind!(
                        Value::get_property(self, &obj_cell, "toString", None)
                            .or_else(|| Value::get_property(self, &obj_cell, "valueOf", None)),
                        js_std::error::create_error(
                            MaybeRc::Owned("Cannot convert object to primitive value"),
                            self
                        )
                    );

                    if let Err(e) = self.begin_function_call(to_prim, Vec::new()) {
                        unwind_abort_if_uncaught!(e);
                    }
                }
                _ => unimplemented!("{:?}", instruction),
            };
        }

        Ok(None)
    }
}

impl Drop for VM {
    fn drop(&mut self) {
        self.stack.reset();
        self.frames.reset();
    }
}
