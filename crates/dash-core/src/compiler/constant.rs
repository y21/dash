use std::ops::Deref;
use std::rc::Rc;

use crate::parser::expr::LiteralExpr;
use crate::parser::statement::FunctionKind;

use super::External;

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: Option<String>,
    pub buffer: Rc<[u8]>,
    pub ty: FunctionKind,
    pub locals: usize,
    pub params: usize,
    pub constants: Rc<[Constant]>,
    pub externals: Box<[External]>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Number(f64),
    String(Rc<str>),
    Identifier(Rc<str>),
    Boolean(bool),
    Function(Function),
    Null,
    Undefined,
}

impl Constant {
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Constant::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&Rc<str>> {
        match self {
            Constant::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_identifier(&self) -> Option<&Rc<str>> {
        match self {
            Constant::Identifier(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Constant::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

impl<'a> From<&LiteralExpr<'a>> for Constant {
    fn from(expr: &LiteralExpr<'a>) -> Self {
        match expr {
            LiteralExpr::Number(n) => Constant::Number(*n),
            LiteralExpr::Identifier(s) => Constant::Identifier(s.as_ref().into()),
            LiteralExpr::String(s) => Constant::String(s.as_ref().into()),
            LiteralExpr::Boolean(b) => Constant::Boolean(*b),
            LiteralExpr::Null => Constant::Null,
            LiteralExpr::Undefined => Constant::Undefined,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConstantPool {
    constants: Vec<Constant>,
}

pub struct LimitExceededError;
impl ConstantPool {
    pub fn new() -> Self {
        Self { constants: Vec::new() }
    }

    pub fn add(&mut self, constant: Constant) -> Result<u16, LimitExceededError> {
        if self.constants.len() > u16::MAX as usize {
            Err(LimitExceededError)
        } else {
            let id = self.constants.len() as u16;
            self.constants.push(constant);
            Ok(id)
        }
    }

    pub fn into_vec(self) -> Vec<Constant> {
        self.constants
    }
}

impl Deref for ConstantPool {
    type Target = [Constant];

    fn deref(&self) -> &Self::Target {
        &self.constants
    }
}
