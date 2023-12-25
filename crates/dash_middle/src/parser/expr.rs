use std::fmt::{self, Debug};

use derive_more::Display;

use crate::interner::{sym, Symbol};
use crate::lexer::token::TokenType;
use crate::sourcemap::Span;

use super::statement::{fmt_list, FunctionDeclaration};

/// The sequence operator (`expr, expr`)
pub type Seq = (Box<Expr>, Box<Expr>);
/// Any prefix expression, i.e. `++foo`
pub type Prefix = (TokenType, Box<Expr>);
/// Any postfix expression, i.e. `foo++`
pub type Postfix = (TokenType, Box<Expr>);

/// A parsed expression
#[derive(Debug, Clone, Display)]
pub enum ExprKind {
    /// Represents a binary expression
    Binary(BinaryExpr),
    /// Represents a grouping expression
    Grouping(GroupingExpr),
    /// Represents a literal, i.e. `foo`
    Literal(LiteralExpr),
    /// Represents an unary expression, i.e. `-foo`, `+bar`, `await foo`
    Unary(UnaryExpr),
    /// An assignment expression, i.e. `foo = bar`
    Assignment(AssignmentExpr),
    /// A function call expression
    Call(FunctionCall),
    /// A conditional expression, i.e. `foo ? bar : baz`
    Conditional(ConditionalExpr),
    /// A property access expression, i.e. `foo.bar`
    PropertyAccess(PropertyAccessExpr),
    /// A sequence expression, i.e. `foo, bar`
    #[display(fmt = "{}, {}", "_0.0", "_0.1")]
    Sequence(Seq),
    /// Any prefix expression, i.e. `++foo`
    #[display(fmt = "{}{}", "_0.0", "_0.1")]
    Prefix(Prefix),
    /// Any postfix expression, i.e. `foo++`
    #[display(fmt = "{}{}", "_0.1", "_0.0")]
    Postfix(Postfix),
    /// An expression that evaluates to a function object
    ///
    /// This includes both normal functions and arrow functions
    Function(FunctionDeclaration),
    /// An array literal expression
    Array(ArrayLiteral),
    /// An object literal expression
    Object(ObjectLiteral),
    /// Compiled bytecode
    #[display(fmt = "<compiled>")]
    Compiled(Vec<u8>),
    /// An empty expression
    Empty,
}

#[derive(Debug, Clone, Display)]
#[display(fmt = "{kind}")]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

impl Expr {
    pub fn binary(l: Expr, r: Expr, op: TokenType) -> Self {
        Self {
            span: l.span.to(r.span),
            kind: ExprKind::binary(l, r, op),
        }
    }

    /// Creates a grouping expression
    ///
    /// NOTE: There must be at least one expression in the group
    pub fn grouping(expr: Vec<Expr>) -> Self {
        Self {
            span: expr.first().unwrap().span.to(expr.last().unwrap().span),
            kind: ExprKind::grouping(expr),
        }
    }

    /// Creates an assignment expression
    pub fn assignment(l: Expr, r: Expr, op: TokenType) -> Self {
        Self {
            span: l.span.to(r.span),
            kind: ExprKind::assignment(l, r, op),
        }
    }

    /// Creates a condition expression
    pub fn conditional(condition: Expr, then: Expr, el: Expr) -> Self {
        Self {
            span: condition.span.to(el.span),
            kind: ExprKind::conditional(condition, then, el),
        }
    }
}

impl ExprKind {
    /// Creates a binary expression
    pub fn binary(l: Expr, r: Expr, op: TokenType) -> Self {
        Self::Binary(BinaryExpr::new(l, r, op))
    }

    /// Creates a grouping expression
    pub fn grouping(expr: Vec<Expr>) -> Self {
        Self::Grouping(GroupingExpr(expr))
    }

    /// Creates an assignment expression
    pub fn assignment(l: Expr, r: Expr, op: TokenType) -> Self {
        Self::Assignment(AssignmentExpr::new_expr_place(l, r, op))
    }

