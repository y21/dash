use crate::vm::{instruction::Instruction, upvalue::Upvalue, VM};
use core::fmt::{self, Debug, Formatter};
use std::cell::RefCell;
use std::rc::Rc;

use super::Value;

pub type NativeFunctionCallback = for<'a> fn(CallContext<'a>) -> Rc<RefCell<Value>>;

pub struct CallContext<'a> {
    pub vm: &'a mut VM,
    pub args: Vec<Rc<RefCell<Value>>>,
    pub receiver: Option<Rc<RefCell<Value>>>,
}

#[derive(Debug, Clone)]
pub enum FunctionType {
    Top,
    Function,
    Closure,
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
    pub params: u32,
    pub receiver: Option<Receiver>,
    pub ty: FunctionType,
    pub buffer: Box<[Instruction]>,
    pub name: Option<String>,
    pub upvalues: u32,
}

impl UserFunction {
    pub fn new(buffer: Vec<Instruction>, params: u32, ty: FunctionType, upvalues: u32) -> Self {
        Self {
            buffer: buffer.into_boxed_slice(),
            params,
            name: None,
            ty,
            receiver: None,
            upvalues,
        }
    }

    pub fn bind(&mut self, recv: Receiver) {
        self.receiver = Some(recv);
    }

    pub fn rebind(mut self, recv: Receiver) -> Self {
        self.receiver = Some(recv);
        self
    }
}

pub struct NativeFunction {
    pub name: &'static str,
    pub func: NativeFunctionCallback,
    pub receiver: Option<Receiver>,
}

impl NativeFunction {
    pub fn new(
        name: &'static str,
        func: for<'a> fn(CallContext<'a>) -> Rc<RefCell<Value>>,
        receiver: Option<Receiver>,
    ) -> Self {
        Self {
            name,
            func,
            receiver,
        }
    }
}

impl Clone for NativeFunction {
    fn clone(&self) -> Self {
        Self {
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
pub enum FunctionKind {
    Closure(Closure),
    User(UserFunction),
    Native(NativeFunction),
}

impl ToString for FunctionKind {
    fn to_string(&self) -> String {
        match self {
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
