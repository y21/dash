use std::{cell::RefCell, fmt};

use derive_more::Display;

use crate::lexer::token::TokenType;

use super::expr::Expr;

/// A JavaScript statement
#[derive(Debug, Clone, Display)]
pub enum Statement<'a> {
    /// Expression statement
    Expression(Expr<'a>),
    /// Variable declaration
    Variable(VariableDeclaration<'a>),
    /// If statement
    If(IfStatement<'a>),
    /// Block statement
    Block(BlockStatement<'a>),
    /// Function declaration
    Function(FunctionDeclaration<'a>),
    /// Any loop
    Loop(Loop<'a>),
    /// Return statement
    Return(ReturnStatement<'a>),
    /// Try catch block
    Try(TryCatch<'a>),
    /// Throw statement
    Throw(Expr<'a>),
    /// Import statement
    Import(ImportKind<'a>),
    /// Export statement
    Export(ExportKind<'a>),
    /// Class declaration
    Class(Class<'a>),
    /// Continue loop statement
    #[display(fmt = "continue;")]
    Continue,
    /// Break loop statement
    #[display(fmt = "break;")]
    Break,
    /// Debugger statement
    #[display(fmt = "debugger;")]
    Debugger,
    /// An empty statement
    ///
    /// This is impossible to occur in JavaScript code, however a statement may be folded to an empty statement
    /// if it does not have any side effects.
    #[display(fmt = ";")]
    Empty,
}

/// The type of a specifier
///
/// This is used in import/export statements, as well as variable declaration
/// in the future. When destructuring is implemented, this enum will make more sense.
#[derive(Debug, Clone, Display)]
pub enum SpecifierKind<'a> {
    /// A raw identifier
    #[display(fmt = "{_0}")]
    Ident(&'a str),
}

impl<'a> SpecifierKind<'a> {
    /// Attempts to return self as an identifier
    pub fn as_ident(&self) -> Option<&'a str> {
        match self {
            Self::Ident(i) => Some(i),
        }
    }
}

/// Type of import statement
#[derive(Debug, Clone, Display)]
pub enum ImportKind<'a> {
    #[display(fmt = "import({_0})")]
    Dynamic(Expr<'a>),
    /// import foo from "bar"
    #[display(fmt = "import {_0} from \"{_1}\"")]
    DefaultAs(SpecifierKind<'a>, &'a str),
    /// import * as foo from "bar"
    #[display(fmt = "import * as {_0} from \"{_1}\"")]
    AllAs(SpecifierKind<'a>, &'a str),
}

/// Type of export statement
#[derive(Debug, Clone)]
pub enum ExportKind<'a> {
    /// export default foo
    Default(Expr<'a>),
    /// export { foo, bar }
    Named(Vec<&'a str>),
    /// export let foo = "bar"
    NamedVar(Vec<VariableDeclaration<'a>>),
}

impl<'a> fmt::Display for ExportKind<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Default(e) => write!(f, "export default {e}"),
            Self::Named(es) => {
                write!(f, "export {{ ")?;
                fmt_list(f, es, ",")?;
                write!(f, " }}")
            }
            Self::NamedVar(nv) => {
                write!(f, "export {{ ")?;
                fmt_list(f, nv, ",")?;
                write!(f, " }}")
            }
        }
    }
}

impl<'a> ImportKind<'a> {
    /// Attempts to return the underlying [SpecifierKind], if present
    pub fn get_specifier(&self) -> Option<&SpecifierKind<'a>> {
        match self {
            Self::Dynamic(_) => None,
            Self::DefaultAs(s, _) => Some(s),
            Self::AllAs(s, _) => Some(s),
        }
    }

    /// Attempts to return the underlying module target, if present
    pub fn get_module_target(&self) -> Option<&'a str> {
        match self {
            Self::Dynamic(_) => None,
            Self::DefaultAs(_, i) => Some(i),
            Self::AllAs(_, i) => Some(i),
        }
    }
}

/// A catch statement
#[derive(Debug, Clone, Display)]
#[display(fmt = "catch ({}) {{ {} }}", "ident.unwrap_or(\"_\")", "body")]
pub struct Catch<'a> {
    /// The body of a catch statement
    pub body: Box<Statement<'a>>,
    /// The identifier of the variable that receives the thrown error
    pub ident: Option<&'a str>,
}

impl<'a> Catch<'a> {
    /// Creates a new catch statement
    pub fn new(body: Statement<'a>, ident: Option<&'a str>) -> Self {
        Self {
            body: Box::new(body),
            ident,
        }
    }
}

/// A try catch statement
#[derive(Debug, Clone)]
pub struct TryCatch<'a> {
    /// The body of the try statement
    pub try_: Box<Statement<'a>>,
    /// Catch statement
    // TODO: make this optional. a try can exist without catch (try finally)
    pub catch: Catch<'a>,
    /// Optional finally block
    pub finally: Option<Box<Statement<'a>>>,
}

impl<'a> fmt::Display for TryCatch<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "try {{ {} }} {}", self.try_, self.catch)?;

        if let Some(finally) = &self.finally {
            write!(f, " finally {{ {} }}", finally)?;
        }

        Ok(())
    }
}

impl<'a> TryCatch<'a> {
    /// Creates a new try catch block
    pub fn new(try_: Statement<'a>, catch: Catch<'a>, finally: Option<Statement<'a>>) -> Self {
        Self {
            try_: Box::new(try_),
            catch,
            finally: finally.map(Box::new),
        }
    }
}

/// A return statement
#[derive(Debug, Clone, Display)]
#[display(fmt = "return {}", _0)]
pub struct ReturnStatement<'a>(pub Expr<'a>);

impl<'a> Default for ReturnStatement<'a> {
    fn default() -> Self {
        Self(Expr::undefined_literal())
    }
}

/// A loop statement
#[derive(Debug, Clone, Display)]
pub enum Loop<'a> {
    /// A for loop
    For(ForLoop<'a>),
    /// A for..of loop
    ForOf(ForOfLoop<'a>),
    /// A while loop
    While(WhileLoop<'a>),
}

impl<'a> From<ForLoop<'a>> for Loop<'a> {
    fn from(f: ForLoop<'a>) -> Self {
        Self::For(f)
    }
}

impl<'a> From<WhileLoop<'a>> for Loop<'a> {
    fn from(f: WhileLoop<'a>) -> Self {
        Self::While(f)
    }
}

/// A for..of loop
#[derive(Debug, Clone, Display)]
#[display(fmt = "for ({} of {}) {{ {} }}", binding, expr, body)]
pub struct ForOfLoop<'a> {
    /// The binding of this loop
    pub binding: VariableBinding<'a>,
    /// The expression to iterate over
    pub expr: Expr<'a>,
    /// The body of this loop
    pub body: Box<Statement<'a>>,
}

/// A for loop
#[derive(Debug, Clone)]
pub struct ForLoop<'a> {
    /// The initializer of a for loop
    pub init: Option<Box<Statement<'a>>>,
    /// The condition that is used to determine when iteration should stop
    pub condition: Option<Expr<'a>>,
    /// Final expression, evaluated after each iteration
    pub finalizer: Option<Expr<'a>>,
    /// The body of a for loop
    pub body: Box<Statement<'a>>,
}

