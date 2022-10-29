use derive_more::Display;

fn fmt_segments<'a>(s: &[TypeSegment<'a>]) -> String {
    s.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(", ")
}

#[derive(Debug, Clone, Display, PartialEq, PartialOrd)]
pub enum TypeSegment<'a> {
    #[display(fmt = "{_0} | {_1}")]
    Union(Box<TypeSegment<'a>>, Box<TypeSegment<'a>>),
    #[display(fmt = "{_0} & {_1}")]
    Intersect(Box<TypeSegment<'a>>, Box<TypeSegment<'a>>),
    #[display(fmt = "{_0}[]")]
    Array(Box<TypeSegment<'a>>),
    #[display(fmt = "{_0}<{}>", "fmt_segments(_1)")]
    Generic(Box<TypeSegment<'a>>, Vec<TypeSegment<'a>>),
    Literal(LiteralType<'a>),
}

#[derive(Debug, Clone, Display, PartialEq, PartialOrd)]
pub enum LiteralType<'a> {
    Identifier(&'a str),
    Boolean(bool),
    Number(f64),
}
