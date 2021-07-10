use std::cell::RefCell;

use super::{expr::Expr, token::TokenType};

/// A JavaScript statement
#[derive(Debug, Clone)]
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
    /// While loop
    While(WhileLoop<'a>),
    /// For loop
    For(ForLoop<'a>),
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
    /// Continue loop statement
    Continue,
    /// Break loop statement
    Break,
    /// Debugger statement
    Debugger,
}

/// The type of a specifier
///
/// This is used in import/export statements, as well as variable declaration
/// in the future. When destructuring is implemented, this enum will make more sense.
#[derive(Debug, Clone)]
pub enum SpecifierKind<'a> {
    /// A raw identifier
    Ident(&'a [u8]),
}

impl<'a> SpecifierKind<'a> {
    /// Attempts to return self as an identifier
    pub fn as_ident(&self) -> Option<&'a [u8]> {
        match self {
            Self::Ident(i) => Some(i),
        }
    }
}

/// Type of import statement
#[derive(Debug, Clone)]
pub enum ImportKind<'a> {
    /// import("foo")
    Dynamic(Expr<'a>),
    /// import foo from "bar"
    DefaultAs(SpecifierKind<'a>, &'a [u8]),
    /// import * as foo from "bar"
    AllAs(SpecifierKind<'a>, &'a [u8]),
}

/// Type of export statement
#[derive(Debug, Clone)]
pub enum ExportKind<'a> {
    /// export default foo
    Default(Expr<'a>),
    /// export { foo, bar }
    Named(Vec<&'a [u8]>),
    /// export let foo = "bar"
    NamedVar(Vec<VariableDeclaration<'a>>),
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
    pub fn get_module_target(&self) -> Option<&'a [u8]> {
        match self {
            Self::Dynamic(_) => None,
            Self::DefaultAs(_, i) => Some(i),
            Self::AllAs(_, i) => Some(i),
        }
    }
}

/// A catch statement
#[derive(Debug, Clone)]
pub struct Catch<'a> {
    /// The body of a catch statement
    pub body: Box<Statement<'a>>,
    /// The identifier of the variable that receives the thrown error
    pub ident: Option<&'a [u8]>,
}

impl<'a> Catch<'a> {
    /// Creates a new catch statement
    pub fn new(body: Statement<'a>, ident: Option<&'a [u8]>) -> Self {
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
    pub catch: Catch<'a>,
    /// Optional finally block
    pub finally: Option<Box<Statement<'a>>>,
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
#[derive(Debug, Clone)]
pub struct ReturnStatement<'a>(pub Expr<'a>);

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
#[derive(Debug, Clone)]
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

/// A function declaration
#[derive(Debug, Clone)]
pub struct FunctionDeclaration<'a> {
    /// The name of this function, if present
    pub name: Option<&'a [u8]>,
    /// Function parameter names
    pub arguments: Vec<&'a [u8]>,
    /// Function body
    pub statements: Vec<Statement<'a>>,
    /// Whether this function is an arrow function
    pub arrow: bool,
}

impl<'a> FunctionDeclaration<'a> {
    /// Creates a new function declaration
    pub fn new(
        name: Option<&'a [u8]>,
        arguments: Vec<&'a [u8]>,
        statements: Vec<Statement<'a>>,
        arrow: bool,
    ) -> Self {
        Self {
            name,
            arguments,
            statements,
            arrow,
        }
    }
}

/// A block statement, primarily used to enter a new scope
#[derive(Debug, Clone)]
pub struct BlockStatement<'a>(pub Vec<Statement<'a>>);

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
#[derive(Debug, Copy, Clone)]
pub enum VariableDeclarationKind {
    /// Var: lifetime extends to function scope
    Var,
    /// Let: lifetime limited to block scope
    Let,
    /// Const: lifetime limited to block scope and no reassigns allowed
    Const,
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

/// A variable declaration
#[derive(Debug, Clone)]
pub struct VariableDeclaration<'a> {
    /// The name/identifier of this variable
    pub name: &'a [u8],
    /// The type of this variable
    pub kind: VariableDeclarationKind,
    /// The value of this variable, if it was initialized
    pub value: Option<Expr<'a>>,
}

impl<'a> VariableDeclaration<'a> {
    /// Creates a new variable declaration
    pub fn new(name: &'a [u8], kind: VariableDeclarationKind, value: Option<Expr<'a>>) -> Self {
        Self { name, kind, value }
    }
}
