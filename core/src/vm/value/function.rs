use crate::gc::Handle;
use crate::vm::{instruction::Instruction, upvalue::Upvalue, VM};
use core::fmt::{self, Debug, Formatter};
use std::any::Any;
use std::collections::HashMap;

use super::object::AnyObject;
use super::Value;

/// A native function that can be called from JavaScript code
pub type NativeFunctionCallback = for<'a> fn(CallContext<'a>) -> Result<CallResult, Handle<Value>>;

/// Represents whether a function can be invoked as a constructor
#[derive(Debug, Clone, Copy)]
pub enum Constructor {
    /// Function can be invoked with or without the new keyword
    Any,
    /// Function can be invoked as a constructor using `new`, but also works without
    Ctor,
    /// Function is not a constructor and cannot be called with `new`
    NoCtor,
}

impl Constructor {
    /// Returns whether the function is constructable
    pub fn constructable(&self) -> bool {
        matches!(self, Constructor::Ctor | Constructor::Any)
    }
}

/// The result of calling a native function
///
/// It is common for a native function to call into a user function
/// I.e. due to conversion that invokes a user function
/// In that case, the function needs to be temporarily suspended
/// and return [CallResult::UserFunction] to notify the caller that it cannot proceed
pub enum CallResult {
    /// A user function needs to be called to proceed
    UserFunction(Handle<Value>, Vec<Handle<Value>>),
    /// We have a value
    Ready(Handle<Value>),
}

/// State that may be used in a native function
///
/// It is common for a native function to use `CallState` to keep track
/// of work it has done. Sometimes, it needs to call a user function and will
/// be called again at a later time when the called function returns.
/// To know where it left off, [CallState] may be used
pub struct CallState<T>(Option<T>);

impl<T> Debug for CallState<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("CallState")
    }
}

impl<T> Default for CallState<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T> CallState<T> {
    /// Applies a function on the inner call state, if present and returns
    /// the closure return value
    pub fn with<F, V>(&mut self, mut func: F) -> Option<V>
    where
        F: FnMut(&mut T) -> V,
    {
        if let Some(state) = &mut self.0 {
            Some(func(state))
        } else {
            None
        }
    }

    /// Returns a reference to the inner optional call state
    pub fn get(&self) -> Option<&T> {
        self.0.as_ref()
    }

    /// Returns a mutable reference to the call state.
    ///
    /// If None, the value passed to this function is stored.
    pub fn get_or_insert(&mut self, value: T) -> &mut T {
        self.0.get_or_insert(value)
    }

    /// Returns a mutable reference to the call state.
    ///
    /// If None, the passed function is called and its return value
    /// will be used as call state.
    pub fn get_or_insert_with<F>(&mut self, func: F) -> &mut T
    where
        F: FnMut() -> T,
    {
        self.0.get_or_insert_with(func)
    }
}

impl CallState<Box<dyn Any>> {
    /// Casts the inner call state to V and returns a mutable reference to it
    pub fn get_or_insert_as<F, V>(&mut self, mut func: F) -> Option<&mut V>
    where
        V: 'static,
        F: FnMut() -> V,
    {
        self.get_or_insert_with(|| Box::new(func()))
            .downcast_mut::<V>()
    }
}

/// Native function call context
pub struct CallContext<'a> {
    /// A mutable reference to the underlying VM
    pub vm: &'a mut VM,
    /// Arguments that were passed to this function
    ///
    /// Note that the order of arguments is last to first,
    /// i.e. the first argument is the last item of the vec
    /// due to the nature of a stack
    pub args: &'a mut Vec<Handle<Value>>,
    /// The receiver (`this`) value
    pub receiver: Option<Handle<Value>>,
    /// Whether this function call is invoked as a constructor call
    pub ctor: bool,
    /// State for this native call
    ///
    /// See docs for [CallState]
    pub state: &'a mut CallState<Box<dyn Any>>,
    /// The return value of a user function call that was made due to
    /// returning [CallResult::UserFunction]
    ///
    /// See docs for [CallResult] for when this would be set
    pub function_call_response: Option<Handle<Value>>,
}

impl<'a> CallContext<'a> {
    /// An iterator over arguments in fixed order
    pub fn arguments(&self) -> impl Iterator<Item = &Handle<Value>> {
        // TODO: fix order
        self.args.iter().rev()
    }

    /// Returns state associated to this call by downcasting it to V
    pub fn state<V: 'static>(&self) -> Option<&V> {
        self.state.get().and_then(|x| x.downcast_ref::<V>())
    }
}

/// The type of a function at runtime
#[derive(Debug, Clone)]
pub enum FunctionType {
    /// Top frame
    ///
    /// This is typically the initial script
    Top,
    /// A normal function
    Function,
    /// A closure
    Closure,
    /// A JavaScript module
    Module,
}