impl<'a> fmt::Display for ForLoop<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "for(")?;

        if let Some(init) = &self.init {
            write!(f, "{}", init)?;
        }

        write!(f, ";")?;

        if let Some(condition) = &self.condition {
            write!(f, "{}", condition)?;
        }

        write!(f, ";")?;

        if let Some(finalizer) = &self.finalizer {
            write!(f, "{}", finalizer)?;
        }

        write!(f, ") {{ {} }}", self.body)
    }
}

impl<'a> ForLoop<'a> {
    /// Creates a new for loop
    pub fn new(
        init: Option<Statement<'a>>,
        condition: Option<Expr<'a>>,
        finalizer: Option<Expr<'a>>,
        body: Statement<'a>,
    ) -> Self {
        Self {
            init: init.map(Box::new),
            condition,
            finalizer,
            body: Box::new(body),
        }
    }
}

/// A while loop
#[derive(Debug, Clone, Display)]
#[display(fmt = "while ({}) {{ {} }}", condition, body)]
pub struct WhileLoop<'a> {
    /// The condition of this while loop, used to determine when to stop iterating
    pub condition: Expr<'a>,
    /// The body of this while loop
    pub body: Box<Statement<'a>>,
}

impl<'a> WhileLoop<'a> {
    /// Creates a new while loop
    pub fn new(condition: Expr<'a>, body: Statement<'a>) -> Self {
        Self {
            condition: condition,
            body: Box::new(body),
        }
    }
}

