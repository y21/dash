use std::{
    borrow::Cow,
    fmt::{self, Debug},
};

use derive_more::Display;

use crate::lexer::token::TokenType;

use super::statement::{fmt_list, FunctionDeclaration};

/// The sequence operator (`expr, expr`)
pub type Seq<'a> = (Box<Expr<'a>>, Box<Expr<'a>>);
/// Any prefix expression, i.e. `++foo`
pub type Prefix<'a> = (TokenType, Box<Expr<'a>>);
/// Any postfix expression, i.e. `foo++`
pub type Postfix<'a> = (TokenType, Box<Expr<'a>>);

/// A parsed expression
#[derive(Debug, Clone, Display)]
pub enum Expr<'a> {
    /// Represents a binary expression
    Binary(BinaryExpr<'a>),
    /// Represents a grouping expression
    Grouping(GroupingExpr<'a>),
    /// Represents a literal, i.e. `foo`
    Literal(LiteralExpr<'a>),
    /// Represents an unary expression, i.e. `-foo`, `+bar`, `await foo`
    Unary(UnaryExpr<'a>),
    /// An assignment expression, i.e. `foo = bar`
    Assignment(AssignmentExpr<'a>),
    /// A function call expression
    Call(FunctionCall<'a>),
    /// A conditional expression, i.e. `foo ? bar : baz`
    Conditional(ConditionalExpr<'a>),
    /// A property access expression, i.e. `foo.bar`
    PropertyAccess(PropertyAccessExpr<'a>),
    /// A sequence expression, i.e. `foo, bar`
    #[display(fmt = "{}, {}", "_0.0", "_0.1")]
    Sequence(Seq<'a>),
    /// Any prefix expression, i.e. `++foo`
    #[display(fmt = "{}{}", "_0.0", "_0.1")]
    Prefix(Prefix<'a>),
    /// Any postfix expression, i.e. `foo++`
    #[display(fmt = "{}{}", "_0.1", "_0.0")]
    Postfix(Postfix<'a>),
    /// An expression that evaluates to a function object
    ///
    /// This includes both normal functions and arrow functions
    Function(FunctionDeclaration<'a>),
    /// An array literal expression
    Array(ArrayLiteral<'a>),
    /// An object literal expression
    Object(ObjectLiteral<'a>),
    /// Compiled bytecode
    #[display(fmt = "<compiled>")]
    Compiled(Vec<u8>),
    /// An empty expression
    Empty,
}

impl<'a> Expr<'a> {
    /// Creates a binary expression
    pub fn binary(l: Expr<'a>, r: Expr<'a>, op: TokenType) -> Self {
        Self::Binary(BinaryExpr::new(l, r, op))
    }

    /// Creates a grouping expression
    pub fn grouping(expr: Vec<Expr<'a>>) -> Self {
        Self::Grouping(GroupingExpr(expr))
    }

    /// Creates an assignment expression
    pub fn assignment(l: Expr<'a>, r: Expr<'a>, op: TokenType) -> Self {
        Self::Assignment(AssignmentExpr::new_expr_place(l, r, op))
    }

    /// Creates an assignment expression
    pub fn assignment_local_space(l: u16, r: Expr<'a>, op: TokenType) -> Self {
        Self::Assignment(AssignmentExpr::new_local_place(l, r, op))
    }

    /// Creates a bool literal expression
    pub fn bool_literal(b: bool) -> Self {
        Self::Literal(LiteralExpr::Boolean(b))
    }

    /// Creates a number literal expression
    pub fn number_literal(n: f64) -> Self {
        Self::Literal(LiteralExpr::Number(n))
    }

    /// Creates a string literal expression
    pub fn string_literal(s: Cow<'a, str>) -> Self {
        Self::Literal(LiteralExpr::String(s))
    }

    pub fn array_literal(a: Vec<ArrayMemberKind<'a>>) -> Self {
        Self::Array(ArrayLiteral(a))
    }

    pub fn object_literal(o: Vec<(ObjectMemberKind<'a>, Expr<'a>)>) -> Self {
        Self::Object(ObjectLiteral(o))
    }

    /// Creates an identifier literal expression
    pub fn identifier(s: Cow<'a, str>) -> Self {
        Self::Literal(LiteralExpr::Identifier(s))
    }

    /// Creates a null literal expression
    pub fn null_literal() -> Self {
        Self::Literal(LiteralExpr::Null)
    }

    /// Creates an undefined literal expression
    pub fn undefined_literal() -> Self {
        Self::Literal(LiteralExpr::Undefined)
    }

    pub fn regex_literal(regex: dash_regex::Regex, source: &'a str) -> Self {
        Self::Literal(LiteralExpr::Regex(regex, source))
    }

    /// Creates a function call expression
    pub fn function_call(target: Expr<'a>, arguments: Vec<Expr<'a>>, constructor_call: bool) -> Self {
        Self::Call(FunctionCall {
            constructor_call,
            target: Box::new(target),
            arguments,
        })
    }

    /// Creates a condition expression
    pub fn conditional(condition: Expr<'a>, then: Expr<'a>, el: Expr<'a>) -> Self {
        Self::Conditional(ConditionalExpr {
            condition: Box::new(condition),
            then: Box::new(then),
            el: Box::new(el),
        })
    }

    /// Creates a property access expression
    pub fn property_access(computed: bool, target: Expr<'a>, property: Expr<'a>) -> Self {
        Self::PropertyAccess(PropertyAccessExpr {
            computed,
            target: Box::new(target),
            property: Box::new(property),
        })
    }

    pub fn unary(op: TokenType, expr: Expr<'a>) -> Self {
        Self::Unary(UnaryExpr::new(op, expr))
    }

    pub fn prefix(op: TokenType, expr: Expr<'a>) -> Self {
        Self::Prefix((op, Box::new(expr)))
    }

    pub fn postfix(op: TokenType, expr: Expr<'a>) -> Self {
        Self::Postfix((op, Box::new(expr)))
    }

    pub fn function(function: FunctionDeclaration<'a>) -> Self {
        Self::Function(function)
    }

    pub fn compiled(c: Vec<u8>) -> Self {
        Self::Compiled(c)
    }

    /// Tries to convert an expression into a list of arrow function parameters
    ///
    /// We only know whether a value is an arrow function after parsing
    pub fn to_arrow_function_parameter_list(&self) -> Option<Vec<&'a str>> {
        match &self {
            Expr::Grouping(g) => {
                let mut list = Vec::with_capacity(g.0.len());
                for expr in &g.0 {
                    list.push(expr.as_identifier()?);
                }
                Some(list)
            }
            Expr::Literal(lit) => Some(vec![lit.as_identifier_borrowed()?]),
            _ => None,
        }
    }

    /// Tries to return the identifier that is associated to this expression
    pub fn as_identifier(&self) -> Option<&'a str> {
        match &self {
            Expr::Literal(lit) => lit.as_identifier_borrowed(),
            _ => None,
        }
    }

    pub fn is_truthy(&self) -> Option<bool> {
        match &self {
            Expr::Literal(lit) => lit.is_truthy(),
            Expr::Assignment(ass) => ass.right.is_truthy(),
            Expr::Grouping(GroupingExpr(group)) => group.last().and_then(|e| e.is_truthy()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ArrayMemberKind<'a> {
    Item(Expr<'a>),
    Spread(Expr<'a>),
}
impl<'a> fmt::Display for ArrayMemberKind<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArrayMemberKind::Item(item) => fmt::Display::fmt(item, f),
            ArrayMemberKind::Spread(item) => {
                f.write_str("...")?;
                fmt::Display::fmt(item, f)
            }
        }
    }
}

/// An array literal expression (`[expr, expr]`)
#[derive(Debug, Clone)]
pub struct ArrayLiteral<'a>(pub Vec<ArrayMemberKind<'a>>);

impl<'a> fmt::Display for ArrayLiteral<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        fmt_list(f, &self.0, ",")?;
        write!(f, "]")
    }
}

#[derive(Debug, Clone)]
pub enum ObjectMemberKind<'a> {
    Getter(Cow<'a, str>),
    Setter(Cow<'a, str>),
    Static(&'a str),
    Spread,
    Dynamic(Expr<'a>),
}

impl<'a> fmt::Display for ObjectMemberKind<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Getter(name) => write!(f, "get {name}"),
            Self::Setter(name) => write!(f, "set {name}"),
            Self::Static(name) => f.write_str(name),
            Self::Dynamic(expr) => write!(f, "[{expr}]"),
            Self::Spread => f.write_str("...<expression unavailable>"), // TODO: figure out a way to display it here
        }
    }
}

/// An object literal expression (`{ k: "v" }`)
#[derive(Debug, Clone)]
pub struct ObjectLiteral<'a>(pub Vec<(ObjectMemberKind<'a>, Expr<'a>)>);

impl<'a> fmt::Display for ObjectLiteral<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;

        for (i, (k, v)) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{k}: ")?;
            write!(f, "{v}")?;
        }

        write!(f, "}}")
    }
}

/// A property access expression
#[derive(Debug, Clone)]
pub struct PropertyAccessExpr<'a> {
    /// Whether this property access is computed
    pub computed: bool,
    /// The target object that is accessed
    pub target: Box<Expr<'a>>,
    /// The property of the object that is accessed
    pub property: Box<Expr<'a>>,
}

impl<'a> fmt::Display for PropertyAccessExpr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.target)?;

        if self.computed {
            write!(f, "[{}]", self.property)?;
        } else {
            write!(f, ".{}", self.property)?;
        }

        Ok(())
    }
}

/// A conditional expression
#[derive(Debug, Clone, Display)]
#[display(fmt = "{condition} ? {then} : {el}")]
pub struct ConditionalExpr<'a> {
    /// The first part of a conditional expression, the condition
    pub condition: Box<Expr<'a>>,
    /// The second part of a conditional expression, a then expression
    pub then: Box<Expr<'a>>,
    /// The last part of a conditional expression, an else expression
    pub el: Box<Expr<'a>>,
}

/// A function call expression
#[derive(Debug, Clone)]
pub struct FunctionCall<'a> {
    /// Whether this function call invokes the constructor (using `new` keyword)
    pub constructor_call: bool,
    /// The target (callee)
    pub target: Box<Expr<'a>>,
    /// Function call arguments
    pub arguments: Vec<Expr<'a>>,
}

impl<'a> fmt::Display for FunctionCall<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}(", self.target)?;
        fmt_list(f, &self.arguments, ",")?;
        write!(f, ")")
    }
}

