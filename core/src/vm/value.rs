use core::fmt::Debug;
use std::{
    borrow::Cow,
    cell::RefCell,
    collections::HashMap,
    fmt::{self, Formatter},
    rc::Rc,
};

use crate::{js_std, util};

use super::{
    instruction::{Constant, Instruction},
    upvalue::Upvalue,
    VM,
};

pub struct CallContext<'a> {
    pub vm: &'a mut VM,
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
    Constant(Box<Constant>),
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
        let k = k.into();

        if value.fields.len() > 0 {
            // We only need to go the "slow" path and look up the given key in a HashMap if there are entries
            if let Some(entry) = value.fields.get(k) {
                return Some(entry.clone());
            }
        }

        match &value.kind {
            ValueKind::Object(o) => o.get_property(value_cell, k),
            _ => Some(Rc::new(RefCell::new(Value::new(ValueKind::Undefined)))),
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
            ValueKind::Constant(_) => true,
            _ => false,
        }
    }

    pub fn as_constant(&self) -> Option<&Constant> {
        match &self.kind {
            ValueKind::Constant(c) => Some(&**c),
            _ => None,
        }
    }

    pub fn as_number(&self) -> f64 {
        match &self.kind {
            ValueKind::Number(n) => *n,
            ValueKind::Bool(b) => *b as u8 as f64,
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

    pub fn as_object_mut(&mut self) -> Option<&mut Object> {
        match &mut self.kind {
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
            ValueKind::Constant(_) => unreachable!(),
            ValueKind::Null => Cow::Borrowed("null"),
            ValueKind::Number(n) => Cow::Owned(n.to_string()),
            ValueKind::Object(o) => o.to_string(),
            ValueKind::Undefined => Cow::Borrowed("undefined"),
        }
    }

    pub fn _typeof(&self) -> &'static str {
        match &self.kind {
            ValueKind::Bool(_) => "boolean",
            ValueKind::Null => "object",
            ValueKind::Object(o) => o._typeof(),
            ValueKind::Number(_) => "number",
            ValueKind::Undefined => "undefined",
            _ => unreachable!(),
        }
    }

    pub fn compare(&self, other: &Value) -> Option<Compare> {
        match &self.kind {
            ValueKind::Number(n) => {
                let rhs = other.as_number();
                if *n > rhs {
                    Some(Compare::Greater)
                } else {
                    Some(Compare::Less)
                }
            }
            ValueKind::Bool(b) => {
                let rhs = other.as_number();
                let lhs = *b as u8 as f64;

                if lhs > rhs {
                    Some(Compare::Greater)
                } else {
                    Some(Compare::Less)
                }
            }
            _ => None,
        }
    }

    pub fn lossy_equal(&self, other: &Value) -> bool {
        self.strict_equal(other) // TODO: handle it separately
    }

    pub fn strict_equal(&self, other: &Value) -> bool {
        match &self.kind {
            ValueKind::Number(n) => {
                let other = match &other.kind {
                    ValueKind::Number(n) => n,
                    _ => return false,
                };

                return *other == *n;
            }
            ValueKind::Bool(b) => {
                let other = match &other.kind {
                    ValueKind::Bool(b) => b,
                    _ => return false,
                };

                return *other == *b;
            }
            ValueKind::Null => matches!(other.kind, ValueKind::Null),
            ValueKind::Undefined => matches!(other.kind, ValueKind::Undefined),
            ValueKind::Object(o) => o.strict_equal(other),
            _ => false,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        self.as_object().and_then(|o| o.as_string())
    }

    pub fn as_string_lossy(&self) -> Option<Cow<str>> {
        match &self.kind {
            ValueKind::Number(n) => Some(Cow::Owned(n.to_string())),
            ValueKind::Bool(b) => Some(Cow::Owned(b.to_string())),
            ValueKind::Null => Some(Cow::Borrowed("null")),
            ValueKind::Undefined => Some(Cow::Borrowed("undefined")),
            ValueKind::Constant(_) => unreachable!(),
            ValueKind::Object(o) => o.as_string_lossy(),
        }
    }

    pub fn into_ident(self) -> Option<String> {
        match self.kind {
            ValueKind::Constant(i) => i.into_ident(),
            _ => None,
        }
    }

    pub fn into_object(self) -> Option<Object> {
        match self.kind {
            ValueKind::Object(o) => Some(*o),
            _ => None,
        }
    }

    pub fn into_string(self) -> Option<String> {
        self.into_object().and_then(|c| c.into_string())
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

    pub fn sub(&self, other: &Value) -> Value {
        match &self.kind {
            ValueKind::Number(n) => {
                let other = other.as_number();
                Value::new(ValueKind::Number(*n - other))
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
    Array(Array),
    Any(AnyObject),
}

#[derive(Debug, Clone)]
pub struct Array {
    pub elements: Vec<Rc<RefCell<Value>>>,
}

impl Array {
    pub fn new(elements: Vec<Rc<RefCell<Value>>>) -> Self {
        Self { elements }
    }
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
                _ => Some(Rc::new(RefCell::new(Value::new(ValueKind::Undefined)))),
            },
            Self::Array(a) => match &k[..] {
                "length" => Some(Value::new(ValueKind::Number(a.elements.len() as f64)).into()),
                "push" => Some(Rc::new(RefCell::new(Value::new(ValueKind::Object(
                    Box::new(Object::Function(FunctionKind::Native(NativeFunction {
                        name: "push",
                        func: js_std::array::push,
                        receiver: Some(Receiver::Bound(cell.clone())),
                    }))),
                ))))),
                _ => {
                    if util::is_numeric(k) {
                        // Unwrapping is ok, we've just made sure it's numeric
                        // We might want to remove the util::is_numeric check and use .is_some()
                        // Or even if let
                        let num = k.parse::<usize>().unwrap();
                        if num < a.elements.len() {
                            return Some(a.elements[num].clone());
                        }
                    }
                    Some(Rc::new(RefCell::new(Value::new(ValueKind::Undefined))))
                }
            },
            _ => Some(Rc::new(RefCell::new(Value::new(ValueKind::Undefined)))),
        }
    }

    fn _typeof(&self) -> &'static str {
        match self {
            Self::Any(_) | Self::Array(_) => "object",
            Self::Function(_) => "function",
            Self::String(_) => "string",
        }
    }

    fn is_truthy(&self) -> bool {
        match self {
            Self::String(s) => !s.is_empty(),
            Self::Array(_) => true,
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

    fn as_string_lossy(&self) -> Option<Cow<str>> {
        match self {
            Self::String(s) => Some(Cow::Borrowed(s)),
            Self::Function(s) => Some(Cow::Owned(s.to_string())),
            Self::Array(_) => Some(Cow::Borrowed("[array, idk i havent done this yet]")),
            Self::Any(_) => Some(Cow::Borrowed("[object Object]")),
        }
    }

    fn into_string(self) -> Option<String> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    fn to_string(&self) -> Cow<str> {
        match self {
            Self::String(s) => Cow::Borrowed(s),
            Self::Function(f) => Cow::Owned(f.to_string()),
            Self::Array(a) => {
                let mut s = String::from("[");
                for (index, element_cell) in a.elements.iter().enumerate() {
                    let element = element_cell.borrow();
                    if index > 0 {
                        s.push(',');
                    }
                    s.push_str(&*element.to_string());
                }
                s.push(']');
                Cow::Owned(s)
            }
            _ => Cow::Borrowed("[object Object]"), // TODO: look if there's a toString function
        }
    }

    fn as_function(&self) -> Option<&FunctionKind> {
        match self {
            Self::Function(kind) => Some(kind),
            _ => None,
        }
    }

    fn lossy_equal(&self, other: &Value) -> bool {
        self.strict_equal(other)
    }

    fn strict_equal(&self, other: &Value) -> bool {
        match self {
            Self::String(s) => {
                let other = match &other.kind {
                    ValueKind::Object(o) => match &**o {
                        Object::String(s) => s,
                        _ => return false,
                    },
                    _ => return false,
                };

                s.eq(other)
            }
            _ => {
                let other = match &other.kind {
                    ValueKind::Object(o) => &**o,
                    _ => return false,
                };

                std::ptr::eq(self as *const _, other as *const _)
            }
        }
    }
}

pub enum Compare {
    Less,
    Greater,
    Equal,
}
