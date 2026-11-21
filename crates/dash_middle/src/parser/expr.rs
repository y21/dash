use crate::compiler::scope::BackLocalId;
use crate::interner::{Symbol, sym};
use crate::lexer::token::TokenType;
use crate::sourcemap::Span;

use super::statement::{Class, FunctionDeclaration};

/// The sequence operator (`expr, expr`)
pub type Seq = (Box<Expr>, Box<Expr>);
/// Any prefix expression, i.e. `++foo`
pub type Prefix = (TokenType, Box<Expr>);
/// Any postfix expression, i.e. `foo++`
pub type Postfix = (TokenType, Box<Expr>);

/// A parsed expression
#[derive(Debug, Clone)]
pub enum ExprKind {
    /// Represents a binary expression
    Binary(BinaryExpr),
    /// Represents a grouping expression
    Grouping(GroupingExpr),
    /// Represents a literal, i.e. `foo`
    Literal(LiteralExpr),
    /// Represents an unary expression, i.e. `-foo`, `+bar`, `await foo`
    Unary(UnaryExpr),
    YieldStar(Box<Expr>),
    /// An assignment expression, i.e. `foo = bar`
    Assignment(AssignmentExpr),
    /// A function call expression
    Call(FunctionCall),
    /// new.target
    NewTarget,
    /// A conditional expression, i.e. `foo ? bar : baz`
    Conditional(ConditionalExpr),
    /// A property access expression, i.e. `foo.bar`
    PropertyAccess(PropertyAccessExpr),
    /// A sequence expression, i.e. `foo, bar`
    Sequence(Seq),
    /// Any prefix expression, i.e. `++foo`
    Prefix(Prefix),
    /// Any postfix expression, i.e. `foo++`
    Postfix(Postfix),
    /// An expression that evaluates to a function object
    ///
    /// This includes both normal functions and arrow functions
    Function(FunctionDeclaration),
    /// A `class {}` expression
    Class(Class),
    /// An array literal expression
    Array(ArrayLiteral),
    /// An object literal expression
    Object(ObjectLiteral),
    /// An optional chaining expression
    Chaining(OptionalChainingExpression),
    /// Compiled bytecode
    Compiled(Vec<u8>),
    /// A dynamic import expression, i.e. `import(specifier, options)`
    Import(DynamicImport),
    /// An empty expression
    Empty,
}

#[derive(Debug, Clone)]
pub struct DynamicImport {
    pub specifier: Box<Expr>,
    pub options: Option<Box<Expr>>,
}

#[derive(Debug, Clone)]
pub struct OptionalChainingExpression {
    pub base: Box<Expr>,
    pub components: Vec<OptionalChainingComponent>,
}

#[derive(Debug, Clone)]
pub enum OptionalChainingComponent {
    /// ?.x
    Ident { property: Symbol, preserve_this: bool },
    /// ?.[expr]
    Dyn { property: Expr, preserve_this: bool },
    /// ?.(args)
    Call(Vec<CallArgumentKind>),
    /// new ...?.(args)
    Construct(Vec<CallArgumentKind>),
}

#[derive(Debug, Clone)]
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
    pub fn assignment_local_space(l: BackLocalId, r: Expr, op: TokenType) -> Self {
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

    pub fn array_literal(members: Vec<ArrayMemberKind>, parenthesized: bool) -> Self {
        Self::Array(ArrayLiteral { members, parenthesized })
    }

    pub fn object_literal(members: Vec<(ObjectMemberKind, Expr)>, parenthesized: bool) -> Self {
        Self::Object(ObjectLiteral { members, parenthesized })
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
        Self::Literal(LiteralExpr::Identifier(sym::undefined))
    }

    pub fn this_literal() -> Self {
        Self::Literal(LiteralExpr::This)
    }

    pub fn regex_literal(regex: dash_regex::Regex, source: Symbol) -> Self {
        Self::Literal(LiteralExpr::Regex(regex, source))
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
    Empty,
}

/// An array literal expression (`[expr, expr]`)
#[derive(Debug, Clone)]
pub struct ArrayLiteral {
    pub members: Vec<ArrayMemberKind>,
    pub parenthesized: bool,
}

#[derive(Debug, Clone)]
pub enum ObjectMemberKind {
    Getter(Symbol),
    DynamicGetter(Expr),
    Setter(Symbol),
    DynamicSetter(Expr),
    Static(Symbol),
    Spread,
    Dynamic(Expr),
    /// `x = 1`
    /// Only valid as part of object destructuring, i.e. `(({ x = 1 }) => x)({})` and `let { x = 1 } = {};`
    Default(Symbol),
}

/// An object literal expression (`{ k: "v" }`)
#[derive(Debug, Clone)]
pub struct ObjectLiteral {
    pub members: Vec<(ObjectMemberKind, Expr)>,
    pub parenthesized: bool,
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

/// A conditional expression
#[derive(Debug, Clone)]
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

/// The target of an assignment
#[derive(Debug, Clone)]
pub enum AssignmentTarget {
    /// Assignment to an expression-place
    Expr(Box<Expr>),
    /// Assignment to a local id (i.e. previously allocated stack space)
    LocalId(BackLocalId),
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
#[derive(Debug, Clone)]
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
    pub fn new_local_place(l: BackLocalId, r: Expr, op: TokenType) -> Self {
        Self {
            left: AssignmentTarget::LocalId(l),
            right: Box::new(r),
            operator: op,
        }
    }
}

/// Any binary expression
#[derive(Debug, Clone)]
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

/// A literal expression
#[derive(Debug, Clone)]
pub enum LiteralExpr {
    /// Boolean literal
    Boolean(bool),
    // Binding(VariableBinding),
    /// Identifier literal (variable lookup)
    Identifier(Symbol),
    /// Number literal
    Number(f64),
    /// String literal, borrowed from input string
    String(Symbol),

    Regex(dash_regex::Regex, Symbol),

    Null,

    This,
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
            Self::String(s) => Some(*s != sym::empty),
            Self::Null => Some(false),
            Self::Regex(..) => Some(true),
            Self::This => None,
        }
    }
}

/// Unary expression
#[derive(Debug, Clone)]
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
