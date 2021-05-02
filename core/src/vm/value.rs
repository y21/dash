use core::fmt::Debug;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub enum Value {
    Ident(String),
    Number(f64),
    Bool(bool),
    Object(Box<Object>),
}

impl Value {
    pub fn try_into_inner(value: Rc<RefCell<Self>>) -> Option<Self> {
        Some(Rc::try_unwrap(value).ok()?.into_inner())
    }
}

impl JsValue for Value {
    fn print(&self) {
        match self {
            Self::Number(n) => {
                dbg!(n);
            }
            Self::Bool(b) => {
                dbg!(b);
            }
            Self::Object(o) => o.print(),
            Self::Ident(o) => {
                dbg!(o);
            }
        }
    }

    fn is_truthy(&self) -> bool {
        match self {
            Self::Bool(b) => *b,
            Self::Number(n) => *n != 0f64,
            Self::Object(o) => o.is_truthy(),
            _ => unreachable!(),
        }
    }

    fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            _ => None,
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
}

#[derive(Debug)]
pub enum Object {
    String(String),
}

impl JsValue for Object {
    fn print(&self) {
        match self {
            Self::String(s) => dbg!(s),
        };
    }

    fn is_truthy(&self) -> bool {
        match self {
            Self::String(s) => s.len() != 0,
        }
    }

    fn as_number(&self) -> Option<f64> {
        unreachable!()
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
}

pub trait JsValue {
    fn print(&self);

    fn as_number(&self) -> Option<f64>;
    fn as_bool(&self) -> Option<bool>;
    fn as_object(&self) -> Option<&Object>;
    fn as_string(&self) -> Option<&str>;

    fn into_object(self) -> Option<Object>;
    fn into_string(self) -> Option<String>;
    fn into_ident(self) -> Option<String>;
    fn is_truthy(&self) -> bool;
}
