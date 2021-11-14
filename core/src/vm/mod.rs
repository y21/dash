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

use crate::{EvalError, agent::Agent, compiler::{compiler::{self, CompileError, Compiler, FunctionKind as CompilerFunctionKind}, instruction::to_vm_instructions}, gc::{Gc, Handle}, parser::{lexer, token}, util::{unlikely, MaybeOwned}, vm::{dispatch::DispatchResult, frame::UnwindHandler, value::{ValueKind, array::Array, function::{CallContext, FunctionKind, UserFunction}, generator::GeneratorIterator}}};
use crate::js_std;

use self::{frame::{Frame, Loop}, instruction::Constant, stack::Stack, statics::Statics, value::{PropertyKey, object::{ExoticObject, Object, ObjectKind}}};

// Force garbage collection at 10000 objects by default
const DEFAULT_GC_OBJECT_COUNT_THRESHOLD: usize = 10000;

/// An error that may occur during bytecode execution
#[derive(Debug)]
pub enum VMError {
    /// An error was thrown and user code did not catch it
    UncaughtError(Value),
}

/// An owned error that may occur during bytecode execution
///
/// The contained value is cloned.
// TODO: this is unused, remove
#[derive(Debug)]
pub enum OwnedVMError {
    /// An error was thrown and user code did not catch it
    UncaughtError(Value),
}

impl From<VMError> for OwnedVMError {
    fn from(e: VMError) -> Self {
        match e {
            VMError::UncaughtError(e) => Self::UncaughtError(e.clone())
        }
    }
}

impl VMError {
    /// Returns the inner value of the error
    pub fn into_value(self) -> Value {
        match self {
            Self::UncaughtError(err) => err
        }
    }

