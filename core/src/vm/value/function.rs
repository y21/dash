use crate::vm::{instruction::Instruction, upvalue::Upvalue, VM};
use core::fmt::{self, Debug, Formatter};
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::rc::Weak;

use super::object::AnyObject;
use super::Value;

pub type NativeFunctionCallback =
    for<'a> fn(CallContext<'a>) -> Result<CallResult, Rc<RefCell<Value>>>;

#[derive(Debug, Clone, Copy)]
pub enum Constructor {
    // Function can be invoked with or without the new keyword
    Any,
    // Function can be invoked as a constructor using `new`, but also works without
    Ctor,
    // Function is not a constructor and cannot be called with `new`
    NoCtor,
}

impl Constructor {
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
///
pub enum CallResult {
    /// A user function needs to be called to proceed
    UserFunction(Rc<RefCell<Value>>, Vec<Rc<RefCell<Value>>>),
    /// We have a value
    Ready(Rc<RefCell<Value>>),
}

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

    pub fn get(&self) -> Option<&T> {
        self.0.as_ref()
    }

    pub fn get_or_insert(&mut self, value: T) -> &mut T {
        self.0.get_or_insert(value)
    }

    pub fn get_or_insert_with<F>(&mut self, func: F) -> &mut T
    where
        F: FnMut() -> T,
    {
        self.0.get_or_insert_with(func)
    }
}

impl CallState<Box<dyn Any>> {
    pub fn get_or_insert_as<F, V>(&mut self, mut func: F) -> Option<&mut V>
    where
        V: 'static,
        F: FnMut() -> V,
    {
        self.get_or_insert_with(|| Box::new(func()))
            .downcast_mut::<V>()
    }
}

pub struct CallContext<'a> {
    pub vm: &'a mut VM,
    pub args: &'a mut Vec<Rc<RefCell<Value>>>,
    pub receiver: Option<Rc<RefCell<Value>>>,
    pub ctor: bool,
    pub state: &'a mut CallState<Box<dyn Any>>,
    pub function_call_response: Option<Rc<RefCell<Value>>>,
}

impl<'a> CallContext<'a> {
    pub fn arguments(&self) -> impl Iterator<Item = &Rc<RefCell<Value>>> {
        // TODO: fix order
        self.args.iter().rev()
    }

    pub fn state<V: 'static>(&self) -> Option<&V> {
        self.state.get().and_then(|x| x.downcast_ref::<V>())
    }
}

#[derive(Debug, Clone)]
pub enum FunctionType {
    Top,
    Function,
    Closure,
    Module,
}

#[derive(Debug, Clone)]
pub enum Receiver {
    Pinned(Rc<RefCell<Value>>),
    Bound(Rc<RefCell<Value>>),
}

impl Receiver {
    pub fn get(&self) -> &Rc<RefCell<Value>> {
        match self {
            Self::Pinned(p) => p,
            Self::Bound(b) => b,
        }
    }

    // TODO: this should be a no op if self is pinned
    pub fn bind(&mut self, recv: Receiver) {
        *self = recv;
    }

    pub fn rebind(self, recv: Receiver) -> Self {
        recv
    }
}

#[derive(Debug, Clone)]
pub struct Closure {
    pub func: UserFunction,
    pub upvalues: Vec<Upvalue>,
}
impl Closure {
    pub fn new(func: UserFunction) -> Self {
        Self {
            func,
            upvalues: Vec::new(),
        }
    }

    pub fn with_upvalues(func: UserFunction, upvalues: Vec<Upvalue>) -> Self {
        Self { func, upvalues }
    }
}

#[derive(Debug, Clone)]
pub struct UserFunction {
    pub ctor: Constructor,
    pub prototype: Option<Weak<RefCell<Value>>>,
    pub params: u32,
    pub receiver: Option<Receiver>,
    pub ty: FunctionType,
    pub buffer: Box<[Instruction]>,
    pub name: Option<String>,
    pub upvalues: u32,
}

