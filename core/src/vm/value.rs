use core::fmt::Debug;
use std::{
    cell::RefCell,
    fmt::{self, Formatter},
    rc::Rc,
};

use super::instruction::Instruction;

#[derive(Debug, Clone)]
pub enum Value {
    Ident(String),
    Number(f64),
    Bool(bool),
    Object(Box<Object>),
    Undefined,
    Null,
}

impl From<Object> for Value {
    fn from(o: Object) -> Self {
        Self::Object(Box::new(o))
    }
}

impl From<UserFunction> for Value {
    fn from(f: UserFunction) -> Self {
        Self::Object(Box::new(Object::Function(FunctionKind::User(f))))
    }
}

impl Value {
    pub fn try_into_inner(value: Rc<RefCell<Self>>) -> Option<Self> {
        Some(Rc::try_unwrap(value).ok()?.into_inner())
    }
}

impl JsValue for Value {
    fn is_truthy(&self) -> bool {
        match self {
            Self::Bool(b) => *b,
            Self::Number(n) => *n != 0f64,
            Self::Object(o) => o.is_truthy(),
            Self::Undefined | Self::Null => false,
            _ => unreachable!(),
        }
    }

    fn is_assignment_target(&self) -> bool {
        match self {
            Self::Ident(_) => true,
            _ => false,
        }
    }

    fn as_number(&self) -> f64 {
        match self {
            Self::Number(n) => *n,
            Self::Object(o) => o.as_number(),
            _ => f64::NAN,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    fn as_object(&self) -> Option<&Object> {
        match self {
            Self::Object(o) => Some(o),
            _ => None,
        }
    }

    fn as_function(&self) -> Option<&FunctionKind> {
        match self {
            Self::Object(o) => o.as_function(),
            _ => None,
        }
    }

    fn as_string(&self) -> Option<&str> {
        self.as_object().and_then(|o| o.as_string())
    }

    fn into_ident(self) -> Option<String> {
        match self {
            Self::Ident(i) => Some(i),
            _ => None,
        }
    }

    fn into_object(self) -> Option<Object> {
        todo!()
    }

    fn into_string(self) -> Option<String> {
        todo!()
    }

    fn add_assign(&mut self, other: &Value) {
        match self {
            Self::Number(n) => {
                let o = other.as_number();
                *n += o;
            }
            _ => todo!(),
        }
    }

    fn sub_assign(&mut self, other: &Value) {
        match self {
            Self::Number(n) => {
                let o = other.as_number();
                *n -= o;
            }
            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserFunction {
    pub params: u32,
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
        }
    }
}

pub struct NativeFunction(pub fn(Vec<Rc<RefCell<Value>>>) -> Rc<RefCell<Value>>);

impl Clone for NativeFunction {
    fn clone(&self) -> Self {
        Self(self.0)
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
}

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

impl JsValue for Object {
    fn is_truthy(&self) -> bool {
        match self {
            Self::String(s) => s.len() != 0,
            Self::Function(..) => true,
        }
    }

    fn add_assign(&mut self, other: &Value) {
        todo!()
    }

    fn sub_assign(&mut self, other: &Value) {
        todo!()
    }

    fn is_assignment_target(&self) -> bool {
        false
    }

    fn as_number(&self) -> f64 {
        f64::NAN // TODO: try to convert it to number?
    }

    fn as_bool(&self) -> Option<bool> {
        unreachable!()
    }

    fn as_object(&self) -> Option<&Object> {
        Some(self)
    }

    fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    fn as_function(&self) -> Option<&FunctionKind> {
        match self {
            Self::Function(kind) => Some(kind),
            _ => None,
        }
    }

    fn into_object(self) -> Option<Object> {
        Some(self)
    }

    fn into_string(self) -> Option<String> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    fn into_ident(self) -> Option<String> {
        unreachable!()
    }

    fn logical_negate(&self) -> bool {
        !self.is_truthy()
    }
}

pub trait JsValue {
    fn as_number(&self) -> f64;
    fn as_bool(&self) -> Option<bool>;
    fn as_object(&self) -> Option<&Object>;
    fn as_string(&self) -> Option<&str>;
    fn as_function(&self) -> Option<&FunctionKind>;

    fn sub_assign(&mut self, other: &Value);
    fn add_assign(&mut self, other: &Value);
    fn unary_negate(&self) -> f64 {
        -self.as_number()
    }
    fn logical_negate(&self) -> bool {
        !self.is_truthy()
    }

    fn into_object(self) -> Option<Object>;
    fn into_string(self) -> Option<String>;
    fn into_ident(self) -> Option<String>;
    fn is_truthy(&self) -> bool;
    fn is_assignment_target(&self) -> bool;
}
