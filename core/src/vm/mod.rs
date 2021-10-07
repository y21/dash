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
/// VM instruction dispatching
pub mod dispatch;

use std::{any::Any, borrow::Cow, cell::RefCell, collections::HashMap, fmt::Debug};

use instruction::{Instruction, Opcode};
use value::Value;

use crate::{EvalError, agent::Agent, compiler::{compiler::{self, CompileError, Compiler, FunctionKind as CompilerFunctionKind}, constants::ConstantPool, instruction::to_vm_instructions}, gc::{Gc, Handle}, parser::{lexer, token}, util::{unlikely, MaybeOwned}, vm::{dispatch::DispatchResult, frame::UnwindHandler, value::{ValueKind, array::Array, function::{CallContext, FunctionKind, UserFunction}, generator::GeneratorIterator}}};
use crate::js_std;

use self::{frame::{Frame, Loop}, instruction::Constant, stack::Stack, statics::Statics, value::{PropertyKey, object::Object}};

// Force garbage collection at 10000 objects by default
const DEFAULT_GC_OBJECT_COUNT_THRESHOLD: usize = 10000;

/// An error that may occur during bytecode execution
#[derive(Debug)]
pub enum VMError {
    /// An error was thrown and user code did not catch it
    UncaughtError(Handle<Value>),
}

/// An owned error that may occur during bytecode execution
///
/// The contained value is cloned.
#[derive(Debug)]
pub enum OwnedVMError {
    /// An error was thrown and user code did not catch it
    UncaughtError(Value), // TODO: not static
}

impl From<VMError> for OwnedVMError {
    fn from(e: VMError) -> Self {
        match e {
            VMError::UncaughtError(e) => Self::UncaughtError(unsafe { e.borrow_unbounded() }.clone())
        }
    }
}

impl VMError {
    /// Returns the inner value of the error
    pub fn into_value(self) -> Handle<Value> {
        match self {
            Self::UncaughtError(err) => err
        }
    }

    /// Formats this error by taking the `stack` property of the error object
    pub fn to_string(&self) -> Cow<str> {
        match self {
            Self::UncaughtError(err_cell) => {
                let err = unsafe { err_cell.borrow_unbounded() };
                let stack_cell = err.get_field(PropertyKey::from("stack")).unwrap();
                let stack_ref = unsafe { stack_cell.borrow_unbounded() };
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
    /// Garbage collector specifically for constant values produced by the compiler
    pub(crate) constants_gc: Gc<Value>,
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
    /// This realms symbol registry
    // TODO: weak references
    pub(crate) symbols: HashMap<Box<str>, Handle<Value>>,
    /// A copy of the garbage collector's unique marker
    gc_marker: *const (),
    gc_object_threshold: usize,
}

impl Debug for VM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VM")
    }
}

impl VM {
    /// Creates a new VM with a provided agent
    pub fn new_with_agent(agent: Box<dyn Agent>) -> Self {
        let mut gc = Gc::new();
        let gc_marker = gc.marker.get();
        let statics = Statics::new(&mut gc);
        let global = gc.register(Value::from(Object::Ordinary));

        let mut vm = Self {
            frames: Stack::new(),
            gc: RefCell::new(gc),
            constants_gc: Gc::new(),
            async_frames: Stack::new(),
            stack: Stack::new(),
            unwind_handlers: Stack::new(),
            loops: Stack::new(),
            gc_object_threshold: DEFAULT_GC_OBJECT_COUNT_THRESHOLD,
            symbols: HashMap::new(),
            global,
            statics,
            slot: None,
            agent,
            gc_marker,
        };
        vm.prepare_stdlib();
        vm
    }

    pub(crate) fn get_gc_marker(&self) -> *const () {
        self.gc_marker
    }

    /// Convenience function for creating a new VM given an input string
    ///
    /// The input string is sent through all previous stages. Any errors that occur
    /// are returned to the caller
    pub fn from_str<'a, A: Agent + 'static>(
        input: &'a str,
        mut agent: Option<A>,
    ) -> Result<Self, FromStrError<'a>> {
        let (buffer, constants, mut gc) = Compiler::from_str(
            input,
            agent.as_mut().map(|a| MaybeOwned::Borrowed(a)),
            CompilerFunctionKind::Function,
        )?
        .compile()
        .map_err(FromStrError::CompileError)?;