impl UserFunction {
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

    pub fn bind(&mut self, new_recv: Receiver) {
        if let Some(recv) = &mut self.receiver {
            recv.bind(new_recv);
        }
    }

    pub fn rebind(mut self, new_recv: Receiver) -> Self {
        if let Some(recv) = &mut self.receiver {
            recv.bind(new_recv);
        }
        self
    }
}

pub struct NativeFunction {
    pub ctor: Constructor,
    pub name: &'static str,
    pub func: NativeFunctionCallback,
    pub receiver: Option<Receiver>,
    pub prototype: Option<Weak<RefCell<Value>>>,
}

impl NativeFunction {
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
        f.debug_struct("NativeFunction").finish()
    }
}

#[derive(Debug, Clone)]
pub struct Module {
    pub buffer: Option<Box<[Instruction]>>,
    pub exports: Exports,
}

impl Module {
    pub fn new(buffer: impl Into<Box<[Instruction]>>) -> Self {
        Self {
            buffer: Some(buffer.into()),
            exports: Exports::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Exports {
    pub default: Option<Rc<RefCell<Value>>>,
    pub named: HashMap<Box<str>, Rc<RefCell<Value>>>,
}

#[derive(Debug, Clone)]
pub enum FunctionKind {
    Closure(Closure),
    User(UserFunction),
    Native(NativeFunction),
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
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Closure(c) => c.func.name.as_deref(),
            Self::User(u) => u.name.as_deref(),
            Self::Native(n) => Some(n.name),
            _ => None,
        }
    }

    pub fn prototype_weak(&self) -> Option<&Weak<RefCell<Value>>> {
        match self {
            Self::Closure(c) => c.func.prototype.as_ref(),
            Self::User(u) => u.prototype.as_ref(),
            Self::Native(n) => n.prototype.as_ref(),
            _ => None,
        }
    }

    pub fn prototype(&self) -> Option<Rc<RefCell<Value>>> {
        self.prototype_weak().and_then(Weak::upgrade)
    }

    pub fn construct(&self, this: &Rc<RefCell<Value>>) -> Value {
        let mut o = Value::from(AnyObject {});
        o.proto = self.prototype_weak().cloned();
        o.constructor = Some(Rc::downgrade(this));
        o
    }

    pub fn set_prototype(&mut self, proto: Weak<RefCell<Value>>) {
        match self {
            Self::Closure(c) => c.func.prototype = Some(proto),
            Self::User(u) => u.prototype = Some(proto),
            Self::Native(n) => n.prototype = Some(proto),
            _ => {}
        };
    }

    pub fn as_closure(&self) -> Option<&Closure> {
        match self {
            Self::Closure(c) => Some(c),
            _ => None,
        }
    }

    pub fn into_closure(self) -> Option<Closure> {
        match self {
            Self::Closure(c) => Some(c),
            _ => None,
        }
    }

    pub fn as_user(&self) -> Option<&UserFunction> {
        match self {
            Self::User(u) => Some(u),
            _ => None,
        }
    }

    pub fn into_user(self) -> Option<UserFunction> {
        match self {
            Self::User(u) => Some(u),
            _ => None,
        }
    }

    pub fn as_native(&self) -> Option<&NativeFunction> {
        match self {
            Self::Native(n) => Some(n),
            _ => None,
        }
    }

    pub fn into_native(self) -> Option<NativeFunction> {
        match self {
            Self::Native(n) => Some(n),
            _ => None,
        }
    }

    pub fn as_module(&self) -> Option<&Module> {
        match self {
            Self::Module(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_module_mut(&mut self) -> Option<&mut Module> {
        match self {
            Self::Module(m) => Some(m),
            _ => None,
        }
    }

    pub fn into_module(self) -> Option<Module> {
        match self {
            Self::Module(m) => Some(m),
            _ => None,
        }
    }
}
