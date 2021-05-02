use core::fmt::Debug;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub enum Value {
    Ident(String),
    Number(f64),
    Bool(bool),
    Object(Box<dyn Object>),
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

    fn into_ident(self) -> String {
        match self {
            Self::Ident(i) => i,
            _ => unreachable!(),
        }
    }
}

pub trait JsValue {
    fn print(&self);

    fn as_number(&self) -> Option<f64>;
    fn into_ident(self) -> String;
    fn is_truthy(&self) -> bool;
}

pub trait Object: JsValue + Debug {
    fn as_string(&self) -> Option<&str>;
    fn is_string(&self) -> bool {
        self.as_string().is_some()
    }
}

#[derive(Debug, Clone)]
pub struct JsString(String);

impl JsString {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

impl Object for JsString {
    fn as_string(&self) -> Option<&str> {
        Some(&self.0)
    }
}

impl JsValue for JsString {
    fn print(&self) {
        println!("{}", self.0);
    }

    fn as_number(&self) -> Option<f64> {
        None
    }

    fn is_truthy(&self) -> bool {
        self.0.len() > 0
    }

    fn into_ident(self) -> String {
        unreachable!()
    }
}