/// The receiver (`this`) of a function
#[derive(Debug, Clone)]
pub enum Receiver {
    /// Receiver is pinned and may not be changed
    Pinned(Handle<Value>),
    /// Receiver is bound to a specific value
    Bound(Handle<Value>),
}

impl Receiver {
    /// Returns the inner `this` value
    pub fn get(&self) -> &Handle<Value> {
        match self {
            Self::Pinned(p) => p,
            Self::Bound(b) => b,
        }
    }

    /// Rebinds this
    // TODO: this should be a no op if self is pinned
    pub fn bind(&mut self, recv: Receiver) {
        *self = recv;
    }

    /// Rebinds this by consuming the Receiver and returning it
    pub fn rebind(self, recv: Receiver) -> Self {
        recv
    }
}

/// A closure, wrapping a user function with values from the upper scope
#[derive(Debug, Clone)]
pub struct Closure {
    /// The inner value
    pub func: UserFunction,
    /// Values from the upper scope
    pub upvalues: Vec<Upvalue>,
}

impl Closure {
    /// Creates a new closure
    pub fn new(func: UserFunction) -> Self {
        Self {
            func,
            upvalues: Vec::new(),
        }
    }

    /// Creates a new closure given a user function and a vector of upvalues
    pub fn with_upvalues(func: UserFunction, upvalues: Vec<Upvalue>) -> Self {
        Self { func, upvalues }
    }
}

/// A JavaScript function created in JavaScript code
#[derive(Debug, Clone)]
pub struct UserFunction {
    /// Whether this function is constructable
    pub ctor: Constructor,
    /// The prototype of this function
    pub prototype: Option<Handle<Value>>,
    /// Number of parameters this function takes
    pub params: u32,
    /// The receiver of this function
    pub receiver: Option<Receiver>,
    /// The type of function
    pub ty: FunctionType,
    /// Function bytecode
    pub buffer: Box<[Instruction]>,
    /// The name of this function
    pub name: Option<String>,
    /// Number of values
    pub upvalues: u32,
}

impl UserFunction {
    /// Creates a new user function
    pub fn new(
        buffer: impl Into<Box<[Instruction]>>,
        params: u32,
        ty: FunctionType,
        upvalues: u32,
        ctor: Constructor,
    ) -> Self {
        Self {
            buffer: buffer.into(),
            params,
            name: None,
            ty,
            receiver: None,
            ctor,
            upvalues,
            prototype: None,
        }
    }

    /// Call `bind` on the underlying [Receiver]
    pub fn bind(&mut self, new_recv: Receiver) {
        if let Some(recv) = &mut self.receiver {
            recv.bind(new_recv);
        }
    }

    /// Call `rebind` on the underlying [Receiver]
    pub fn rebind(mut self, new_recv: Receiver) -> Self {
        if let Some(recv) = &mut self.receiver {
            recv.bind(new_recv);
        }
        self
    }
}

/// A native function that can be called from JavaScript code
pub struct NativeFunction {
    /// Whether this function can be invoked as a constructor
    pub ctor: Constructor,
    /// The name of this function
    pub name: &'static str,
    /// A pointer to the function
    pub func: NativeFunctionCallback,
    /// The receiver of this function
    pub receiver: Option<Receiver>,
    /// The prototype of this function
    pub prototype: Option<Handle<Value>>,
}

impl NativeFunction {
    /// Creates a new native function
    pub fn new(
        name: &'static str,
        func: NativeFunctionCallback,
        receiver: Option<Receiver>,
        ctor: Constructor,
    ) -> Self {
        Self {
            ctor,
            name,
            func,
            receiver,
            prototype: None,
        }
    }
}

impl Clone for NativeFunction {
    fn clone(&self) -> Self {
        Self {
            prototype: self.prototype.clone(),
            ctor: self.ctor,
            func: self.func,
            name: self.name,
            receiver: self.receiver.clone(),
        }
    }
}

impl Debug for NativeFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeFunction")
            .field("name", &self.name)
            .finish()
    }
}

/// A JavaScript module
#[derive(Debug, Clone)]
pub struct Module {
    /// Module bytecode, if present
    pub buffer: Option<Box<[Instruction]>>,
    /// The exports namespace
    pub exports: Exports,
}

impl Module {
    /// Creates a new module
    pub fn new(buffer: impl Into<Box<[Instruction]>>) -> Self {
        Self {
            buffer: Some(buffer.into()),
            exports: Exports::default(),
        }
    }
}

/// JavaScript module exports
#[derive(Debug, Clone, Default)]
pub struct Exports {
    /// The default export, if set
    pub default: Option<Handle<Value>>,
    /// Named exports
    pub named: HashMap<Box<str>, Handle<Value>>,
}

