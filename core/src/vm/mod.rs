/// Abstract operations defined in the spec
pub mod abstractions;
/// JavaScript value conversions
pub mod conversions;
/// Frame
pub mod frame;
/// Instruction
pub mod instruction;
/// Stack data structure
pub mod stack;
/// Static/global data
pub mod statics;
/// Runtime upvalues
pub mod upvalue;
/// JavaScript values
pub mod value;

use std::{any::Any, borrow::Cow, cell::RefCell, collections::HashMap};

use instruction::{Instruction, Opcode};
use value::Value;

use crate::{
    agent::Agent,
    compiler::compiler::{self, CompileError, Compiler, FunctionKind as CompilerFunctionKind},
    gc::{Gc, Handle},
    js_std::{self, error::MaybeRc},
    parser::{lexer, token},
    util::{unlikely, MaybeOwned},
    vm::{
        frame::{NativeResume, UnwindHandler},
        upvalue::Upvalue,
        value::{
            array::Array,
            function::{
                CallContext, CallResult, CallState, Closure, Constructor, FunctionKind,
                FunctionType, Receiver, UserFunction,
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

/// An error that may occur during bytecode execution
#[derive(Debug)]
pub enum VMError {
    /// An error was thrown and user code did not catch it
    UncaughtError(Handle<Value>),
}

impl VMError {
    /// Formats this error by taking the `stack` property of the error object
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

/// An error that may occur in one of the previous stages before interpreting
#[derive(Debug)]
pub enum FromStrError<'a> {
    /// Lexer error
    LexError(Vec<lexer::Error<'a>>),
    /// Parser
    ParseError(Vec<token::Error<'a>>),
    /// Compiler error
    CompileError(CompileError<'a>),
}

impl<'a> From<compiler::FromStrError<'a>> for FromStrError<'a> {
    fn from(e: compiler::FromStrError<'a>) -> Self {
        match e {
            compiler::FromStrError::LexError(l) => Self::LexError(l),
            compiler::FromStrError::ParseError(p) => Self::ParseError(p),
        }
    }
}

/// A JavaScript bytecode VM
pub struct VM {
    /// Garbage collector that manages the heap of this VM
    pub(crate) gc: RefCell<Gc<Value>>,
    /// Call stack
    pub(crate) frames: Stack<Frame, 256>,
    /// Async task queue. Processed when execution has finished
    pub(crate) async_frames: Stack<Frame, 256>,
    /// Stack
    pub(crate) stack: Stack<Handle<Value>, 512>,
    /// Global namespace
    pub(crate) global: Handle<Value>,
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
    /// Creates a new VM with a provided agent
    pub fn new_with_agent(func: UserFunction, agent: Box<dyn Agent>) -> Self {
        let mut gc = Gc::new();
        let statics = Statics::new(&mut gc);
        let global = gc.register(Value::from(AnyObject {}));

        let mut frames = Stack::new();
        frames.push(Frame {
            buffer: func.buffer.clone(),
            func: gc.register(Value::from(Closure::new(func))),
            ip: 0,
            sp: 0,
            state: None,
            resume: None,
        });

        let mut vm = Self {
            frames,
            gc: RefCell::new(gc),
            async_frames: Stack::new(),
            stack: Stack::new(),
            global,
            statics,
            unwind_handlers: Stack::new(),
            loops: Stack::new(),
            slot: None,
            agent,
        };
        vm.prepare_stdlib();
        vm
    }

    /// Convenience function for creating a new VM given an input string
    ///
    /// The input string is sent through all previous stages. Any errors that occur
    /// are returned to the caller
    pub fn from_str<'a, A: Agent + 'static>(
        input: &'a str,
        mut agent: Option<A>,
    ) -> Result<Self, FromStrError<'a>> {
        let buffer = Compiler::from_str(
            input,
            agent.as_mut().map(|a| MaybeOwned::Borrowed(a)),
            CompilerFunctionKind::Function,
        )?
        .compile()
        .map_err(FromStrError::CompileError)?;

        let func = UserFunction::new(buffer, 0, FunctionType::Top, 0, Constructor::NoCtor);

        Ok(match agent {
            Some(agent) => Self::new_with_agent(func, Box::new(agent)),
            None => Self::new(func),
        })
    }

    /// Creates a new VM
    pub fn new(func: UserFunction) -> Self {
        Self::new_with_agent(func, Box::new(()))
    }

    /// Returns a reference to the global object
    pub fn global(&self) -> &Handle<Value> {
        &self.global
    }

    /// Sets data slot
    ///
    /// Embedders can use this to store data that may be used throughout native calls
    pub fn set_slot<T: 'static>(&mut self, value: T) {
        self.slot.insert(Box::new(value) as Box<dyn Any>);
    }

    /// Gets slot data and tries to downcast it to T
    pub fn get_slot<T: 'static>(&self) -> Option<&T> {
        let slot = self.slot.as_ref()?;
        slot.downcast_ref::<T>()
    }

    /// Returns a mutable reference to slot data and tries to downcast to T
    pub fn get_slot_mut<T: 'static>(&mut self) -> Option<&mut T> {
        let slot = self.slot.as_mut()?;
        slot.downcast_mut::<T>()
    }

    /// Returns a reference to the current execution frame
    fn frame(&self) -> &Frame {
        unsafe { self.frames.get_unchecked() }
    }

    /// Returns a mutable reference to the current execution frame
    fn frame_mut(&mut self) -> &mut Frame {
        unsafe { self.frames.get_mut_unchecked() }
    }

    /// Returns the current instruction pointer
    fn ip(&self) -> usize {
        self.frame().ip
    }

    /// Estimates whether it makes sense to perform a garbage collection
    fn should_gc(&self) -> bool {
        true
    }

    /// Setup for a GC cycle
    ///
    /// Everything that can possibly be reached from JS code needs to be marked.
    fn mark_roots(&mut self) {
        Value::mark(&self.global);
        self.stack.mark_visited();
        self.frames.mark_visited();
        self.async_frames.mark_visited();
    }

    /// Performs a GC cycle
    // TODO: safe?
    pub unsafe fn perform_gc(&mut self) {
        self.mark_roots();
        unsafe { self.gc.borrow_mut().sweep() };
    }

    /// Returns the bytecode buffer of the current execution frame
    fn buffer(&self) -> &[Instruction] {
        &self.frame().buffer
    }

    /// Checks whether the VM has reached the end of this buffer
    fn is_eof(&self) -> bool {
        // If we ever somehow jump too far, that's a bug
        // In debug builds, we can afford to assert this
        debug_assert!(self.ip() <= self.buffer().len());
        self.ip() >= self.buffer().len()
    }

    /// Returns the next instruction
    fn next(&mut self) -> Option<&Instruction> {
        if self.is_eof() {
            return None;
        }

        self.frame_mut().ip += 1;

        Some(&self.buffer()[self.ip() - 1])
    }

    /// Reads a constant
    fn read_constant(&mut self) -> Option<Constant> {
        self.next().cloned().map(|x| x.into_operand())
    }

    /// Reads an opode
    fn read_op(&mut self) -> Option<Opcode> {
        self.next().cloned().map(|x| x.into_op())
    }

    /// Reads a user function
    fn read_user_function(&mut self) -> Option<UserFunction> {
        self.read_constant()
            .and_then(|c| c.into_value())
            .and_then(|v| v.into_object())
            .and_then(|o| match o {
                Object::Function(FunctionKind::User(f)) => Some(f),
                _ => None,
            })
    }

    /// Reads a number
    fn read_number(&mut self) -> f64 {
        self.stack.pop().borrow().as_number()
    }

    /// Reads an index
    fn read_index(&mut self) -> Option<usize> {
        self.stack
            .pop()
            .borrow()
            .as_constant()
            .and_then(|c| c.as_index())
    }

    fn pop_owned(&mut self) -> Option<Value> {
        Some(self.stack.pop().borrow().clone())
    }

    fn read_lhs_rhs(&mut self) -> (Handle<Value>, Handle<Value>) {
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

    /// Creates a JavaScript object
    pub fn create_object(&self) -> Value {
        self.create_js_value(AnyObject {})
    }

    /// Creates a JavaScript object with its [[Prototype]] set to null
    pub fn create_null_object(&self) -> Value {
        let mut o = Value::from(AnyObject {});
        o.detect_internal_properties(self);
        // Override [[Prototype]]
        o.proto = None;
        o
    }

    /// Creates a JavaScript object with provided fields
    pub fn create_object_with_fields(
        &self,
        fields: impl Into<HashMap<Box<str>, Handle<Value>>>,
    ) -> Value {
        let mut o = self.create_object();
        o.fields = fields.into();
        o
    }

    /// Creates a JavaScript value
    pub fn create_js_value(&self, value: impl Into<Value>) -> Value {
        let mut value = value.into();
        value.detect_internal_properties(self);
        value
    }

    /// Creates a JavaScript array
    pub fn create_array(&self, arr: Array) -> Value {
        let mut o = Value::from(arr);
        o.proto = Some(Handle::clone(&self.statics.array_proto));
        o.constructor = Some(Handle::clone(&self.statics.array_ctor));
        o
    }

    #[rustfmt::skip]
    fn prepare_stdlib(&mut self) {
        // All values that live in self.statics do not have a [[Prototype]] set
        // so we do it here
        fn patch_value(this: &VM, value: &Handle<Value>) {
            value.borrow_mut().detect_internal_properties(this);
        }
        
        fn patch_constructor(this: &VM, func: &Handle<Value>, prototype: &Handle<Value>) {
            let mut func_ref = func.borrow_mut();
            let real_func = func_ref.as_function_mut().unwrap();
            real_func.set_prototype(Handle::clone(prototype));
            func_ref.detect_internal_properties(this);
        }

        let mut global = self.global.borrow_mut();
        global.detect_internal_properties(self);
        // TODO: attaching globalThis causes a reference cycle and memory leaks
        // We somehow need to have
        // global.set_property("globalThis", self.global.clone());
        global.set_property("globalThis", Handle::clone(&self.global));

        patch_value(self, &self.statics.error_proto);
        patch_value(self, &self.statics.function_proto);
        patch_value(self, &self.statics.promise_proto);
        patch_value(self, &self.statics.boolean_proto);
        patch_value(self, &self.statics.number_proto);
        
        {
            let mut o = self.statics.string_proto.borrow_mut();
            o.detect_internal_properties(self);
            o.set_property("charAt", Handle::clone(&self.statics.string_char_at));
            o.set_property("charCodeAt", Handle::clone(&self.statics.string_char_code_at));
            o.set_property("endsWith", Handle::clone(&self.statics.string_ends_with));
            o.set_property("anchor", Handle::clone(&self.statics.string_anchor));
            o.set_property("big", Handle::clone(&self.statics.string_big));
            o.set_property("blink", Handle::clone(&self.statics.string_blink));
            o.set_property("bold", Handle::clone(&self.statics.string_bold));
            o.set_property("fixed", Handle::clone(&self.statics.string_fixed));
            o.set_property("fontcolor", Handle::clone(&self.statics.string_fontcolor));
            o.set_property("fontsize", Handle::clone(&self.statics.string_fontsize));
            o.set_property("italics", Handle::clone(&self.statics.string_italics));
            o.set_property("link", Handle::clone(&self.statics.string_link));
            o.set_property("small", Handle::clone(&self.statics.string_small));
            o.set_property("strike", Handle::clone(&self.statics.string_strike));
            o.set_property("sub", Handle::clone(&self.statics.string_sub));
            o.set_property("sup", Handle::clone(&self.statics.string_sup));
        }

        {
            let mut o = self.statics.array_proto.borrow_mut();
            o.detect_internal_properties(self);
            o.set_property("push", Handle::clone(&self.statics.array_push));
            o.set_property("concat", Handle::clone(&self.statics.array_concat));
            o.set_property("map", Handle::clone(&self.statics.array_map));
            o.set_property("every", Handle::clone(&self.statics.array_every));
            o.set_property("fill", Handle::clone(&self.statics.array_fill));
            o.set_property("filter", Handle::clone(&self.statics.array_filter));
            o.set_property("find", Handle::clone(&self.statics.array_find));
            o.set_property("findIndex", Handle::clone(&self.statics.array_find_index));
            o.set_property("flat", Handle::clone(&self.statics.array_flat));
            o.set_property("forEach", Handle::clone(&self.statics.array_for_each));
            o.set_property("from", Handle::clone(&self.statics.array_from));
            o.set_property("includes", Handle::clone(&self.statics.array_includes));
            o.set_property("indexOf", Handle::clone(&self.statics.array_index_of));
            o.set_property("join", Handle::clone(&self.statics.array_join));
            o.set_property("lastIndexOf", Handle::clone(&self.statics.array_last_index_of));
            o.set_property("of", Handle::clone(&self.statics.array_of));
            o.set_property("pop", Handle::clone(&self.statics.array_pop));
            o.set_property("reduce", Handle::clone(&self.statics.array_reduce));
            o.set_property("reduceRight", Handle::clone(&self.statics.array_reduce_right));
            o.set_property("reverse", Handle::clone(&self.statics.array_reverse));
            o.set_property("shift", Handle::clone(&self.statics.array_shift));
            o.set_property("slice", Handle::clone(&self.statics.array_slice));
            o.set_property("some", Handle::clone(&self.statics.array_some));
            o.set_property("sort", Handle::clone(&self.statics.array_sort));
            o.set_property("splice", Handle::clone(&self.statics.array_splice));
            o.set_property("unshift", Handle::clone(&self.statics.array_unshift));
        }

        {
            let mut array_ctor = self.statics.array_ctor.borrow_mut();
            array_ctor.set_property("isArray", Handle::clone(&self.statics.array_is_array));
        }

        {
            let mut promise_ctor = self.statics.promise_ctor.borrow_mut();
            promise_ctor.set_property("resolve", Handle::clone(&self.statics.promise_resolve));
            promise_ctor.set_property("reject", Handle::clone(&self.statics.promise_reject));
        }

        {
            let mut o = self.statics.weakset_proto.borrow_mut();
            o.detect_internal_properties(self);
            o.set_property("has", Handle::clone(&self.statics.weakset_has));
            o.set_property("add", Handle::clone(&self.statics.weakset_add));
            o.set_property("delete", Handle::clone(&self.statics.weakset_delete));
        }

        {
            let mut o = self.statics.weakmap_proto.borrow_mut();
            o.detect_internal_properties(self);
            o.set_property("has", Handle::clone(&self.statics.weakmap_has));
            o.set_property("add", Handle::clone(&self.statics.weakmap_add));
            o.set_property("get", Handle::clone(&self.statics.weakmap_get));
            o.set_property("delete", Handle::clone(&self.statics.weakmap_delete));
        }

        {
            let mut object_proto = self.statics.object_proto.borrow_mut();
            object_proto.constructor = Some(Handle::clone(&self.statics.object_ctor));
            object_proto.proto = Some(Value::new(ValueKind::Null).into_handle(self));
            object_proto.set_property("toString", Handle::clone(&self.statics.object_to_string));
        }

        // Constructors
        patch_constructor(self, &self.statics.boolean_ctor, &self.statics.boolean_proto);
        patch_constructor(self, &self.statics.number_ctor, &self.statics.number_proto);
        patch_constructor(self, &self.statics.string_ctor, &self.statics.string_proto);
        patch_constructor(self, &self.statics.function_ctor, &self.statics.function_proto);
        patch_constructor(self, &self.statics.array_ctor, &self.statics.array_proto);
        patch_constructor(self, &self.statics.weakset_ctor, &self.statics.weakset_proto);
        patch_constructor(self, &self.statics.weakmap_ctor, &self.statics.weakmap_proto);
        patch_constructor(self, &self.statics.object_ctor, &self.statics.object_proto);
        patch_constructor(self, &self.statics.error_ctor, &self.statics.error_proto);
        patch_constructor(self, &self.statics.promise_ctor, &self.statics.promise_proto);
        // Other functions/methods
        patch_value(self, &self.statics.isnan);
        patch_value(self, &self.statics.object_define_property);
        patch_value(self, &self.statics.object_get_own_property_names);
        patch_value(self, &self.statics.object_to_string);
        patch_value(self, &self.statics.isnan);
        patch_value(self, &self.statics.console_log);
        patch_value(self, &self.statics.array_push);
        patch_value(self, &self.statics.math_pow);
        patch_value(self, &self.statics.math_abs);
        patch_value(self, &self.statics.math_ceil);
        patch_value(self, &self.statics.math_floor);
        patch_value(self, &self.statics.math_max);
        patch_value(self, &self.statics.math_random);
        patch_value(self, &self.statics.weakset_has);
        patch_value(self, &self.statics.weakset_add);
        patch_value(self, &self.statics.weakset_delete);
        patch_value(self, &self.statics.weakmap_has);
        patch_value(self, &self.statics.weakmap_add);
        patch_value(self, &self.statics.weakmap_get);
        patch_value(self, &self.statics.weakmap_delete);
        patch_value(self, &self.statics.json_parse);
        patch_value(self, &self.statics.json_stringify);
        patch_value(self, &self.statics.string_char_at);
        patch_value(self, &self.statics.string_char_code_at);
        patch_value(self, &self.statics.string_ends_with);
        patch_value(self, &self.statics.string_anchor);
        patch_value(self, &self.statics.string_big);
        patch_value(self, &self.statics.string_blink);
        patch_value(self, &self.statics.string_bold);
        patch_value(self, &self.statics.string_fixed);
        patch_value(self, &self.statics.string_fontcolor);
        patch_value(self, &self.statics.string_fontsize);
        patch_value(self, &self.statics.string_italics);
        patch_value(self, &self.statics.string_link);
        patch_value(self, &self.statics.string_small);
        patch_value(self, &self.statics.string_strike);
        patch_value(self, &self.statics.string_sub);
        patch_value(self, &self.statics.string_sup);
        patch_value(self, &self.statics.promise_resolve);
        patch_value(self, &self.statics.promise_reject);

        global.set_property("NaN", self.create_js_value(f64::NAN).into_handle(self));
        global.set_property("Infinity", self.create_js_value(f64::INFINITY).into_handle(self));
        global.set_property("isNaN", self.statics.isnan.clone());

        {
            let mut object_ctor = self.statics.object_ctor.borrow_mut();
            object_ctor.set_property("defineProperty", self.statics.object_define_property.clone());
            object_ctor.set_property("getOwnPropertyNames", self.statics.object_get_own_property_names.clone());
            object_ctor.set_property("getPrototypeOf", self.statics.object_get_prototype_of.clone());
            global.set_property("Object", Handle::clone(&self.statics.object_ctor));
        }

        {
            let mut math_obj = self.create_object();
            math_obj.set_property("pow", Handle::clone(&self.statics.math_pow));
            math_obj.set_property("abs", Handle::clone(&self.statics.math_abs));
            math_obj.set_property("ceil", Handle::clone(&self.statics.math_ceil));
            math_obj.set_property("floor", Handle::clone(&self.statics.math_floor));
            math_obj.set_property("max", Handle::clone(&self.statics.math_max));
            math_obj.set_property("random", Handle::clone(&self.statics.math_random));

            math_obj.set_property("PI", self.create_js_value(std::f64::consts::PI).into_handle(self));
            math_obj.set_property("E", self.create_js_value(std::f64::consts::E).into_handle(self));
            math_obj.set_property("LN10", self.create_js_value(std::f64::consts::LN_10).into_handle(self));
            math_obj.set_property("LN2", self.create_js_value(std::f64::consts::LN_2).into_handle(self));
            math_obj.set_property("LOG10E", self.create_js_value(std::f64::consts::LOG10_E).into_handle(self));
            math_obj.set_property("LOG2E", self.create_js_value(std::f64::consts::LOG2_E).into_handle(self));
            math_obj.set_property("SQRT2",self.create_js_value(std::f64::consts::SQRT_2).into_handle(self));
            global.set_property("Math", math_obj.into_handle(self));
        }

        {
            let mut json_obj = self.create_object();
            json_obj.set_property("parse", Handle::clone(&self.statics.json_parse));
            json_obj.set_property("stringify", Handle::clone(&self.statics.json_stringify));
            global.set_property("JSON", json_obj.into_handle(self));
        }

        {
            let mut console_obj = self.create_object();
            console_obj.set_property("log", Handle::clone(&self.statics.console_log));
            global.set_property("console", console_obj.into_handle(self));
        }

        global.set_property("Error", self.statics.error_ctor.clone());
        global.set_property("Boolean", self.statics.boolean_ctor.clone());
        global.set_property("Number", self.statics.number_ctor.clone());
        global.set_property("String", self.statics.string_ctor.clone());
        global.set_property("Function", self.statics.function_ctor.clone());
        global.set_property("Array", self.statics.array_ctor.clone());
        global.set_property("WeakSet", self.statics.weakset_ctor.clone());
        global.set_property("WeakMap", self.statics.weakmap_ctor.clone());
        global.set_property("Promise", self.statics.promise_ctor.clone());
    }

    fn unwind(&mut self, value: Handle<Value>) -> Result<(), Handle<Value>> {
        // TODO: clean up resources caused by this unwind
        if self.unwind_handlers.get_stack_pointer() == 0 {
            return Err(value);
        }

        // Try to get the last unwind handler
        let handler = self.unwind_handlers.pop();

        // Go back the call stack back to where the last try/catch block lives
        let this_frame_pointer = self.frames.get_stack_pointer();
        self.frames
            .discard_multiple(this_frame_pointer - handler.frame_pointer);

        // ... and update the instruction pointer to the catch ip
        self.frame_mut().ip = handler.catch_ip;

        if let Some(catch_value_sp) = handler.catch_value_sp {
            // If this handler has a catch value associated, we want to set it
            self.stack
                .set_relative(self.frame().sp, catch_value_sp, value);
        }

        Ok(())
    }

    /// Generates a formatted stacktrace
    ///
    /// This prints the call stack, specifically function names.
    /// It is used by the `Error` constructor.
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
        old_func: Handle<Value>,
        new_func: Handle<Value>,
        old_args: Vec<Handle<Value>>,
        new_args: Vec<Handle<Value>>,
        state: CallState<Box<dyn Any>>,
        receiver: Option<Handle<Value>>,
    ) {
        let func_ref = new_func.borrow();
        match func_ref.as_function().unwrap() {
            FunctionKind::Closure(closure) => {
                let sp = self.stack.get_stack_pointer();
                self.frame_mut().sp = sp;

                let frame = Frame {
                    buffer: closure.func.buffer.clone(),
                    ip: 0,
                    func: Handle::clone(&new_func),
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
        func_cell: Handle<Value>,
        mut params: Vec<Handle<Value>>,
    ) -> Result<(), Handle<Value>> {
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
                        Handle::clone(&func_cell),
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
            self.stack
                .push(Value::new(ValueKind::Undefined).into_handle(self));
        }

        Ok(())
    }

    /// Runs all async tasks in the queue
    ///
    /// This should only be called when the frame stack is empty
    pub fn run_async_tasks(&mut self) {
        debug_assert!(self.frames.get_stack_pointer() == 0);
        let async_frames = self.async_frames.take();
        self.frames = async_frames;
        // Uncaught errors and return values in async tasks are swallowed.
        let _ = self.interpret();
    }

    /// Queues a task/frame for execution when the call stack is empty
    pub fn queue_async_task(&mut self, frame: Frame) {
        self.async_frames.push(frame);
    }

    /// Starts interpreting bytecode
    pub fn interpret(&mut self) -> Result<Option<Handle<Value>>, VMError> {
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
            if unlikely(self.should_gc()) {
                unsafe { self.perform_gc() };
            }

            let instruction = self.buffer()[self.ip()].as_op();

            self.frame_mut().ip += 1;

            match instruction {
                Opcode::Eof => return Ok(None),
                Opcode::Constant => {
                    let mut constant = self.read_constant().map(|c| c.try_into_value()).unwrap();

                    // Values emitted by the compiler do not have a [[Prototype]] set
                    // so we need to do that here when pushing a value onto the stack
                    constant.detect_internal_properties(self);

                    self.stack.push(constant.into_handle(self));
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

                    self.stack.push(
                        self.create_js_value(FunctionKind::Closure(closure))
                            .into_handle(self),
                    );
                }
                Opcode::Negate => {
                    let maybe_number = self.read_number();

                    self.stack
                        .push(self.create_js_value(-maybe_number).into_handle(self));
                }
                Opcode::Positive => {
                    let maybe_number = self.read_number();

                    self.stack
                        .push(self.create_js_value(maybe_number).into_handle(self));
                }
                Opcode::LogicalNot => {
                    let is_truthy = self.stack.pop().borrow().is_truthy();

                    self.stack
                        .push(self.create_js_value(!is_truthy).into_handle(self));
                }
                Opcode::Add => {
                    let result = self.with_lhs_rhs_borrowed(Value::add).into_handle(self);
                    self.stack.push(result);
                }
                Opcode::Sub => {
                    let result = self.with_lhs_rhs_borrowed(Value::sub).into_handle(self);
                    self.stack.push(result);
                }
                Opcode::Mul => {
                    let result = self.with_lhs_rhs_borrowed(Value::mul).into_handle(self);
                    self.stack.push(result);
                }
                Opcode::Div => {
                    let result = self.with_lhs_rhs_borrowed(Value::div).into_handle(self);
                    self.stack.push(result);
                }
                Opcode::Rem => {
                    let result = self.with_lhs_rhs_borrowed(Value::rem).into_handle(self);
                    self.stack.push(result);
                }
                Opcode::Exponentiation => {
                    let result = self.with_lhs_rhs_borrowed(Value::pow).into_handle(self);
                    self.stack.push(result);
                }
                Opcode::LeftShift => {
                    let result = self
                        .with_lhs_rhs_borrowed(Value::left_shift)
                        .into_handle(self);
                    self.stack.push(result);
                }
                Opcode::RightShift => {
                    let result = self
                        .with_lhs_rhs_borrowed(Value::right_shift)
                        .into_handle(self);
                    self.stack.push(result);
                }
                Opcode::UnsignedRightShift => {
                    let result = self
                        .with_lhs_rhs_borrowed(Value::unsigned_right_shift)
                        .into_handle(self);
                    self.stack.push(result);
                }
                Opcode::BitwiseAnd => {
                    let result = self
                        .with_lhs_rhs_borrowed(Value::bitwise_and)
                        .into_handle(self);
                    self.stack.push(result);
                }
                Opcode::BitwiseOr => {
                    let result = self
                        .with_lhs_rhs_borrowed(Value::bitwise_or)
                        .into_handle(self);
                    self.stack.push(result);
                }
                Opcode::BitwiseXor => {
                    let result = self
                        .with_lhs_rhs_borrowed(Value::bitwise_xor)
                        .into_handle(self);
                    self.stack.push(result);
                }
                Opcode::BitwiseNot => {
                    let result = self.with_lhs_borrowed(Value::bitwise_not).into_handle(self);
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
                    global.set_property(name, Value::new(ValueKind::Undefined).into_handle(self));
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
                        Value::new(ValueKind::Undefined).into_handle(self),
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
                            let receiver = Some(this.into_handle(self));
                            let ctx = CallContext {
                                vm: self,
                                args: &mut params,
                                ctor: true,
                                receiver,
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
                            closure.func.receiver = Some(Receiver::Bound(this.into_handle(self)));
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
                        func: Handle::clone(&func_cell),
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
                        self.stack
                            .push(Value::new(ValueKind::Undefined).into_handle(self));
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

                        (module.into_handle(self), buffer)
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
                        frame_pointer: self.frames.get_stack_pointer(),
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
                        Handle::clone(default)
                    } else {
                        self.create_object().into_handle(self)
                    };

                    {
                        let mut exports_mut = exports.borrow_mut();
                        for (key, value) in &func.exports.named {
                            exports_mut.set_property(&**key, Handle::clone(value));
                        }
                    }

                    self.stack
                        .discard_multiple(self.stack.get_stack_pointer() - frame.sp);

                    unsafe { self.stack.set_stack_pointer(frame.sp) };
                    self.stack.push(exports);
                }
                Opcode::Return => {
                    // We might be in a try block, in which case we need to remove the handler
                    let maybe_tc_frame_pointer =
                        unsafe { self.unwind_handlers.get() }.map(|c| c.frame_pointer);

                    let frame_pointer = self.frames.get_stack_pointer();

                    if maybe_tc_frame_pointer == Some(frame_pointer) {
                        self.unwind_handlers.pop();
                    }

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

                    unsafe { self.stack.set_stack_pointer(this.sp) };

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
                                    Handle::clone(&resume.func),
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
                        self.stack.push(Handle::clone(this.get()));
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
                    self.stack
                        .push(self.create_js_value(is_less).into_handle(self));
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
                    self.stack
                        .push(self.create_js_value(is_less_eq).into_handle(self));
                }
                Opcode::Greater => {
                    let rhs_cell = self.stack.pop();
                    let rhs = rhs_cell.borrow();
                    let lhs_cell = self.stack.pop();
                    let lhs = lhs_cell.borrow();

                    let is_greater = matches!(lhs.compare(&rhs), Some(Compare::Greater));
                    self.stack
                        .push(self.create_js_value(is_greater).into_handle(self));
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
                    self.stack
                        .push(self.create_js_value(is_greater_eq).into_handle(self));
                }
                Opcode::StaticPropertyAccess => {
                    let property = self.pop_owned().unwrap().into_ident().unwrap();
                    let is_assignment = self.read_index().unwrap() == 1;
                    let target_cell = self.stack.pop();

                    let value = if is_assignment {
                        let maybe_value = Value::get_property(self, &target_cell, &property, None);
                        maybe_value.unwrap_or_else(|| {
                            let mut target = target_cell.borrow_mut();
                            let value = Value::new(ValueKind::Undefined).into_handle(self);
                            target.set_property(property, Handle::clone(&value));
                            value
                        })
                    } else {
                        Value::unwrap_or_undefined(
                            Value::get_property(self, &target_cell, &property, None),
                            self,
                        )
                    };
                    self.stack.push(value);
                }
                Opcode::Equality => {
                    let eq = self.with_lhs_rhs_borrowed(Value::lossy_equal);
                    self.stack.push(self.create_js_value(eq).into_handle(self));
                }
                Opcode::Inequality => {
                    let eq = self.with_lhs_rhs_borrowed(Value::lossy_equal);
                    self.stack.push(self.create_js_value(!eq).into_handle(self));
                }
                Opcode::StrictEquality => {
                    let eq = self.with_lhs_rhs_borrowed(Value::strict_equal);
                    self.stack.push(self.create_js_value(eq).into_handle(self));
                }
                Opcode::StrictInequality => {
                    let eq = self.with_lhs_rhs_borrowed(Value::strict_equal);
                    self.stack.push(self.create_js_value(!eq).into_handle(self));
                }
                Opcode::Typeof => {
                    let value = self.stack.pop().borrow()._typeof().to_owned();

                    self.stack.push(
                        self.create_js_value(Object::String(value))
                            .into_handle(self),
                    );
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
                    self.stack.push(result.into_handle(self));
                }
                Opcode::Assignment => {
                    let value_cell = self.stack.pop();
                    let target_cell = self.stack.pop();

                    let value = value_cell.borrow();
                    // TODO: cloning might not be the right thing to do
                    let value = value.clone();

                    let mut target = target_cell.borrow_mut();
                    **target = value;
                    self.stack.push(target_cell.clone());
                }
                Opcode::Void => {
                    self.stack.pop();
                    self.stack
                        .push(Value::new(ValueKind::Undefined).into_handle(self));
                }
                Opcode::ArrayLiteral => {
                    let element_count = self.read_index().unwrap();
                    let mut elements = Vec::with_capacity(element_count);
                    for _ in 0..element_count {
                        elements.push(self.stack.pop());
                    }
                    self.stack
                        .push(self.create_array(Array::new(elements)).into_handle(self));
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
                        .push(self.create_object_with_fields(fields).into_handle(self));
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
                            let value = Value::new(ValueKind::Undefined).into_handle(self);
                            target.set_property(property_s.to_string(), Handle::clone(&value));
                            value
                        })
                    } else {
                        Value::unwrap_or_undefined(
                            Value::get_property(self, &target_cell, &*property_s, None),
                            self,
                        )
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
                            self.stack.push(Handle::clone(&obj_cell));
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
                Opcode::Debugger => self.agent.debugger(),
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
        self.async_frames.reset();
    }
}
