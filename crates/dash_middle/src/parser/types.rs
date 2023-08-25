use derive_more::Display;

use crate::interner::Symbol;

fn fmt_segments(s: &[TypeSegment]) -> String {
    s.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(", ")
}

#[derive(Debug, Clone, Display, PartialEq)]
pub enum TypeSegment {
    #[display(fmt = "{_0} | {_1}")]
    Union(Box<TypeSegment>, Box<TypeSegment>),
    #[display(fmt = "{_0} & {_1}")]
    Intersect(Box<TypeSegment>, Box<TypeSegment>),
    #[display(fmt = "{_0}[]")]
    Array(Box<TypeSegment>),
    #[display(fmt = "{_0}<{}>", "fmt_segments(_1)")]
    Generic(Box<TypeSegment>, Vec<TypeSegment>),
    Literal(LiteralType),
    #[display(fmt = "_")]
    Infer,
    #[display(fmt = "any")]
    Any,
    #[display(fmt = "unknown")]
    Unknown,
    #[display(fmt = "never")]
    Never,
    #[display(fmt = "number")]
    Number,
    #[display(fmt = "string")]
    String,
    #[display(fmt = "boolean")]
    Boolean,
    Undefined,
    Null,
}

#[derive(Debug, Clone, Display, PartialEq)]
pub enum LiteralType {
    Identifier(Symbol),
    Boolean(bool),
    Number(f64),
}
