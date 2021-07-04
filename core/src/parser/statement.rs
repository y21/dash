use std::cell::RefCell;

use super::{expr::Expr, token::TokenType};

#[derive(Debug, Clone)]
pub enum Statement<'a> {
    Expression(Expr<'a>),
    Variable(VariableDeclaration<'a>),
    If(IfStatement<'a>),
    Block(BlockStatement<'a>),
    Function(FunctionDeclaration<'a>),
    While(WhileLoop<'a>),
    For(ForLoop<'a>),
    Return(ReturnStatement<'a>),
    Try(TryCatch<'a>),
    Throw(Expr<'a>),
    Import(ImportKind<'a>),
    Export(ExportKind<'a>),
    Continue,
    Break,
    Debugger,
}

#[derive(Debug, Clone)]
pub enum SpecifierKind<'a> {
    Ident(&'a [u8]),
}

impl<'a> SpecifierKind<'a> {
    pub fn as_ident(&self) -> Option<&'a [u8]> {
        match self {
            Self::Ident(i) => Some(i),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ImportKind<'a> {
    /// import("foo")
    Dynamic(Expr<'a>),
    /// import foo from "bar"
    DefaultAs(SpecifierKind<'a>, &'a [u8]),
    /// import * as foo from "bar"
    AllAs(SpecifierKind<'a>, &'a [u8]),
}

#[derive(Debug, Clone)]
pub enum ExportKind<'a> {
    /// export default foo
    Default(Expr<'a>),
    // export { foo, bar }
    Named(Vec<&'a [u8]>),
    // export let foo = "bar"
    NamedVar(Vec<VariableDeclaration<'a>>),
}

impl<'a> ImportKind<'a> {
    pub fn get_specifier(&self) -> Option<&SpecifierKind<'a>> {
        match self {
            Self::Dynamic(_) => None,
            Self::DefaultAs(s, _) => Some(s),
            Self::AllAs(s, _) => Some(s),
        }
    }

    pub fn get_module_target(&self) -> Option<&'a [u8]> {
        match self {
            Self::Dynamic(_) => None,
            Self::DefaultAs(_, i) => Some(i),
            Self::AllAs(_, i) => Some(i),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Catch<'a> {
    pub body: Box<Statement<'a>>,
    pub ident: Option<&'a [u8]>,
}

impl<'a> Catch<'a> {
    pub fn new(body: Statement<'a>, ident: Option<&'a [u8]>) -> Self {
        Self {
            body: Box::new(body),
            ident,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TryCatch<'a> {
    pub try_: Box<Statement<'a>>,
    pub catch: Catch<'a>,
    pub finally: Option<Box<Statement<'a>>>,
}

impl<'a> TryCatch<'a> {
    pub fn new(try_: Statement<'a>, catch: Catch<'a>, finally: Option<Statement<'a>>) -> Self {
        Self {
            try_: Box::new(try_),
            catch,
            finally: finally.map(Box::new),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReturnStatement<'a>(pub Expr<'a>);

#[derive(Debug, Clone)]
pub struct ForLoop<'a> {
    pub init: Option<Box<Statement<'a>>>,
    pub condition: Option<Expr<'a>>,
    pub finalizer: Option<Expr<'a>>,
    pub body: Box<Statement<'a>>,
}

impl<'a> ForLoop<'a> {
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

#[derive(Debug, Clone)]
pub struct WhileLoop<'a> {
    pub condition: Expr<'a>,
    pub body: Box<Statement<'a>>,
}

impl<'a> WhileLoop<'a> {
    pub fn new(condition: Expr<'a>, body: Statement<'a>) -> Self {
        Self {
            condition: condition,
            body: Box::new(body),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionDeclaration<'a> {
    pub name: Option<&'a [u8]>,
    pub arguments: Vec<&'a [u8]>,
    pub statements: Vec<Statement<'a>>,
}

impl<'a> FunctionDeclaration<'a> {
    pub fn new(
        name: Option<&'a [u8]>,
        arguments: Vec<&'a [u8]>,
        statements: Vec<Statement<'a>>,
    ) -> Self {
        Self {
            name,
            arguments,
            statements,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlockStatement<'a>(pub Vec<Statement<'a>>);

#[derive(Debug, Clone)]
pub struct IfStatement<'a> {
    pub condition: Expr<'a>,
    pub then: Box<Statement<'a>>,
    pub branches: RefCell<Vec<IfStatement<'a>>>, // Compiler hackery requires branches to be a RefCell
    pub el: Option<Box<Statement<'a>>>,
}

#[derive(Debug, Clone)]
pub enum IfBranch<'a> {
    If(IfStatement<'a>),
    Else(Statement<'a>),
}

impl<'a> IfStatement<'a> {
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

#[derive(Debug, Copy, Clone)]
pub enum VariableDeclarationKind {
    Var,
    Let,
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

#[derive(Debug, Clone)]
pub struct VariableDeclaration<'a> {
    pub name: &'a [u8],
    pub kind: VariableDeclarationKind,
    pub value: Option<Expr<'a>>,
}

impl<'a> VariableDeclaration<'a> {
    pub fn new(name: &'a [u8], kind: VariableDeclarationKind, value: Option<Expr<'a>>) -> Self {
        Self { name, kind, value }
    }
}
