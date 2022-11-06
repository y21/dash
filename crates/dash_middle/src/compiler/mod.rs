use strum_macros::FromRepr;

use crate::lexer::token::TokenType;
use crate::parser;
use crate::parser::expr::{AssignmentExpr, Expr, GroupingExpr, LiteralExpr};

use self::external::External;
use self::scope::CompileValueType;
use self::{constant::ConstantPool, scope::Scope};

#[cfg(feature = "format")]
use serde::{Deserialize, Serialize};
pub mod constant;
pub mod external;
#[cfg(feature = "format")]
pub mod format;
pub mod instruction;
pub mod instruction_iter;
pub mod scope;

#[cfg_attr(feature = "format", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct CompileResult {
    pub instructions: Vec<u8>,
    pub cp: ConstantPool,
    pub locals: usize,
    pub externals: Vec<External>,
}

/// Function call metadata
///
/// Highest bit = set if constructor call
/// 2nd highest bit = set if object call
/// remaining 6 bits = number of arguments
#[repr(transparent)]
pub struct FunctionCallMetadata(u8);

impl From<u8> for FunctionCallMetadata {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<FunctionCallMetadata> for u8 {
    fn from(value: FunctionCallMetadata) -> Self {
        value.0
    }
}

impl FunctionCallMetadata {
    pub fn new_checked(mut value: u8, constructor: bool, object: bool) -> Option<Self> {
        if value & 0b11000000 == 0 {
            if constructor {
                value |= 0b10000000;
            }

            if object {
                value |= 0b01000000;
            }

            Some(Self(value))
        } else {
            None
        }
    }

    pub fn value(&self) -> u8 {
        self.0 & !0b11000000
    }

    pub fn is_constructor_call(&self) -> bool {
        self.0 & (1 << 7) != 0
    }

    pub fn is_object_call(&self) -> bool {
        self.0 & (1 << 6) != 0
    }
}

#[repr(u8)]
#[derive(FromRepr, Clone, Copy)]
pub enum StaticImportKind {
    All,
    Default,
}

#[repr(u8)]
#[derive(FromRepr)]
pub enum ObjectMemberKind {
    Getter,
    Setter,
    Static,
    Dynamic,
}

use parser::expr::ObjectMemberKind as ParserObjectMemberKind;

impl From<&ParserObjectMemberKind<'_>> for ObjectMemberKind {
    fn from(v: &ParserObjectMemberKind<'_>) -> Self {
        match v {
            ParserObjectMemberKind::Dynamic(..) => Self::Dynamic,
            ParserObjectMemberKind::Getter(..) => Self::Getter,
            ParserObjectMemberKind::Setter(..) => Self::Setter,
            ParserObjectMemberKind::Static(..) => Self::Static,
        }
    }
}

pub fn infer_type<'a>(cx: &mut Scope<'a>, expr: &Expr<'a>) -> Option<CompileValueType> {
    match expr {
        Expr::Literal(LiteralExpr::Boolean(..)) => Some(CompileValueType::Boolean),
        Expr::Literal(LiteralExpr::Null) => Some(CompileValueType::Null),
        Expr::Literal(LiteralExpr::Undefined) => Some(CompileValueType::Undefined),
        Expr::Literal(LiteralExpr::Number(..)) => Some(CompileValueType::Number),
        Expr::Literal(LiteralExpr::String(..)) => Some(CompileValueType::String),
        Expr::Literal(LiteralExpr::Identifier(ident)) => match cx.find_local(&ident) {
            Some((_, local)) => local.inferred_type().borrow().clone(),
            None => None,
        },
        Expr::Grouping(GroupingExpr(exprs)) => {
            let mut last = None;
            for expr in exprs.iter() {
                last = infer_type(cx, expr);
            }
            Some(last.expect("GroupingExpr should have at least one expression"))
        }
        Expr::Assignment(AssignmentExpr { left, right, .. }) => {
            let right_type = infer_type(cx, right);
            if let Expr::Literal(LiteralExpr::Identifier(ident)) = left.as_ref() {
                if let Some((_, local)) = cx.find_local(&ident) {
                    let left_type = local.inferred_type();
                    let left_type_ref = left_type.borrow();

                    if left_type_ref.as_ref() == right_type.as_ref() {
                        // No change.
                    } else {
                        match (left_type_ref.as_ref(), right_type.as_ref()) {
                            (Some(left), Some(right)) => {
                                let left = left.clone();
                                let right = right.clone();
                                drop(left_type_ref);
                                *left_type.borrow_mut() =
                                    Some(CompileValueType::Either(Box::new(left), Box::new(right)));
                            }
                            (_, Some(right)) => {
                                drop(left_type_ref);
                                *left_type.borrow_mut() = Some(CompileValueType::Maybe(Box::new(right.clone())));
                            }
                            (_, _) => {
                                drop(left_type_ref);
                                *left_type.borrow_mut() = None;
                            }
                        }
                    }
                }
            }

            right_type
        }
        Expr::Binary(bin) => match bin.operator {
            TokenType::Plus => {
                let left = infer_type(cx, &bin.left);
                let right = infer_type(cx, &bin.right);

                match (left, right) {
                    (Some(CompileValueType::String), _) => Some(CompileValueType::String),
                    (_, Some(CompileValueType::String)) => Some(CompileValueType::String),
                    (Some(CompileValueType::Number), _) => Some(CompileValueType::Number),
                    (_, Some(CompileValueType::Number)) => Some(CompileValueType::Number),
                    _ => None,
                }
            }
            TokenType::Minus | TokenType::Star | TokenType::Slash => {
                let left = infer_type(cx, &bin.left);
                let right = infer_type(cx, &bin.right);

                match (left, right) {
                    (Some(CompileValueType::Number), _) => Some(CompileValueType::Number),
                    (_, Some(CompileValueType::Number)) => Some(CompileValueType::Number),
                    _ => None,
                }
            }
            _ => None,
        },
        _ => None,
    }
}