        let mut vm = match agent {
            Some(agent) => Self::new_with_agent(Box::new(agent)),
            None => Self::new(),
        };

        vm.mark_constants(&mut gc);

        vm.constants_gc.transfer(gc);

        let frame = Frame::from_buffer(false, to_vm_instructions(buffer), constants, &vm);
        vm.frames.push(frame);

        Ok(vm)
    }

    /// Creates a new VM
    pub fn new() -> Self {
        Self::new_with_agent(Box::new(()))
    }

    /// Returns a reference to the global object
    pub fn global(&self) -> &Handle<Value> {
        &self.global
    }

    /// Sets data slot
    ///
    /// Embedders can use this to store data that may be used throughout native calls
    pub fn set_slot<T: 'static>(&mut self, value: T) {
        let _ = self.slot.insert(Box::new(value) as Box<dyn Any>);
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

    /// Sets the object threshold for garbage collection
    ///
    /// The VM may perform a GC cycle at a GC point if the number of objects in the heap
    /// is greater than this threshold
    ///
    /// Note: the VM will automatically adjust this value to a more appropriate number
    /// as GC cycles occur
    pub fn set_gc_object_threshold(&mut self, threshold: usize) {
        self.gc_object_threshold = threshold;
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
    unsafe fn should_gc(&self) -> bool {
        unsafe { (&*self.gc.as_ptr()).heap.len >= self.gc_object_threshold }
    }

    /// Setup for a GC cycle
    ///
    /// Everything that can possibly be reached from JS code needs to be marked.
    fn mark_roots(&mut self) {
        Value::mark(&self.global);
        self.stack.mark_visited();
        self.frames.mark_visited();
        self.async_frames.mark_visited();

        // Generator iterator prototype is referenced nowhere else,
        // so we must mark it explicitly here
        // TODO: this is incorrect behavior
        // (function*(){}).constructor !== Function
        // GeneratorFunction.prototype should point to generator_iterator_proto
        Value::mark(&self.statics.generator_iterator_proto);
    }

    /// Performs a GC cycle
    // TODO: safe?
    pub unsafe fn perform_gc(&mut self) {
        self.mark_roots();
        let new_object_count = unsafe {
            let mut gc = self.gc.borrow_mut();
            gc.sweep();
            gc.heap.len
        };

        // Adjust threshold
        self.gc_object_threshold = new_object_count * 2;
    }

    /// Returns the bytecode buffer of the current execution frame
    fn buffer(&self) -> &[Instruction] {
        &self.frame().buffer
    }

    /// Returns the constant pool of the current execution frame
    fn constants(&self) -> &[Constant] {
        // TODO: resolve unsoundness
        let value = unsafe { &*self.frame().func.as_ptr() };
        let func = unsafe { &*value.as_ptr() };

        func.as_function()
            .and_then(|x| x.constants())
            .unwrap()
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
        let index = self.next().cloned().map(|x| x.into_operand())?;
        self.constants().get(index as usize).cloned()
    }

    /// Reads an opode
    fn read_op(&mut self) -> Option<Opcode> {
        self.next().map(|x| x.as_op())
    }

    /// Reads a user function
    fn read_user_function(&mut self) -> Option<UserFunction> {
        self.read_constant()
            .and_then(Constant::into_function)
            .and_then(FunctionKind::into_user)
    }

    /// Reads a number
    fn read_number(&mut self) -> f64 {
        unsafe { self.stack.pop().borrow_unbounded() }.as_number()
    }

    /// Reads an index
    fn read_index(&mut self) -> Option<usize> {
        self.read_constant()
            .and_then(Constant::into_index)
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
        let lhs = unsafe { lhs_cell.borrow_unbounded() };
        func(&*lhs)
    }

    fn with_lhs_rhs_borrowed<F, T>(&mut self, func: F) -> T
    where
        F: Fn(&Value, &Value) -> T,
    {
        let (lhs_cell, rhs_cell) = self.read_lhs_rhs();
        let lhs = unsafe { lhs_cell.borrow_unbounded() };
        let rhs = unsafe { rhs_cell.borrow_unbounded() };
        func(&*lhs, &*rhs)
    }

    /// Creates a JavaScript object
    pub fn create_object(&self) -> Value {
        self.create_js_value(Object::Ordinary)
    }

    /// Creates a JavaScript object with its [[Prototype]] set to null
    pub fn create_null_object(&self) -> Value {
        let mut o = Value::from(Object::Ordinary);
        o.detect_internal_properties(self);
        // Override [[Prototype]]
        o.proto = None;
        o
    }

    /// Creates a JavaScript object with provided fields
    pub fn create_object_with_fields(
        &self,
        fields: impl Into<HashMap<PropertyKey<'static>, Handle<Value>>>,
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
            unsafe { value.borrow_mut_unbounded() }.detect_internal_properties(this);
        }
        
        fn patch_constructor(this: &VM, func: &Handle<Value>, prototype: &Handle<Value>) {
            let mut func_ref = unsafe { func.borrow_mut_unbounded() };
            let real_func = func_ref.as_function_mut().unwrap();
            real_func.set_prototype(Handle::clone(prototype));
            func_ref.detect_internal_properties(this);
        }

        let mut global = unsafe { self.global.borrow_mut_unbounded() };
        global.detect_internal_properties(self);
        global.set_property("globalThis".into(), Handle::clone(&self.global));

        patch_value(self, &self.statics.error_proto);
        patch_value(self, &self.statics.function_proto);
        patch_value(self, &self.statics.promise_proto);
        patch_value(self, &self.statics.boolean_proto);
        patch_value(self, &self.statics.number_proto);

        {
            let mut o = unsafe { self.statics.generator_iterator_proto.borrow_mut_unbounded() };
            o.detect_internal_properties(self);
            o.set_property(
                Handle::clone(&self.statics.symbol_iterator).into(),
                Handle::clone(&self.statics.identity)
            );
            o.set_property("next".into(), Handle::clone(&self.statics.generator_iterator_next));
            o.set_property("return".into(), Handle::clone(&self.statics.generator_iterator_return));
        }

        {
            let mut o = unsafe { self.statics.symbol_ctor.borrow_mut_unbounded() };
            o.detect_internal_properties(self);
            o.set_property("for".into(), Handle::clone(&self.statics.symbol_for));
            o.set_property("keyFor".into(), Handle::clone(&self.statics.symbol_key_for));
            o.set_property("iterator".into(), Handle::clone(&self.statics.symbol_iterator));
            o.set_property("asyncIterator".into(), Handle::clone(&self.statics.symbol_async_iterator));
            o.set_property("hasInstance".into(), Handle::clone(&self.statics.symbol_has_instance));
            o.set_property("isConcatSpreadable".into(), Handle::clone(&self.statics.symbol_is_concat_spreadable));
            o.set_property("match".into(), Handle::clone(&self.statics.symbol_match));
            o.set_property("matchAll".into(), Handle::clone(&self.statics.symbol_match_all));
            o.set_property("replace".into(), Handle::clone(&self.statics.symbol_replace));
            o.set_property("search".into(), Handle::clone(&self.statics.symbol_search));
            o.set_property("species".into(), Handle::clone(&self.statics.symbol_species));
            o.set_property("split".into(), Handle::clone(&self.statics.symbol_split));
            o.set_property("toPrimitive".into(), Handle::clone(&self.statics.symbol_to_primitive));
            o.set_property("toStringTag".into(), Handle::clone(&self.statics.symbol_to_string_tag));
            o.set_property("unscopables".into(), Handle::clone(&self.statics.symbol_unscopables));
            global.set_property("Symbol".into(), Handle::clone(&self.statics.symbol_ctor));
        }
        
        {
            let mut o = unsafe { self.statics.string_proto.borrow_mut_unbounded() };
            o.detect_internal_properties(self);
            o.set_property("charAt".into(), Handle::clone(&self.statics.string_char_at));
            o.set_property("charCodeAt".into(), Handle::clone(&self.statics.string_char_code_at));
            o.set_property("endsWith".into(), Handle::clone(&self.statics.string_ends_with));
            o.set_property("anchor".into(), Handle::clone(&self.statics.string_anchor));
            o.set_property("big".into(), Handle::clone(&self.statics.string_big));
            o.set_property("blink".into(), Handle::clone(&self.statics.string_blink));
            o.set_property("bold".into(), Handle::clone(&self.statics.string_bold));
            o.set_property("fixed".into(), Handle::clone(&self.statics.string_fixed));
            o.set_property("fontcolor".into(), Handle::clone(&self.statics.string_fontcolor));
            o.set_property("fontsize".into(), Handle::clone(&self.statics.string_fontsize));
            o.set_property("italics".into(), Handle::clone(&self.statics.string_italics));
            o.set_property("link".into(), Handle::clone(&self.statics.string_link));
            o.set_property("small".into(), Handle::clone(&self.statics.string_small));
            o.set_property("strike".into(), Handle::clone(&self.statics.string_strike));
            o.set_property("sub".into(), Handle::clone(&self.statics.string_sub));
            o.set_property("sup".into(), Handle::clone(&self.statics.string_sup));
            o.set_property("includes".into(), Handle::clone(&self.statics.string_includes));
            o.set_property("indexOf".into(), Handle::clone(&self.statics.string_index_of));
            o.set_property("padStart".into(), Handle::clone(&self.statics.string_pad_start));
            o.set_property("padEnd".into(), Handle::clone(&self.statics.string_pad_end));
            o.set_property("repeat".into(), Handle::clone(&self.statics.string_repeat));
            o.set_property("toLowerCase".into(), Handle::clone(&self.statics.string_to_lowercase));
            o.set_property("toUpperCase".into(), Handle::clone(&self.statics.string_to_uppercase));
            o.set_property("replace".into(), Handle::clone(&self.statics.string_replace));
        }

        {
            let mut o = unsafe { self.statics.array_proto.borrow_mut_unbounded() };
            o.detect_internal_properties(self);
            o.set_property("push".into(), Handle::clone(&self.statics.array_push));
            o.set_property("concat".into(), Handle::clone(&self.statics.array_concat));
            o.set_property("map".into(), Handle::clone(&self.statics.array_map));
            o.set_property("every".into(), Handle::clone(&self.statics.array_every));
            o.set_property("fill".into(), Handle::clone(&self.statics.array_fill));
            o.set_property("filter".into(), Handle::clone(&self.statics.array_filter));
            o.set_property("find".into(), Handle::clone(&self.statics.array_find));
            o.set_property("findIndex".into(), Handle::clone(&self.statics.array_find_index));
            o.set_property("flat".into(), Handle::clone(&self.statics.array_flat));
            o.set_property("forEach".into(), Handle::clone(&self.statics.array_for_each));
            o.set_property("from".into(), Handle::clone(&self.statics.array_from));
            o.set_property("includes".into(), Handle::clone(&self.statics.array_includes));
            o.set_property("indexOf".into(), Handle::clone(&self.statics.array_index_of));
            o.set_property("join".into(), Handle::clone(&self.statics.array_join));
            o.set_property("lastIndexOf".into(), Handle::clone(&self.statics.array_last_index_of));
            o.set_property("of".into(), Handle::clone(&self.statics.array_of));
            o.set_property("pop".into(), Handle::clone(&self.statics.array_pop));
            o.set_property("reduce".into(), Handle::clone(&self.statics.array_reduce));
            o.set_property("reduceRight".into(), Handle::clone(&self.statics.array_reduce_right));
            o.set_property("reverse".into(), Handle::clone(&self.statics.array_reverse));
            o.set_property("shift".into(), Handle::clone(&self.statics.array_shift));
            o.set_property("slice".into(), Handle::clone(&self.statics.array_slice));
            o.set_property("some".into(), Handle::clone(&self.statics.array_some));
            o.set_property("sort".into(), Handle::clone(&self.statics.array_sort));
            o.set_property("splice".into(), Handle::clone(&self.statics.array_splice));
            o.set_property("unshift".into(), Handle::clone(&self.statics.array_unshift));
        }

        {
            let mut array_ctor = unsafe { self.statics.array_ctor.borrow_mut_unbounded() };
            array_ctor.set_property("isArray".into(), Handle::clone(&self.statics.array_is_array));
        }

        {
            let mut promise_ctor = unsafe { self.statics.promise_ctor.borrow_mut_unbounded() };
            promise_ctor.set_property("resolve".into(), Handle::clone(&self.statics.promise_resolve));
            promise_ctor.set_property("reject".into(), Handle::clone(&self.statics.promise_reject));
        }

        {
            let mut o = unsafe { self.statics.weakset_proto.borrow_mut_unbounded() };
            o.detect_internal_properties(self);
            o.set_property("has".into(), Handle::clone(&self.statics.weakset_has));
            o.set_property("add".into(), Handle::clone(&self.statics.weakset_add));
            o.set_property("delete".into(), Handle::clone(&self.statics.weakset_delete));
        }

        {
            let mut o = unsafe { self.statics.weakmap_proto.borrow_mut_unbounded() };
            o.detect_internal_properties(self);
            o.set_property("has".into(), Handle::clone(&self.statics.weakmap_has));
            o.set_property("add".into(), Handle::clone(&self.statics.weakmap_add));
            o.set_property("get".into(), Handle::clone(&self.statics.weakmap_get));
            o.set_property("delete".into(), Handle::clone(&self.statics.weakmap_delete));
        }

        {
            let mut object_proto = unsafe { self.statics.object_proto.borrow_mut_unbounded() };
            object_proto.constructor = Some(Handle::clone(&self.statics.object_ctor));
            object_proto.proto = Some(Value::new(ValueKind::Null).into_handle(self));
            object_proto.set_property("toString".into(), Handle::clone(&self.statics.object_to_string));
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
        patch_constructor(self, &self.statics.symbol_ctor, &self.statics.symbol_proto);
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
        patch_value(self, &self.statics.string_includes);
        patch_value(self, &self.statics.string_index_of);
        patch_value(self, &self.statics.string_pad_start);
        patch_value(self, &self.statics.string_pad_end);
        patch_value(self, &self.statics.string_repeat);
        patch_value(self, &self.statics.string_to_lowercase);
        patch_value(self, &self.statics.string_to_uppercase);
        patch_value(self, &self.statics.string_replace);
        patch_value(self, &self.statics.promise_resolve);
        patch_value(self, &self.statics.promise_reject);
        patch_value(self, &self.statics.generator_iterator_next);
        patch_value(self, &self.statics.generator_iterator_return);
        patch_value(self, &self.statics.symbol_iterator);
        patch_value(self, &self.statics.symbol_async_iterator);
        patch_value(self, &self.statics.symbol_has_instance);
        patch_value(self, &self.statics.symbol_is_concat_spreadable);
        patch_value(self, &self.statics.symbol_match);
        patch_value(self, &self.statics.symbol_match_all);
        patch_value(self, &self.statics.symbol_replace);
        patch_value(self, &self.statics.symbol_search);
        patch_value(self, &self.statics.symbol_species);
        patch_value(self, &self.statics.symbol_split);
        patch_value(self, &self.statics.symbol_to_primitive);
        patch_value(self, &self.statics.symbol_to_string_tag);
        patch_value(self, &self.statics.symbol_unscopables);
        patch_value(self, &self.statics.symbol_for);
        patch_value(self, &self.statics.symbol_key_for);
        

        global.set_property("NaN".into(), self.create_js_value(f64::NAN).into_handle(self));
        global.set_property("Infinity".into(), self.create_js_value(f64::INFINITY).into_handle(self));
        global.set_property("isNaN".into(), self.statics.isnan.clone());

        {
            let mut object_ctor = unsafe { self.statics.object_ctor.borrow_mut_unbounded() };
            object_ctor.set_property("defineProperty".into(), self.statics.object_define_property.clone());
            object_ctor.set_property("getOwnPropertyNames".into(), self.statics.object_get_own_property_names.clone());
            object_ctor.set_property("getPrototypeOf".into(), self.statics.object_get_prototype_of.clone());
            global.set_property("Object".into(), Handle::clone(&self.statics.object_ctor));
        }

        {
            let mut math_obj = self.create_object();
            math_obj.set_property("pow".into(), Handle::clone(&self.statics.math_pow));
            math_obj.set_property("abs".into(), Handle::clone(&self.statics.math_abs));
            math_obj.set_property("ceil".into(), Handle::clone(&self.statics.math_ceil));
            math_obj.set_property("floor".into(), Handle::clone(&self.statics.math_floor));
            math_obj.set_property("max".into(), Handle::clone(&self.statics.math_max));
            math_obj.set_property("random".into(), Handle::clone(&self.statics.math_random));

            math_obj.set_property("PI".into(), self.create_js_value(std::f64::consts::PI).into_handle(self));
            math_obj.set_property("E".into(), self.create_js_value(std::f64::consts::E).into_handle(self));
            math_obj.set_property("LN10".into(), self.create_js_value(std::f64::consts::LN_10).into_handle(self));
            math_obj.set_property("LN2".into(), self.create_js_value(std::f64::consts::LN_2).into_handle(self));
            math_obj.set_property("LOG10E".into(), self.create_js_value(std::f64::consts::LOG10_E).into_handle(self));
            math_obj.set_property("LOG2E".into(), self.create_js_value(std::f64::consts::LOG2_E).into_handle(self));
            math_obj.set_property("SQRT2".into(),self.create_js_value(std::f64::consts::SQRT_2).into_handle(self));
            global.set_property("Math".into(), math_obj.into_handle(self));
        }

        {
            let mut json_obj = self.create_object();
            json_obj.set_property("parse".into(), Handle::clone(&self.statics.json_parse));
            json_obj.set_property("stringify".into(), Handle::clone(&self.statics.json_stringify));
            global.set_property("JSON".into(), json_obj.into_handle(self));
        }

        {
            let mut console_obj = self.create_object();
            console_obj.set_property("log".into(), Handle::clone(&self.statics.console_log));
            global.set_property("console".into(), console_obj.into_handle(self));
        }

        global.set_property("Error".into(), self.statics.error_ctor.clone());
        global.set_property("Boolean".into(), self.statics.boolean_ctor.clone());
        global.set_property("Number".into(), self.statics.number_ctor.clone());
        global.set_property("String".into(), self.statics.string_ctor.clone());
        global.set_property("Function".into(), self.statics.function_ctor.clone());
        global.set_property("Array".into(), self.statics.array_ctor.clone());
        global.set_property("WeakSet".into(), self.statics.weakset_ctor.clone());
        global.set_property("WeakMap".into(), self.statics.weakmap_ctor.clone());
        global.set_property("Promise".into(), self.statics.promise_ctor.clone());
    }

    fn unwind(&mut self, value: Handle<Value>, fp: usize) -> Result<(), Handle<Value>> {
        // TODO: clean up resources caused by this unwind
        if self.unwind_handlers.is_empty() {
            return Err(value);
        }

        if let Some(handler) = unsafe { self.unwind_handlers.get() } {
            if handler.frame_pointer < fp {
                return Err(value);
            }
        } else {
            // todo: this branch is unnecessary, see if statement above
            return Err(value);
        }

        // Try to get the last unwind handler
        let handler = self.unwind_handlers.pop();

        let this_frame_pointer = self.frames.len();
        // Go back the call stack back to where the last try/catch block lives
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
            let func = unsafe { frame.func.borrow_unbounded() };
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

    fn begin_function_call(
        &mut self,
        func_cell: Handle<Value>,
        mut params: Vec<Handle<Value>>,
    ) -> Result<(), Handle<Value>> {
        let func_cell_ref = unsafe { func_cell.borrow_unbounded() };

        let closure = match func_cell_ref.as_function() {
            Some(FunctionKind::Native(f)) => {
                let receiver = f.receiver.as_ref().map(|rx| rx.get().clone());
                let ctx = CallContext {
                    vm: self,
                    args: &mut params,
                    ctor: false,
                    receiver,
                };

                let result = (f.func)(ctx)?;
                self.stack.push(result);

                return Ok(());
            }
            Some(FunctionKind::Closure(u)) => u,
            None => return Err(js_std::error::create_error("Invoked value is not a function", self)),
            // There should never be raw user functions
            _ => unreachable!(),
        };

        let origin_param_count = closure.func.params as usize;
        let param_count = params.len();

        if param_count > origin_param_count {
            // remove extra params
            params.drain(param_count..);
        }

        for _ in 0..(origin_param_count.saturating_sub(param_count)) {
            // add dummy undefined values for remaining, missing arguments
            params.push(Value::new(ValueKind::Undefined).into_handle(self));
        }

        if closure.func.ty.is_generator() {
            let iterator = GeneratorIterator::new(Handle::clone(&func_cell), params);
            let value = self.create_js_value(iterator).into_handle(self);
            self.stack.push(value);
            return Ok(());
        }

        let current_sp = self.stack.len();

        let frame = Frame {
            buffer: closure.func.buffer.clone(),
            ip: 0,
            func: func_cell.clone(),
            sp: current_sp,
            iterator_caller: None,
            is_constructor: false
        };
        self.frames.push(frame);

        for param in params.into_iter() {
            self.stack.push(param);
        }

        Ok(())
    }

    /// Runs all async tasks in the queue
    ///
    /// This should only be called when the frame stack is empty
    pub fn run_async_tasks(&mut self) {
        debug_assert!(self.frames.is_empty());
        let async_frames = self.async_frames.take();
        self.frames = async_frames;
        // Uncaught errors and return values in async tasks are swallowed.
        let _ = self.interpret();
    }

    /// Queues a task/frame for execution when the call stack is empty
    pub fn queue_async_task(&mut self, frame: Frame) {
        self.async_frames.push(frame);
    }

    /// Executes an execution frame
    pub fn execute_frame(&mut self, frame: Frame, can_gc: bool) -> Result<DispatchResult, VMError> {
        let frame_idx = self.frames.len();
        self.frames.push(frame);

        macro_rules! unwind_abort_if_uncaught {
            ($e:expr) => {
                if let Err(e) = self.unwind($e, frame_idx) {
                    self.frames.reset();
                    self.stack.reset();
                    self.loops.reset();
                    return Err(VMError::UncaughtError(e));
                } else {
                    continue;
                }
            };
        }

        while self.frames.len() > frame_idx {
            unsafe {
                if can_gc && unlikely(self.should_gc()) {
                    self.perform_gc();
                }
            }

            let opcode = self.buffer()[self.ip()].as_op();

            self.frame_mut().ip += 1;

            match dispatch::handle(self, opcode, frame_idx) {
                Ok(Some(result)) => return Ok(result),
                Ok(None) => {}
                Err(e) => unwind_abort_if_uncaught!(e),
            };
       }

       // is it *really* ok to even get to this point? undecided.
       unreachable!()
    }

    /// Starts interpreting bytecode
    pub fn interpret(&mut self) -> Result<Option<Handle<Value>>, VMError> {
        let frame = if !self.frames.is_empty() {
            self.frames.pop()
        } else {
            return Ok(None);
        };

        self.execute_frame(frame, true).map(DispatchResult::into_value)
    }

    /// Evaluates a JavaScript source string in this VM
    pub fn eval<'a>(&mut self, source: &'a str) -> Result<Option<Handle<Value>>, EvalError<'a>> {
        let (buffer, constants, mut gc) = Compiler::<()>::from_str(source, None, CompilerFunctionKind::Function)
            .map_err(FromStrError::from)
            .map_err(EvalError::from)?
            .compile()
            .map_err(EvalError::CompileError)?;

        self.mark_constants(&mut gc);

        self.constants_gc.transfer(gc);

        let frame = Frame::from_buffer(false, to_vm_instructions(buffer), constants, self);

        self.execute_frame(frame, true)
            .map(DispatchResult::into_value)
            .map_err(EvalError::VMError)
    }

    /// Sets internal properties ([[prototype]] and constructor) of every value in the provided GC
    /// and marks constants, for example assigns a 
    fn mark_constants(&self, gc: &mut Gc<Value>) {
        for guard in gc.heap.iter() {
            let mut value = guard.borrow_mut();
            value.detect_internal_properties(self);
        }
    }
}

impl Drop for VM {
    fn drop(&mut self) {
        self.stack.reset();
        self.frames.reset();
        self.async_frames.reset();
    }
}