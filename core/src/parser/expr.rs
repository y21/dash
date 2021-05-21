use crate::vm::value::{Object, Value, ValueKind};

use super::token::TokenType;

pub type Seq<'a> = (Box<Expr<'a>>, Box<Expr<'a>>);
pub type Postfix<'a> = (TokenType, Box<Expr<'a>>);

#[derive(Debug, Clone)]
pub enum Expr<'a> {
    Binary(BinaryExpr<'a>),
    Grouping(GroupingExpr<'a>),
    Literal(LiteralExpr<'a>),
    Unary(UnaryExpr<'a>),
    Assignment(AssignmentExpr<'a>),
    Call(FunctionCall<'a>),
    Conditional(ConditionalExpr<'a>),
    PropertyAccess(PropertyAccessExpr<'a>),
    Sequence(Seq<'a>),
    Postfix(Postfix<'a>),
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
        Self::Literal(LiteralExpr::Undefined)
    }

    pub fn function_call(target: Expr<'a>, arguments: Vec<Expr<'a>>) -> Self {
        Self::Call(FunctionCall {
            target: Box::new(target),
            arguments,
        })
    }

    pub fn conditional(condition: Expr<'a>, then: Expr<'a>, el: Expr<'a>) -> Self {
        Self::Conditional(ConditionalExpr {
            condition: Box::new(condition),
            then: Box::new(then),
            el: Box::new(el),
        })
    }

    pub fn property_access(computed: bool, target: Expr<'a>, property: Expr<'a>) -> Self {
        Self::PropertyAccess(PropertyAccessExpr {
            computed,
            target: Box::new(target),
            property: Box::new(property),
        })
    }
}

#[derive(Debug, Clone)]
pub struct PropertyAccessExpr<'a> {
    pub computed: bool,
    pub target: Box<Expr<'a>>,
    pub property: Box<Expr<'a>>,
}

#[derive(Debug, Clone)]
pub struct ConditionalExpr<'a> {
    pub condition: Box<Expr<'a>>,
    pub then: Box<Expr<'a>>,
    pub el: Box<Expr<'a>>,
}

#[derive(Debug, Clone)]
pub struct FunctionCall<'a> {
    pub target: Box<Expr<'a>>,
    pub arguments: Vec<Expr<'a>>,
}

#[derive(Debug, Clone)]
pub struct AssignmentExpr<'a> {
    pub left: Box<Expr<'a>>,
    pub right: Box<Expr<'a>>,
    pub operator: TokenType,
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct GroupingExpr<'a>(pub Box<Expr<'a>>);

#[derive(Debug, Clone)]
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
            Self::Boolean(b) => Value::new(ValueKind::Bool(*b)),
            Self::Number(n) => Value::new(ValueKind::Number(*n)),
            Self::Identifier(i) => {
                Value::new(ValueKind::Ident(std::str::from_utf8(i).unwrap().to_owned()))
            }
            Self::String(s) => Value::new(ValueKind::Object(Box::new(Object::String(
                std::str::from_utf8(s).unwrap().to_owned(),
            )))),
            Self::Undefined => Value::new(ValueKind::Undefined),
            Self::Null => Value::new(ValueKind::Null),
        }
    }
}

#[derive(Debug, Clone)]
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
