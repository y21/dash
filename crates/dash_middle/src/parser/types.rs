use crate::interner::Symbol;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeSegment {
    Union(Box<TypeSegment>, Box<TypeSegment>),
    Intersect(Box<TypeSegment>, Box<TypeSegment>),
    Array(Box<TypeSegment>),
    Generic(Box<TypeSegment>, Vec<TypeSegment>),
    Literal(LiteralType),
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralType {
    Identifier(Symbol),
    Boolean(bool),
    Number(f64),
}
