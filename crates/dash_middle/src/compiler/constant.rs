use std::ops::Deref;
use std::rc::Rc;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::parser::expr::LiteralExpr;
use crate::parser::statement::FunctionKind;
use crate::parser::statement::VariableBinding;
use crate::parser::statement::VariableDeclarationName;

use super::external::External;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: Option<String>,
    pub buffer: Box<[u8]>,
    pub ty: FunctionKind,
    pub locals: usize,
    pub params: usize,
    pub constants: Box<[Constant]>,
    pub externals: Box<[External]>,
    pub r#async: bool,
    /// If the parameter list uses the rest operator ..., then this will be Some(local_id)
    pub rest_local: Option<u16>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Number(f64),
    String(Rc<str>),
    Identifier(Rc<str>),
    Boolean(bool),
    Function(Rc<Function>),
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

    pub fn from_literal<'a>(expr: &LiteralExpr<'a>) -> Self {
        match expr {
            LiteralExpr::Number(n) => Self::Number(*n),
            LiteralExpr::Identifier(s) => Self::Identifier(s.as_ref().into()),
            LiteralExpr::Binding(VariableBinding {
                name: VariableDeclarationName::Identifier(name),
                ..
            }) => Self::Identifier((*name).into()),
            LiteralExpr::Binding(..) => panic!("Cannot convert destructuring binding to constant"),
            LiteralExpr::String(s) => Self::String(s.as_ref().into()),
            LiteralExpr::Boolean(b) => Self::Boolean(*b),
            LiteralExpr::Null => Self::Null,
            LiteralExpr::Undefined => Self::Undefined,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
