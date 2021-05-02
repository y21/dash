use crate::vm::value::Value;

use super::token::TokenType;

#[derive(Debug)]
pub enum Expr<'a> {
    Binary(BinaryExpr<'a>),
    Grouping(GroupingExpr<'a>),
    Literal(LiteralExpr<'a>),
    Unary(UnaryExpr<'a>),
    Assignment(AssignmentExpr<'a>),
}

impl<'a> Expr<'a> {
    pub fn binary(l: Expr<'a>, r: Expr<'a>, op: TokenType) -> Self {
        Self::Binary(BinaryExpr::new(l, r, op))
    }

    pub fn grouping(expr: Expr<'a>) -> Self {
        Self::Grouping(GroupingExpr(Box::new(expr)))
    }

    pub fn assignment(l: Expr<'a>, r: Expr<'a>, op: TokenType) -> Self {
        Self::Assignment(AssignmentExpr::new(l, r, op))
    }

    pub fn bool_literal(b: bool) -> Self {
        Self::Literal(LiteralExpr::Boolean(b))
    }

    pub fn number_literal(n: f64) -> Self {
        Self::Literal(LiteralExpr::Number(n))
    }

    pub fn string_literal(s: &'a [u8]) -> Self {
        Self::Literal(LiteralExpr::String(s))
    }

    pub fn identifier(s: &'a [u8]) -> Self {
        Self::Literal(LiteralExpr::Identifier(s))
    }

    pub fn null_literal() -> Self {
        Self::Literal(LiteralExpr::Null)
    }

    pub fn undefined_literal() -> Self {
        Self::Literal(LiteralExpr::Null)
    }
}

#[derive(Debug)]
pub struct AssignmentExpr<'a> {
    left: Box<Expr<'a>>, // ??
    right: Box<Expr<'a>>,
    operator: TokenType,
}

impl<'a> AssignmentExpr<'a> {
    pub fn new(l: Expr<'a>, r: Expr<'a>, op: TokenType) -> Self {
        Self {
            left: Box::new(l),
            right: Box::new(r),
            operator: op,
        }
    }
}

#[derive(Debug)]
pub struct BinaryExpr<'a> {
    pub left: Box<Expr<'a>>,
    pub right: Box<Expr<'a>>,
    pub operator: TokenType,
}

impl<'a> BinaryExpr<'a> {
    pub fn new(l: Expr<'a>, r: Expr<'a>, op: TokenType) -> Self {
        Self {
            left: Box::new(l),
            right: Box::new(r),
            operator: op,
        }
    }
}

#[derive(Debug)]
pub struct GroupingExpr<'a>(pub Box<Expr<'a>>);

#[derive(Debug)]
pub enum LiteralExpr<'a> {
    Boolean(bool),
    Identifier(&'a [u8]),
    Number(f64),
    String(&'a [u8]),
    Null,
    Undefined,
}

impl<'a> LiteralExpr<'a> {
    pub fn to_value(&self) -> Value {
        match self {
            Self::Boolean(b) => Value::Bool(*b),
            Self::Number(n) => Value::Number(*n),
            Self::Identifier(i) => Value::Ident(std::str::from_utf8(i).unwrap().to_owned()),
            _ => unimplemented!(),
        }
    }
}

/*impl<'a> From<LiteralExpr<'a>> for String {
    fn from(e: LiteralExpr<'a>) -> Self {
        match e {
            LiteralExpr::Boolean(b) => b.to_string(),
            LiteralExpr::Identifier(i) => String::from_utf8_lossy(i).to_string(),
            LiteralExpr::Null => String::from("null"),
            LiteralExpr::Undefined => String::from("undefined"),
            LiteralExpr::
        }
    }
}*/

#[derive(Debug)]
pub struct UnaryExpr<'a> {
    pub operator: TokenType,
    pub expr: Box<Expr<'a>>,
}

impl<'a> UnaryExpr<'a> {
    pub fn new(op: TokenType, expr: Expr<'a>) -> Self {
        Self {
            operator: op,
            expr: Box::new(expr),
        }
    }
}