/// The type of function
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FunctionKind {
    /// A normal function
    Function,
    /// A generator function
    Generator,
    /// An arrow function
    Arrow,
}

/// A function declaration
#[derive(Debug, Clone)]
pub struct FunctionDeclaration<'a> {
    /// The name of this function, if present
    pub name: Option<&'a str>,
    /// Function parameter names
    pub arguments: Vec<&'a str>,
    /// Function body
    pub statements: Vec<Statement<'a>>,
    /// The type of function
    pub ty: FunctionKind,
}

impl<'a> fmt::Display for FunctionDeclaration<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: ty

        write!(f, "function")?;

        if let Some(name) = self.name {
            write!(f, " {name}")?;
        }

        write!(f, "(")?;

        fmt_list(f, &self.arguments, ",")?;

        write!(f, ") {{")?;

        fmt_list(f, &self.statements, ";")?;

        Ok(())
    }
}

pub fn fmt_list<'a, D>(f: &mut fmt::Formatter<'a>, it: &[D], delim: &str) -> fmt::Result
where
    D: fmt::Display,
{
    for (i, expr) in it.iter().enumerate() {
        if i > 0 {
            write!(f, "{delim} ")?;
        }
        write!(f, "{}", expr)?;
    }

    Ok(())
}

impl<'a> FunctionDeclaration<'a> {
    /// Creates a new function declaration
    pub fn new(
        name: Option<&'a str>,
        arguments: Vec<&'a str>,
        statements: Vec<Statement<'a>>,
        ty: FunctionKind,
    ) -> Self {
        Self {
            name,
            arguments,
            statements,
            ty,
        }
    }
}

/// A block statement, primarily used to enter a new scope
#[derive(Debug, Clone)]
pub struct BlockStatement<'a>(pub Vec<Statement<'a>>);

impl<'a> fmt::Display for BlockStatement<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;

        fmt_list(f, &self.0, ";")?;

        write!(f, "}}")?;

        Ok(())
    }
}

/// An if statement
#[derive(Debug, Clone)]
pub struct IfStatement<'a> {
    /// Condition of this if statement
    pub condition: Expr<'a>,
    /// Body of this if statement
    pub then: Box<Statement<'a>>,
    /// Branches (`else if`'s)
    ///
    /// Compiler hackery requires branches to be a RefCell.
    /// The Visitor trait does not give us a mutable reference to IfStatement,
    /// so we need to interior mutability to be able to mutate branches from within
    /// the compiler
    pub branches: RefCell<Vec<IfStatement<'a>>>,
    /// Last else branch that executes if no other branch matches, if present
    pub el: Option<Box<Statement<'a>>>,
}

impl<'a> fmt::Display for IfStatement<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "if ({}) {{ {} }} ", self.condition, self.then)?;

        let branches = self.branches.borrow();
        for IfStatement { condition, then, .. } in branches.iter() {
            write!(f, " else if ({condition}) {{ {then} }} ")?;
        }

        if let Some(el) = &self.el {
            write!(f, " else {{ {el} }}")?;
        }

        Ok(())
    }
}

impl<'a> IfStatement<'a> {
    /// Creates a new if statement
    pub fn new(
        condition: Expr<'a>,
        then: Statement<'a>,
        branches: Vec<IfStatement<'a>>,
        el: Option<Box<Statement<'a>>>,
    ) -> Self {
        Self {
            condition,
            then: Box::new(then),
            branches: RefCell::new(branches),
            el,
        }
    }
}

/// The type of a variable declaration
#[derive(Debug, Copy, Clone, Display)]
pub enum VariableDeclarationKind {
    /// Var: lifetime extends to function scope
    #[display(fmt = "var")]
    Var,

    /// Let: lifetime limited to block scope
    #[display(fmt = "let")]
    Let,

    /// Const: lifetime limited to block scope and no reassigns allowed
    #[display(fmt = "const")]
    Const,

    /// Unnameable variables cannot be referred to by JavaScript code directly and are created by the compiler
    #[display(fmt = "__intrinsic_var")]
    Unnameable,
}