/// The kind of this function
#[derive(Debug, Clone)]
pub enum FunctionKind {
    /// A closure
    Closure(Closure),
    /// A user function
    User(UserFunction),
    /// A native function
    Native(NativeFunction),
    /// A JavaScript module
    Module(Module),
}

impl ToString for FunctionKind {
    fn to_string(&self) -> String {
        match self {
            // Users cannot access modules directly
            Self::Module(_) => unreachable!(),
            Self::Native(n) => format!("function {}() {{ [native code] }}", n.name),
            Self::User(u) => format!("function {}() {{ ... }}", u.name.as_deref().unwrap_or("")),
            Self::Closure(c) => {
                format!(
                    "function {}() {{ ... }}",
                    c.func.name.as_deref().unwrap_or("")
                )
            }
        }
    }
}

impl FunctionKind {
    /// Returns the name of this function, if present
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Closure(c) => c.func.name.as_deref(),
            Self::User(u) => u.name.as_deref(),
            Self::Native(n) => Some(n.name),
            _ => None,
        }
    }

    /// Returns a [Handle] to the prototype of this function, if it has one
    pub fn prototype(&self) -> Option<&Handle<Value>> {
        match self {
            Self::Closure(c) => c.func.prototype.as_ref(),
            Self::User(u) => u.prototype.as_ref(),
            Self::Native(n) => n.prototype.as_ref(),
            _ => None,
        }
    }

    pub(crate) fn mark(&self) {
        match self {
            FunctionKind::Module(module) => {
                if let Some(handle) = &module.exports.default {
                    Value::mark(handle)
                }

                for (_, handle) in &module.exports.named {
                    Value::mark(handle)
                }
            }
            FunctionKind::Native(native) => {
                if let Some(handle) = &native.receiver {
                    Value::mark(handle.get())
                }

                if let Some(handle) = &native.prototype {
                    Value::mark(handle)
                }
            }
            FunctionKind::User(func) => {
                if let Some(handle) = &func.receiver {
                    Value::mark(handle.get())
                }

                if let Some(handle) = &func.prototype {
                    Value::mark(handle)
                }
            }
            FunctionKind::Closure(closure) => {
                if let Some(handle) = &closure.func.receiver {
                    Value::mark(handle.get())
                }

                if let Some(handle) = &closure.func.prototype {
                    Value::mark(handle)
                }

                for upvalue in &closure.upvalues {
                    upvalue.mark_visited();
                }
            }
        }
    }

    /// Attempts to create an object with its [[Prototype]] set to this
    /// functions prototype
    pub fn construct(&self, this: &Handle<Value>) -> Value {
        let mut o = Value::from(AnyObject {});
        o.proto = self.prototype().cloned();
        o.constructor = Some(Handle::clone(this));
        o
    }

    /// Sets the prototype of this function
    pub fn set_prototype(&mut self, proto: Handle<Value>) {
        match self {
            Self::Closure(c) => c.func.prototype = Some(proto),
            Self::User(u) => u.prototype = Some(proto),
            Self::Native(n) => n.prototype = Some(proto),
            _ => {}
        };
    }

    /// Returns self as a closure, if it is one
    pub fn as_closure(&self) -> Option<&Closure> {
        match self {
            Self::Closure(c) => Some(c),
            _ => None,
        }
    }

    /// Returns self as an owned closure, if it is one
    pub fn into_closure(self) -> Option<Closure> {
        match self {
            Self::Closure(c) => Some(c),
            _ => None,
        }
    }

    /// Returns self as a user function, if it is one
    pub fn as_user(&self) -> Option<&UserFunction> {
        match self {
            Self::User(u) => Some(u),
            _ => None,
        }
    }

    /// Returns self as an owned user function, if it is one
    pub fn into_user(self) -> Option<UserFunction> {
        match self {
            Self::User(u) => Some(u),
            _ => None,
        }
    }

    /// Returns self as a native function, if it is one
    pub fn as_native(&self) -> Option<&NativeFunction> {
        match self {
            Self::Native(n) => Some(n),
            _ => None,
        }
    }

    /// Returns self as an owned native function, if it is one
    pub fn into_native(self) -> Option<NativeFunction> {
        match self {
            Self::Native(n) => Some(n),
            _ => None,
        }
    }

    /// Returns self as a JavaScript module, if it is one
    pub fn as_module(&self) -> Option<&Module> {
        match self {
            Self::Module(m) => Some(m),
            _ => None,
        }
    }

    /// Returns self as a mutable reference to the underlying JavaScript module,
    /// if it is one
    pub fn as_module_mut(&mut self) -> Option<&mut Module> {
        match self {
            Self::Module(m) => Some(m),
            _ => None,
        }
    }

    /// Returns self as an owned JavaScript module, if it is one
    pub fn into_module(self) -> Option<Module> {
        match self {
            Self::Module(m) => Some(m),
            _ => None,
        }
    }
}