/// The target of an assignment
#[derive(Debug, Clone, Display)]
pub enum AssignmentTarget<'a> {
    /// Assignment to an expression-place
    Expr(Box<Expr<'a>>),
    /// Assignment to a local id (i.e. previously allocated stack space)
    LocalId(u16),
}

impl<'a> AssignmentTarget<'a> {
    pub fn as_expr(&self) -> Option<&Expr<'a>> {
        match self {
            Self::Expr(e) => Some(e),
            _ => None,
        }
    }
}

/// An assignment expression
#[derive(Debug, Clone, Display)]
#[display(fmt = "{left} {operator} {right}")]
pub struct AssignmentExpr<'a> {
    /// The lefthand side (place-expression)
    pub left: AssignmentTarget<'a>,
    /// The righthand side (value)
    pub right: Box<Expr<'a>>,
    /// The type of assignment, (`=`/`+=`/etc)
    pub operator: TokenType,
}

impl<'a> AssignmentExpr<'a> {
    /// Creates a new assignment expression
    pub fn new(l: AssignmentTarget<'a>, r: Expr<'a>, op: TokenType) -> Self {
        Self {
            left: l,
            right: Box::new(r),
            operator: op,
        }
    }
    /// Convenient method for `AssignmentExpr::new(AssignmentTarget::Expr(Box::new(left)), right, op)`
    pub fn new_expr_place(l: Expr<'a>, r: Expr<'a>, op: TokenType) -> Self {
        Self {
            left: AssignmentTarget::Expr(Box::new(l)),
            right: Box::new(r),
            operator: op,
        }
    }
    /// Convenient method for `AssignmentExpr::new(AssignmentTarget::LocalId(left), right, op)`
    pub fn new_local_place(l: u16, r: Expr<'a>, op: TokenType) -> Self {
        Self {
            left: AssignmentTarget::LocalId(l),
            right: Box::new(r),
            operator: op,
        }
    }
}