    /// Formats this error by taking the `stack` property of the error object
    pub fn to_string(&self, vm: &VM) -> Cow<'static, str> {
        match self {
            Self::UncaughtError(err) => {
                let stack = err.get_field(vm, PropertyKey::from("stack"))
                    .or_else(|| err.get_field(vm, PropertyKey::from("message")))
                    .unwrap_or_else(|| err.clone());

                stack.to_string(vm)
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
    pub(crate) gc: RefCell<Gc<Object>>,
    /// Garbage collector specifically for constant objects produced by the compiler
    pub(crate) constants_gc: Gc<Object>,
    /// Call stack
    pub(crate) frames: Stack<Frame, 256>,
    /// Async task queue. Processed when execution has finished
    pub(crate) async_frames: Stack<Frame, 256>,
    /// Stack
    pub(crate) stack: Stack<Value, 512>,
    /// Global namespace
    pub(crate) global: Value,
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
    pub(crate) symbols: HashMap<Box<str>, Value>,
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
        let global = Value::new(ValueKind::Object(gc.register(Object::new(ObjectKind::Ordinary))));

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
    pub fn global(&self) -> &Value {
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
        Object::mark(&self.statics.generator_iterator_proto);
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
        self.stack.pop().as_number(self)
    }

    /// Reads an index
    fn read_index(&mut self) -> Option<usize> {
        self.read_constant()
            .and_then(Constant::into_index)
    }

    fn read_lhs_rhs(&mut self) -> (Value, Value) {
        let rhs = self.stack.pop();
        let lhs = self.stack.pop();
        (lhs, rhs)
    }

    /// Creates a JavaScript object
    pub fn create_ordinary_object(&self) -> Handle<Object> {
        let mut o = Object::new(ObjectKind::Ordinary);
        o.constructor = Some(Handle::clone(&self.statics.object_ctor));
        o.prototype = Some(Handle::clone(&self.statics.object_proto));
        self.register_object(o)
    }

    /// Creates a JavaScript object
    pub fn create_null_object(&self) -> Handle<Object> {
        let mut o = Object::new(ObjectKind::Ordinary);
        self.register_object(Object::new(ObjectKind::Ordinary))
    }

    /// Creates a JavaScript object with provided fields
    pub fn create_object_with_fields(
        &self,
        fields: impl Into<HashMap<PropertyKey<'static>, Value>>,
    ) -> Handle<Object> {
        let mut o = Object::new(ObjectKind::Ordinary);
        o.fields = fields.into();
        self.register_object(o)
    }

    /// Registers an unboxed JavaScript array for garbage collection
    pub fn register_array(&self, array: Array) -> Handle<Object> {
        // todo: detect_internal_properties
        self.register_object(Object::new(ObjectKind::Exotic(ExoticObject::Array(array))))
    }

    /// Registers an unboxed JavaScript object for garbage collection
    pub fn register_object(&self, object: Object) -> Handle<Object> {
        self.gc.borrow_mut().register(object)
    }

    #[rustfmt::skip]
    fn prepare_stdlib(&self) {
        // All values that live in self.statics do not have a [[Prototype]] set
        // so we do it here
        fn patch_value(this: &VM, value: &Handle<Object>) {
            value.borrow_mut(this).detect_internal_properties(this);
        }
        
        fn patch_constructor(this: &VM, func: &Handle<Object>, prototype: &Handle<Object>) {
            let func_ref = func.borrow_mut(this);
            let real_func = func_ref.as_function_mut().unwrap();
            real_func.set_prototype(Handle::clone(prototype));
            func_ref.detect_internal_properties(this);
        }

        let mut global = &self.global;
        global.detect_internal_properties(self);
        global.set_property(self, "globalThis", global.clone());

        patch_value(self, &self.statics.error_proto);
        patch_value(self, &self.statics.function_proto);
        patch_value(self, &self.statics.promise_proto);
        patch_value(self, &self.statics.boolean_proto);
        patch_value(self, &self.statics.number_proto);

        {
            let mut o = self.statics.generator_iterator_proto.borrow_mut(self);
            o.detect_internal_properties(self);
            o.set_property(
                Handle::clone(&self.statics.symbol_iterator),
                Handle::clone(&self.statics.identity)
            );
            o.set_property("next", Handle::clone(&self.statics.generator_iterator_next));
            o.set_property("return", Handle::clone(&self.statics.generator_iterator_return));
        }

        {
            let mut o = self.statics.symbol_ctor.borrow_mut(self);
            o.detect_internal_properties(self);
            o.set_property("for", Handle::clone(&self.statics.symbol_for));
            o.set_property("keyFor", Handle::clone(&self.statics.symbol_key_for));
            o.set_property("iterator", Handle::clone(&self.statics.symbol_iterator));
            o.set_property("asyncIterator", Handle::clone(&self.statics.symbol_async_iterator));
            o.set_property("hasInstance", Handle::clone(&self.statics.symbol_has_instance));
            o.set_property("isConcatSpreadable", Handle::clone(&self.statics.symbol_is_concat_spreadable));
            o.set_property("match", Handle::clone(&self.statics.symbol_match));
            o.set_property("matchAll", Handle::clone(&self.statics.symbol_match_all));
            o.set_property("replace", Handle::clone(&self.statics.symbol_replace));
            o.set_property("search", Handle::clone(&self.statics.symbol_search));
            o.set_property("species", Handle::clone(&self.statics.symbol_species));
            o.set_property("split", Handle::clone(&self.statics.symbol_split));
            o.set_property("toPrimitive", Handle::clone(&self.statics.symbol_to_primitive));
            o.set_property("toStringTag", Handle::clone(&self.statics.symbol_to_string_tag));
            o.set_property("unscopables", Handle::clone(&self.statics.symbol_unscopables));
            global.set_property(self, "Symbol", Handle::clone(&self.statics.symbol_ctor));
        }
        
        {
            let mut o = self.statics.string_proto.borrow_mut(self);
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
            o.set_property("includes", Handle::clone(&self.statics.string_includes));
            o.set_property("indexOf", Handle::clone(&self.statics.string_index_of));
            o.set_property("padStart", Handle::clone(&self.statics.string_pad_start));
            o.set_property("padEnd", Handle::clone(&self.statics.string_pad_end));
            o.set_property("repeat", Handle::clone(&self.statics.string_repeat));
            o.set_property("toLowerCase", Handle::clone(&self.statics.string_to_lowercase));
            o.set_property("toUpperCase", Handle::clone(&self.statics.string_to_uppercase));
            o.set_property("replace", Handle::clone(&self.statics.string_replace));
        }
        
        {
            let mut o = self.statics.array_proto.borrow_mut(self);
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
            let mut array_ctor = self.statics.array_ctor.borrow_mut(self);
            array_ctor.set_property("isArray", Handle::clone(&self.statics.array_is_array));
        }
        
        {
            let mut promise_ctor = self.statics.promise_ctor.borrow_mut(self);
            promise_ctor.set_property("resolve", Handle::clone(&self.statics.promise_resolve));
            promise_ctor.set_property("reject", Handle::clone(&self.statics.promise_reject));
        }
        
        {
            let mut o = self.statics.weakset_proto.borrow_mut(self);
            o.detect_internal_properties(self);
            o.set_property("has", Handle::clone(&self.statics.weakset_has));
            o.set_property("add", Handle::clone(&self.statics.weakset_add));
            o.set_property("delete", Handle::clone(&self.statics.weakset_delete));
        }
        
        {
            let mut o = self.statics.weakmap_proto.borrow_mut(self);
            o.detect_internal_properties(self);
            o.set_property("has", Handle::clone(&self.statics.weakmap_has));
            o.set_property("add", Handle::clone(&self.statics.weakmap_add));
            o.set_property("get", Handle::clone(&self.statics.weakmap_get));
            o.set_property("delete", Handle::clone(&self.statics.weakmap_delete));
        }

        {
            let mut o = self.statics.object_proto.borrow_mut(self);
            o.constructor = Some(Handle::clone(&self.statics.object_ctor));
            o.prototype = None;
            o.set_property("toString", Handle::clone(&self.statics.object_to_string));
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
        patch_value(self, &self.statics.object_get_own_property_symbols);
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
        

        global.set_property(self, "NaN", f64::NAN);
        global.set_property(self, "Infinity", f64::INFINITY);
        global.set_property(self, "isNaN", Handle::clone(&self.statics.isnan));

        {
            let mut o = self.statics.object_ctor.borrow_mut(self);
            o.set_property("defineProperty", Handle::clone(&self.statics.object_define_property));
            o.set_property("getOwnPropertyNames", Handle::clone(&self.statics.object_get_own_property_names));
            o.set_property("getOwnPropertySymbols", Handle::clone(&self.statics.object_get_own_property_symbols));
            o.set_property("getPrototypeOf", Handle::clone(&self.statics.object_get_prototype_of));
            global.set_property(self, "Object", Handle::clone(&self.statics.object_ctor));
        }

        {
            let mut math_obj = Object::new(ObjectKind::Ordinary);
            math_obj.set_property("pow", Handle::clone(&self.statics.math_pow));
            math_obj.set_property("abs", Handle::clone(&self.statics.math_abs));
            math_obj.set_property("ceil", Handle::clone(&self.statics.math_ceil));
            math_obj.set_property("floor", Handle::clone(&self.statics.math_floor));
            math_obj.set_property("max", Handle::clone(&self.statics.math_max));
            math_obj.set_property("random", Handle::clone(&self.statics.math_random));
        
            math_obj.set_property("PI", std::f64::consts::PI);
            math_obj.set_property("E", std::f64::consts::E);
            math_obj.set_property("LN10", std::f64::consts::LN_10);
            math_obj.set_property("LN2", std::f64::consts::LN_2);
            math_obj.set_property("LOG10E", std::f64::consts::LOG10_E);
            math_obj.set_property("LOG2E", std::f64::consts::LOG2_E);
            math_obj.set_property("SQRT2",std::f64::consts::SQRT_2);
            global.set_property(self, "Math", self.register_object(math_obj));
        }

        {
            let mut json_obj = Object::new(ObjectKind::Ordinary);
            json_obj.set_property("parse", Handle::clone(&self.statics.json_parse));
            json_obj.set_property("stringify", Handle::clone(&self.statics.json_stringify));
            global.set_property(self, "JSON", self.register_object(json_obj));
        }

        {
            let mut console_obj = Object::new(ObjectKind::Ordinary);
            console_obj.set_property("log", Handle::clone(&self.statics.console_log));
            global.set_property(self, "console", self.register_object(console_obj));
        }

        global.set_property(self, "Error", self.statics.error_ctor.clone());
        global.set_property(self, "Boolean", self.statics.boolean_ctor.clone());
        global.set_property(self, "Number", self.statics.number_ctor.clone());
        global.set_property(self, "String", self.statics.string_ctor.clone());
        global.set_property(self, "Function", self.statics.function_ctor.clone());
        global.set_property(self, "Array", self.statics.array_ctor.clone());
        global.set_property(self, "WeakSet", self.statics.weakset_ctor.clone());
        global.set_property(self, "WeakMap", self.statics.weakmap_ctor.clone());
        global.set_property(self, "Promise", self.statics.promise_ctor.clone());
    }

    fn unwind(&mut self, value: Value, fp: usize) -> Result<(), Value> {
        // TODO: clean up resources caused by this unwind
        if self.unwind_handlers.is_empty() {
            return Err(value);
        }

        if let Some(handler) = self.unwind_handlers.get() {
            if handler.frame_pointer <= fp {
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
        for frame in self.frames.as_array_bottom().take(10) {
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

    /// Attempts to push a value onto the stack
    pub(crate) fn try_push_stack(&mut self, value: Value) -> Result<(), Value> {
        let ok = self.stack.try_push(value);

        if !ok {
            Err(js_std::error::create_error("Maximum stack size exceeded", self).into())
        } else {
            Ok(())
        }
    }

    /// Attempts to push a value onto the stack
    pub(crate) fn try_push_frame(&mut self, frame: Frame) -> Result<(), Value> {
        let ok = self.frames.try_push(frame);

        if !ok {
            Err(js_std::error::create_error("Maximum call stack size exceeded", self).into())
        } else {
            Ok(())
        }
    }

    fn begin_function_call(
        &mut self,
        func_cell: Value,
        mut params: Vec<Value>,
    ) -> Result<(), Value> {
        let object = func_cell.as_object();
        let object_cell = func_cell.as_object().map(|x| x.borrow(self));

        let closure = match object_cell.and_then(|x| x.as_function_mut()) {
            Some(FunctionKind::Native(f)) => {
                let receiver = f.receiver.as_ref().map(|rx| rx.get().clone());
                let ctx = CallContext {
                    vm: self,
                    args: &mut params,
                    ctor: false,
                    receiver,
                };

                let result = (f.func)(ctx)?;
                self.try_push_stack(result)?;

                return Ok(());
            }
            Some(FunctionKind::Closure(u)) => u,
            None => return Err(js_std::error::create_error("Invoked value is not a function", self).into()),
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
            params.push(Value::new(ValueKind::Undefined));
        }

        let object = func_cell.as_object().unwrap();

        if closure.func.ty.is_generator() {
            // it's ok to unwrap as_object()
            // at this point we know it's an object (specifically a function)
            let iterator = GeneratorIterator::new(Handle::clone(object), params);
            let value = Value::from(self.register_object(iterator.into()));
            self.try_push_stack(value)?;
            return Ok(());
        }

        let current_sp = self.stack.len();

        let frame = Frame {
            buffer: closure.func.buffer.clone(),
            ip: 0,
            func: Handle::clone(object),
            sp: current_sp,
            iterator_caller: None,
            is_constructor: false
        };
        self.try_push_frame(frame)?;

        for param in params.into_iter() {
            self.try_push_stack(param)?;
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
        self.try_push_frame(frame).map_err(VMError::UncaughtError)?;

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
                Err(err) => {
                    if let Err(e) = self.unwind(err, frame_idx) {
                        return Err(VMError::UncaughtError(e));
                    } else {
                        continue;
                    }
                }
            };
       }

       // is it *really* ok to even get to this point? undecided.
       unreachable!()
    }

    /// Starts interpreting bytecode
    pub fn interpret(&mut self) -> Result<Option<Value>, VMError> {
        let frame = if !self.frames.is_empty() {
            self.frames.pop()
        } else {
            return Ok(None);
        };

        self.execute_frame(frame, true).map(DispatchResult::into_value)
    }

    /// Evaluates a JavaScript source string in this VM
    pub fn eval<'a>(&mut self, source: &'a str) -> Result<Option<Value>, EvalError<'a>> {
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
    fn mark_constants(&self, gc: &mut Gc<Object>) {
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