use super::{statement::FunctionDeclaration, token::TokenType};

/// The sequence operator (`expr, expr`)
pub type Seq<'a> = (Box<Expr<'a>>, Box<Expr<'a>>);
/// Any postfix expression, i.e. `foo++`
pub type Postfix<'a> = (TokenType, Box<Expr<'a>>);
/// An array literal expression (`[expr, expr]`)
pub type ArrayLiteral<'a> = Vec<Expr<'a>>;
/// An object literal expression (`{ k: "v" }`)
pub type ObjectLiteral<'a> = Vec<(/*(Expr<'a>*/ &'a [u8], Expr<'a>)>;

/// A parsed expression
#[derive(Debug, Clone)]
pub enum Expr<'a> {
    /// Represents any binary expression, i.e. `foo + bar`
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
    Sequence(Seq<'a>),
    /// Any postfix expression, i.e. `foo++`
    Postfix(Postfix<'a>),
    /// An expression that evaluates to a function object
    ///
    /// This includes both normal functions and arrow functions
    Function(FunctionDeclaration<'a>),
    /// An array literal expression
    Array(ArrayLiteral<'a>),
    /// An object literal expression
    Object(ObjectLiteral<'a>),
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
        Self::Assignment(AssignmentExpr::new(l, r, op))
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
    pub fn string_literal(s: &'a [u8]) -> Self {
        Self::Literal(LiteralExpr::String(s))
    }

    /// Creates an identifier literal expression
    pub fn identifier(s: &'a [u8]) -> Self {
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

    /// Creates a function call expression
    pub fn function_call(
        target: Expr<'a>,
        arguments: Vec<Expr<'a>>,
        constructor_call: bool,
    ) -> Self {
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

    /// Tries to convert an expression into a list of arrow function parameters
    ///
    /// We only know whether a value is an arrow function after parsing
    pub fn to_arrow_function_parameter_list(&self) -> Option<Vec<&'a [u8]>> {
        match self {
            Self::Grouping(g) => {
                let mut list = Vec::with_capacity(g.0.len());
                for expr in &g.0 {
                    list.push(expr.to_identifier()?);
                }
                Some(list)
            }
            Self::Literal(lit) => Some(vec![lit.as_identifier()?]),
            _ => None,
        }
    }

    /// Tries to return the identifier that is associated to this expression
    pub fn to_identifier(&self) -> Option<&'a [u8]> {
        match self {
            Self::Literal(lit) => lit.as_identifier(),
            _ => None,
        }
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

/// A conditional expression
#[derive(Debug, Clone)]
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

/// An assignment expression
#[derive(Debug, Clone)]
pub struct AssignmentExpr<'a> {
    /// The lefthand side (target)
    pub left: Box<Expr<'a>>,
    /// The righthand side (value)
    pub right: Box<Expr<'a>>,
    /// The type of assignment, (`=`/`+=`/etc)
    pub operator: TokenType,
}

impl<'a> AssignmentExpr<'a> {
    /// Creates a new assignment expression
    pub fn new(l: Expr<'a>, r: Expr<'a>, op: TokenType) -> Self {
        Self {
            left: Box::new(l),
            right: Box::new(r),
            operator: op,
        }
    }
}

/// Any binary expression
#[derive(Debug, Clone)]
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

/// A literal expression
#[derive(Debug, Clone)]
pub enum LiteralExpr<'a> {
    /// Boolean literal
    Boolean(bool),
    /// Identifier literal (variable lookup)
    Identifier(&'a [u8]),
    /// Number literal
    Number(f64),
    /// String literal, borrowed from input string
    String(&'a [u8]),
    /// Null literal
    Null,
    /// Undefined literal
    Undefined,
}

impl<'a> LiteralExpr<'a> {
    /// Tries to get the identifier of a literal, if present
    pub fn as_identifier(&self) -> Option<&'a [u8]> {
        match self {
            Self::Boolean(b) => Some(b.then(|| b"true" as &[u8]).unwrap_or(b"false" as &[u8])),
            Self::Identifier(ident) => Some(ident),
            Self::Undefined => Some(b"undefined"),
            _ => None,
        }
    }
}

/// Unary expression
#[derive(Debug, Clone)]
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
