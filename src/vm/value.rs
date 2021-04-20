use core::fmt::Debug;

#[derive(Debug)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Object(Box<dyn Object>)
}

impl JsValue for Value {
    fn print(&self) {
        match self {
            Self::Number(n) => { dbg!(n); },
            Self::Bool(b) => { dbg!(b); },
            Self::Object(o) => o.print()
        }
    }

    fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            _ => None
        }
    }
}

pub trait JsValue {
    fn print(&self);

    fn as_number(&self) -> Option<f64>;
}

pub trait Object: JsValue + Debug {
    fn as_string(&self) -> Option<&str>;
    fn is_string(&self) -> bool {
        self.as_string().is_some()
    }
}

#[derive(Debug)]
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
}