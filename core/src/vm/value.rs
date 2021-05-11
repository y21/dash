use core::fmt::Debug;
use std::{
    borrow::Cow,
    cell::RefCell,
    collections::HashMap,
    fmt::{self, Formatter},
    rc::Rc,
};

use crate::js_std;

use super::{instruction::Instruction, VM};

pub struct CallContext<'a> {
    pub vm: &'a VM,
    pub args: Vec<Rc<RefCell<Value>>>,
    pub receiver: Option<Rc<RefCell<Value>>>,
}

#[derive(Debug, Clone)]
pub struct Value {
    pub kind: ValueKind,
    pub fields: HashMap<Box<str>, Rc<RefCell<Value>>>,
    pub constructor: Option<Rc<RefCell<Value>>>,
}

impl Value {
    pub fn new(kind: ValueKind) -> Self {
        Self {
            kind,
            fields: HashMap::new(),
            constructor: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ValueKind {
    Ident(String),
    Number(f64),
    Bool(bool),
    Object(Box<Object>),
    Undefined,
    Null,
}

impl Value {
    pub fn try_into_inner(value: Rc<RefCell<Self>>) -> Option<Self> {
        Some(Rc::try_unwrap(value).ok()?.into_inner())
    }

    pub fn get_property(value_cell: &Rc<RefCell<Value>>, k: &str) -> Option<Rc<RefCell<Value>>> {
        let value = value_cell.borrow();

        if value.fields.len() > 0 {
            // We only need to go the "slow" path and look up the given key in a HashMap if there are entries
            if let Some(entry) = value.fields.get(k) {
                return Some(entry.clone());
            }
        }

        match &value.kind {
            ValueKind::Object(o) => o.get_property(value_cell, k),
            _ => unreachable!(),
        }
    }

    pub fn set_property(&mut self, k: impl Into<Box<str>>, v: Rc<RefCell<Value>>) {
        self.fields.insert(k.into(), v);
    }

    pub fn is_truthy(&self) -> bool {
        match &self.kind {
            ValueKind::Bool(b) => *b,
            ValueKind::Number(n) => *n != 0f64,
            ValueKind::Object(o) => o.is_truthy(),
            ValueKind::Undefined | ValueKind::Null => false,
            _ => unreachable!(),
        }
    }

    pub fn is_assignment_target(&self) -> bool {
        match &self.kind {
            ValueKind::Ident(_) => true,
            _ => false,
        }
    }

    pub fn as_number(&self) -> f64 {
        match &self.kind {
            ValueKind::Number(n) => *n,
            ValueKind::Object(o) => o.as_number(),
            _ => f64::NAN,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match &self.kind {
            ValueKind::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&Object> {
        match &self.kind {
            ValueKind::Object(o) => Some(o),
            _ => None,
        }
    }

    pub fn as_function(&self) -> Option<&FunctionKind> {
        match &self.kind {
            ValueKind::Object(o) => o.as_function(),
            _ => None,
        }
    }

    pub fn to_string(&self) -> Cow<str> {
        match &self.kind {
            ValueKind::Bool(b) => Cow::Owned(b.to_string()),
            ValueKind::Ident(s) => Cow::Borrowed(&s),
            ValueKind::Null => Cow::Borrowed("null"),
            ValueKind::Number(n) => Cow::Owned(n.to_string()),
            ValueKind::Object(o) => o.to_string(),
            ValueKind::Undefined => Cow::Borrowed("undefined"),
        }
    }

    pub fn compare(&self, other: &Value) -> Option<Compare> {
        match &self.kind {
            ValueKind::Number(n) => {
                let rhs = other.as_number();
                if *n > rhs {
                    Some(Compare::Less)
                } else {
                    Some(Compare::Greater)
                }
            }
            ValueKind::Bool(b) => {
                let rhs = other.as_number();
                let lhs = *b as u8 as f64;

                if lhs > rhs {
                    Some(Compare::Less)
                } else {
                    Some(Compare::Greater)
                }
            }
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        self.as_object().and_then(|o| o.as_string())
    }

    pub fn into_ident(self) -> Option<String> {
        match self.kind {
            ValueKind::Ident(i) => Some(i),
            _ => None,
        }
    }

    pub fn into_object(self) -> Option<Object> {
        todo!()
    }

    pub fn into_string(self) -> Option<String> {
        todo!()
    }

    pub fn add_assign(&mut self, other: &Value) {
        match &mut self.kind {
            ValueKind::Number(n) => {
                let o = other.as_number();
                *n += o;
            }
            _ => todo!(),
        }
    }

    pub fn sub_assign(&mut self, other: &Value) {
        match &mut self.kind {
            ValueKind::Number(n) => {
                let o = other.as_number();
                *n -= o;
            }
            _ => todo!(),
        }
    }
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
pub struct UserFunction {
    pub params: u32,
    pub receiver: Option<Receiver>,
    pub ty: FunctionType,
    pub buffer: Box<[Instruction]>,
    pub name: Option<String>,
}

impl UserFunction {
    pub fn new(buffer: Vec<Instruction>, params: u32, ty: FunctionType) -> Self {
        Self {
            buffer: buffer.into_boxed_slice(),
            params,
            name: None,
            ty,
            receiver: None,
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
    pub func: for<'a> fn(CallContext<'a>) -> Rc<RefCell<Value>>,
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
pub enum Object {
    String(String),
    Function(FunctionKind),
    Any(AnyObject),
}

#[derive(Debug, Clone)]
pub struct AnyObject {}

#[derive(Debug, Clone)]
pub enum FunctionType {
    Top,
    Function,
    Closure,
}

#[derive(Debug, Clone)]
pub enum FunctionKind {
    User(UserFunction),
    Native(NativeFunction),
}

impl ToString for FunctionKind {
    fn to_string(&self) -> String {
        match self {
            Self::Native(n) => format!("function {}() {{ [native code] }}", n.name),
            Self::User(u) => format!("function {}() {{ ... }}", u.name.as_deref().unwrap_or("")),
        }
    }
}

impl Object {
    fn get_property(&self, cell: &Rc<RefCell<Value>>, k: &str) -> Option<Rc<RefCell<Value>>> {
        match self {
            Self::String(_) => match &k[..] {
                "indexOf" => Some(Rc::new(RefCell::new(Value::new(ValueKind::Object(
                    Box::new(Object::Function(FunctionKind::Native(NativeFunction {
                        name: "indexOf",
                        func: js_std::string::index_of,
                        receiver: Some(Receiver::Bound(cell.clone())),
                    }))),
                ))))),
                _ => todo!(),
            },
            _ => todo!(),
        }
    }

    fn is_truthy(&self) -> bool {
        match self {
            Self::String(s) => s.len() != 0,
            Self::Function(..) => true,
            Self::Any(_) => true,
        }
    }

    fn as_number(&self) -> f64 {
        f64::NAN // TODO: try to convert it to number?
    }

    fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    fn to_string(&self) -> Cow<str> {
        match self {
            Self::String(s) => Cow::Borrowed(s),
            Self::Function(f) => Cow::Owned(f.to_string()),
            _ => Cow::Borrowed("[object Object]"), // TODO: look if there's a toString function
        }
    }

    fn as_function(&self) -> Option<&FunctionKind> {
        match self {
            Self::Function(kind) => Some(kind),
            _ => None,
        }
    }
}

pub enum Compare {
    Less,
    Greater,
    Equal,
}