    /// Creates an assignment expression
    pub fn assignment_local_space(l: u16, r: Expr, op: TokenType) -> Self {
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
    pub fn string_literal(s: Symbol) -> Self {
        Self::Literal(LiteralExpr::String(s))
    }

    pub fn array_literal(a: Vec<ArrayMemberKind>) -> Self {
        Self::Array(ArrayLiteral(a))
    }

    pub fn object_literal(o: Vec<(ObjectMemberKind, Expr)>) -> Self {
        Self::Object(ObjectLiteral(o))
    }

    /// Creates an identifier literal expression
    pub fn identifier(s: Symbol) -> Self {
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

    pub fn regex_literal(regex: dash_regex::ParsedRegex, flags: dash_regex::Flags, source: Symbol) -> Self {
        Self::Literal(LiteralExpr::Regex(regex, flags, source))
    }

    /// Creates a function call expression
    pub fn function_call(target: Expr, arguments: Vec<CallArgumentKind>, constructor_call: bool) -> Self {
        Self::Call(FunctionCall {
            constructor_call,
            target: Box::new(target),
            arguments,
        })
    }

    /// Creates a condition expression
    pub fn conditional(condition: Expr, then: Expr, el: Expr) -> Self {
        Self::Conditional(ConditionalExpr {
            condition: Box::new(condition),
            then: Box::new(then),
            el: Box::new(el),
        })
    }

    /// Creates a property access expression
    pub fn property_access(computed: bool, target: Expr, property: Expr) -> Self {
        Self::PropertyAccess(PropertyAccessExpr {
            computed,
            target: Box::new(target),
            property: Box::new(property),
        })
    }

    pub fn unary(op: TokenType, expr: Expr) -> Self {
        Self::Unary(UnaryExpr::new(op, expr))
    }

    pub fn prefix(op: TokenType, expr: Expr) -> Self {
        Self::Prefix((op, Box::new(expr)))
    }

    pub fn postfix(op: TokenType, expr: Expr) -> Self {
        Self::Postfix((op, Box::new(expr)))
    }

    pub fn function(function: FunctionDeclaration) -> Self {
        Self::Function(function)
    }

    pub fn compiled(c: Vec<u8>) -> Self {
        Self::Compiled(c)
    }

    /// Tries to convert an expression into a list of arrow function parameters
    ///
    /// We only know whether a value is an arrow function after parsing
    pub fn to_arrow_function_parameter_list(&self) -> Option<Vec<Symbol>> {
        match &self {
            ExprKind::Grouping(g) => {
                let mut list = Vec::with_capacity(g.0.len());
                for expr in &g.0 {
                    list.push(expr.kind.as_identifier()?);
                }
                Some(list)
            }
            ExprKind::Literal(LiteralExpr::Identifier(ident)) => Some(vec![*ident]),
            _ => None,
        }
    }

    /// Tries to return the identifier that is associated to this expression
    pub fn as_identifier(&self) -> Option<Symbol> {
        match &self {
            ExprKind::Literal(LiteralExpr::Identifier(ident)) => Some(*ident),
            _ => None,
        }
    }

    pub fn is_truthy(&self) -> Option<bool> {
        match &self {
            ExprKind::Literal(lit) => lit.is_truthy(),
            ExprKind::Assignment(ass) => ass.right.kind.is_truthy(),
            ExprKind::Grouping(GroupingExpr(group)) => group.last().and_then(|e| e.kind.is_truthy()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ArrayMemberKind {
    Item(Expr),
    Spread(Expr),
}

impl fmt::Display for ArrayMemberKind {
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
pub struct ArrayLiteral(pub Vec<ArrayMemberKind>);

impl fmt::Display for ArrayLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        fmt_list(f, &self.0, ",")?;
        write!(f, "]")
    }
}

#[derive(Debug, Clone)]
pub enum ObjectMemberKind {
    Getter(Symbol),
    Setter(Symbol),
    Static(Symbol),
    Spread,
    Dynamic(Expr),
}

impl fmt::Display for ObjectMemberKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Getter(name) => write!(f, "get {name}"),
            Self::Setter(name) => write!(f, "set {name}"),
            Self::Static(name) => write!(f, "{name}"),
            Self::Dynamic(expr) => write!(f, "[{expr}]"),
            Self::Spread => f.write_str("...<expression unavailable>"), // TODO: figure out a way to display it here
        }
    }
}

/// An object literal expression (`{ k: "v" }`)
#[derive(Debug, Clone)]
pub struct ObjectLiteral(pub Vec<(ObjectMemberKind, Expr)>);

impl fmt::Display for ObjectLiteral {
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
pub struct PropertyAccessExpr {
    /// Whether this property access is computed
    pub computed: bool,
    /// The target object that is accessed
    pub target: Box<Expr>,
    /// The property of the object that is accessed
    pub property: Box<Expr>,
}

impl fmt::Display for PropertyAccessExpr {
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
pub struct ConditionalExpr {
    /// The first part of a conditional expression, the condition
    pub condition: Box<Expr>,
    /// The second part of a conditional expression, a then expression
    pub then: Box<Expr>,
    /// The last part of a conditional expression, an else expression
    pub el: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum CallArgumentKind {
    /// A normal argument
    Normal(Expr),
    /// A spread argument
    Spread(Expr),
}

impl fmt::Display for CallArgumentKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CallArgumentKind::Normal(expr) => fmt::Display::fmt(expr, f),
            CallArgumentKind::Spread(expr) => {
                f.write_str("...")?;
                fmt::Display::fmt(expr, f)
            }
        }
    }
}

/// A function call expression
#[derive(Debug, Clone)]
pub struct FunctionCall {
    /// Whether this function call invokes the constructor (using `new` keyword)
    pub constructor_call: bool,
    /// The target (callee)
    pub target: Box<Expr>,
    /// Function call arguments
    pub arguments: Vec<CallArgumentKind>,
}

impl fmt::Display for FunctionCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}(", self.target)?;
        fmt_list(f, &self.arguments, ",")?;
        write!(f, ")")
    }
}