impl From<TokenType> for VariableDeclarationKind {
    fn from(tok: TokenType) -> Self {
        match tok {
            TokenType::Let => VariableDeclarationKind::Let,
            TokenType::Const => VariableDeclarationKind::Const,
            TokenType::Var => VariableDeclarationKind::Var,
            _ => unreachable!(),
        }
    }
}

/// A variable binding
#[derive(Debug, Clone, Display)]
#[display(fmt = "{} {}", kind, name)]
pub struct VariableBinding<'a> {
    /// The name/identifier of this variable
    pub name: &'a str,
    /// The type of this variable
    pub kind: VariableDeclarationKind,
}

/// A variable declaration
#[derive(Debug, Clone)]
pub struct VariableDeclaration<'a> {
    /// Variable bindings
    pub binding: VariableBinding<'a>,
    /// The value of this variable, if it was initialized
    pub value: Option<Expr<'a>>,
}

impl<'a> fmt::Display for VariableDeclaration<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.binding)?;

        if let Some(value) = &self.value {
            write!(f, " = {}", value)?;
        }

        write!(f, ";")
    }
}

impl<'a> VariableDeclaration<'a> {
    /// Creates a new variable declaration
    pub fn new(binding: VariableBinding<'a>, value: Option<Expr<'a>>) -> Self {
        Self { binding, value }
    }
}

/// A JavaScript class
#[derive(Debug, Clone)]
pub struct Class<'a> {
    /// The name of this class, if present
    ///
    /// Class expressions don't necessarily need to have a name
    pub name: Option<&'a str>,
    /// The superclass of this class, if present
    pub extends: Option<Expr<'a>>,
    /// Members of this class
    pub members: Vec<ClassMember<'a>>,
}

impl<'a> fmt::Display for Class<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "class {}", self.name.unwrap_or_default())?;

        if let Some(extends) = &self.extends {
            write!(f, " extends {}", extends)?;
        }

        write!(f, " {{\n")?;

        fmt_list(f, &self.members, ";")?;

        write!(f, "}}")
    }
}

impl<'a> Class<'a> {
    /// Returns a reference to the constructor, if present
    pub fn constructor(&self) -> Option<&FunctionDeclaration<'a>> {
        self.members.iter().find_map(|cm| cm.as_constructor())
    }
}

/// A JavaScript class member
#[derive(Debug, Clone)]
pub struct ClassMember<'a> {
    /// Whether this class member is declared as static
    pub static_: bool,
    /// Whether this class member is declared as private
    pub private: bool,
    /// The type of class member
    pub kind: ClassMemberKind<'a>,
}

impl<'a> fmt::Display for ClassMember<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.static_ {
            write!(f, "static ")?;
        }

        if self.private {
            write!(f, "private ")?;
        }

        write!(f, "{}", self.kind)
    }
}

impl<'a> ClassMember<'a> {
    /// Returns the inner function if this member is the constructor
    pub fn as_constructor(&self) -> Option<&FunctionDeclaration<'a>> {
        // Constructor cannot be private or static
        if self.private || self.static_ {
            return None;
        }

        match &self.kind {
            ClassMemberKind::Method(m) if m.name == Some("constructor") => Some(m),
            _ => None,
        }
    }

    /// Returns the identifier of this class member
    pub fn name(&self) -> &'a str {
        match &self.kind {
            ClassMemberKind::Property(p) => p.name,
            // Methods *always* have names, so unwrapping is OK here
            ClassMemberKind::Method(m) => m.name.unwrap(),
        }
    }
}

/// The type of class member
#[derive(Debug, Clone, Display)]
pub enum ClassMemberKind<'a> {
    /// A class method
    Method(FunctionDeclaration<'a>),
    /// A class property
    Property(ClassProperty<'a>),
}

/// A class property
#[derive(Debug, Clone)]
pub struct ClassProperty<'a> {
    /// The name of this property
    pub name: &'a str,
    /// The default value of this property, set when its constructor is called
    pub value: Option<Expr<'a>>,
}

impl<'a> fmt::Display for ClassProperty<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;

        if let Some(value) = &self.value {
            write!(f, " = {}", value)?;
        }

        write!(f, ";")
    }
}