/// Any binary expression
#[derive(Debug, Clone, Display)]
#[display(fmt = "{left} {operator} {right}")]
pub struct BinaryExpr<'a> {
    /// Lefthand side
    pub left: Box<Expr<'a>>,
    /// Righthand side
    pub right: Box<Expr<'a>>,
    /// Operator
    pub operator: TokenType,
}

impl<'a> BinaryExpr<'a> {
    /// Creates a new binary expression
    pub fn new(l: Expr<'a>, r: Expr<'a>, op: TokenType) -> Self {
        Self {
            left: Box::new(l),
            right: Box::new(r),
            operator: op,
        }
    }
}

/// A grouping expression
#[derive(Debug, Clone)]
pub struct GroupingExpr<'a>(pub Vec<Expr<'a>>);

impl<'a> fmt::Display for GroupingExpr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        fmt_list(f, &self.0, ",")?;
        write!(f, ")")
    }
}

/// A literal expression
#[derive(Debug, Clone, Display)]
pub enum LiteralExpr<'a> {
    /// Boolean literal
    Boolean(bool),
    // Binding(VariableBinding<'a>),
    /// Identifier literal (variable lookup)
    Identifier(Cow<'a, str>),
    /// Number literal
    Number(f64),
    /// String literal, borrowed from input string
    #[display(fmt = "\"{_0}\"")]
    String(Cow<'a, str>),

    #[display(fmt = "/{_1}/")]
    Regex(dash_regex::Regex, &'a str),

    #[display(fmt = "null")]
    Null,

    #[display(fmt = "undefined")]
    Undefined,
}

impl<'a> LiteralExpr<'a> {
    /// Tries to get the identifier of a literal, if present
    pub fn as_identifier_borrowed(&self) -> Option<&'a str> {
        match self {
            Self::Boolean(b) => Some(b.then(|| "true").unwrap_or("false")),
            Self::Identifier(Cow::Borrowed(i)) => Some(i),
            Self::Undefined => Some("undefined"),
            Self::Null => Some("null"),
            Self::String(Cow::Borrowed(s)) => Some(s),
            _ => None,
        }
    }

    /// Converts the identifier of a literal
    pub fn to_identifier(&self) -> Cow<'a, str> {
        match self {
            Self::Boolean(b) => Cow::Borrowed(b.then(|| "true").unwrap_or("false")),
            Self::Identifier(ident) => ident.clone(),
            Self::Undefined => Cow::Borrowed("undefined"),
            Self::Null => Cow::Borrowed("null"),
            Self::Number(n) => Cow::Owned(n.to_string()),
            Self::String(s) => s.clone(),
            Self::Regex(_, s) => Cow::Borrowed(*s),
        }
    }

    /// Checks whether this literal is always a truthy value
    ///
    /// The optimizer may use this for optimizing potential branches
    pub fn is_truthy(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            Self::Identifier(_) => None,
            Self::Number(n) => Some(*n != 0.0),
            Self::String(s) => Some(!s.is_empty()),
            Self::Null => Some(false),
            Self::Undefined => Some(false),
            Self::Regex(..) => Some(true),
        }
    }
}

/// Unary expression
#[derive(Debug, Clone, Display)]
#[display(fmt = "{operator}{expr}")]
pub struct UnaryExpr<'a> {
    /// The operator that was used
    pub operator: TokenType,
    /// Expression
    pub expr: Box<Expr<'a>>,
}

impl<'a> UnaryExpr<'a> {
    /// Creates a new unary expression
    pub fn new(op: TokenType, expr: Expr<'a>) -> Self {
        Self {
            operator: op,
            expr: Box::new(expr),
        }
    }
}