/// The target of an assignment
#[derive(Debug, Clone, Display)]
pub enum AssignmentTarget {
    /// Assignment to an expression-place
    Expr(Box<Expr>),
    /// Assignment to a local id (i.e. previously allocated stack space)
    LocalId(u16),
}

impl AssignmentTarget {
    pub fn as_expr(&self) -> Option<&Expr> {
        match self {
            Self::Expr(e) => Some(e),
            _ => None,
        }
    }
}

/// An assignment expression
#[derive(Debug, Clone, Display)]
#[display(fmt = "{left} {operator} {right}")]
pub struct AssignmentExpr {
    /// The lefthand side (place-expression)
    pub left: AssignmentTarget,
    /// The righthand side (value)
    pub right: Box<Expr>,
    /// The type of assignment, (`=`/`+=`/etc)
    pub operator: TokenType,
}

impl AssignmentExpr {
    /// Creates a new assignment expression
    pub fn new(l: AssignmentTarget, r: Expr, op: TokenType) -> Self {
        Self {
            left: l,
            right: Box::new(r),
            operator: op,
        }
    }
    /// Convenient method for `AssignmentExpr::new(AssignmentTarget::Expr(Box::new(left)), right, op)`
    pub fn new_expr_place(l: Expr, r: Expr, op: TokenType) -> Self {
        Self {
            left: AssignmentTarget::Expr(Box::new(l)),
            right: Box::new(r),
            operator: op,
        }
    }
    /// Convenient method for `AssignmentExpr::new(AssignmentTarget::LocalId(left), right, op)`
    pub fn new_local_place(l: u16, r: Expr, op: TokenType) -> Self {
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
pub struct BinaryExpr {
    /// Lefthand side
    pub left: Box<Expr>,
    /// Righthand side
    pub right: Box<Expr>,
    /// Operator
    pub operator: TokenType,
}

impl BinaryExpr {
    /// Creates a new binary expression
    pub fn new(l: Expr, r: Expr, op: TokenType) -> Self {
        Self {
            left: Box::new(l),
            right: Box::new(r),
            operator: op,
        }
    }
}

/// A grouping expression
#[derive(Debug, Clone)]
pub struct GroupingExpr(pub Vec<Expr>);

impl fmt::Display for GroupingExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        fmt_list(f, &self.0, ",")?;
        write!(f, ")")
    }
}

/// A literal expression
#[derive(Debug, Clone, Display)]
pub enum LiteralExpr {
    /// Boolean literal
    Boolean(bool),
    // Binding(VariableBinding),
    /// Identifier literal (variable lookup)
    Identifier(Symbol),
    /// Number literal
    Number(f64),
    /// String literal, borrowed from input string
    #[display(fmt = "\"{_0}\"")]
    String(Symbol),

    #[display(fmt = "/{_2}/")]
    Regex(dash_regex::ParsedRegex, dash_regex::Flags, Symbol),

    #[display(fmt = "null")]
    Null,

    #[display(fmt = "undefined")]
    Undefined,
}

impl LiteralExpr {
    /// Checks whether this literal is always a truthy value
    ///
    /// The optimizer may use this for optimizing potential branches
    pub fn is_truthy(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            Self::Identifier(_) => None,
            Self::Number(n) => Some(*n != 0.0),
            Self::String(s) => Some(*s != sym::EMPTY),
            Self::Null => Some(false),
            Self::Undefined => Some(false),
            Self::Regex(..) => Some(true),
        }
    }
}

/// Unary expression
#[derive(Debug, Clone, Display)]
#[display(fmt = "{operator}{expr}")]
pub struct UnaryExpr {
    /// The operator that was used
    pub operator: TokenType,
    /// Expression
    pub expr: Box<Expr>,
}

impl UnaryExpr {
    /// Creates a new unary expression
    pub fn new(op: TokenType, expr: Expr) -> Self {
        Self {
            operator: op,
            expr: Box::new(expr),
        }
    }
}
