use super::{expr::Expr, token::TokenType};

#[derive(Debug, Clone)]
pub enum Statement<'a> {
    Expression(Expr<'a>),
    Variable(VariableDeclaration<'a>),
    If(IfStatement<'a>),
    Block(BlockStatement<'a>),
    Function(FunctionDeclaration<'a>),
    While(WhileLoop<'a>),
    Return(ReturnStatement<'a>),
    Try(TryCatch<'a>),
    Throw(Expr<'a>),
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
    pub branches: Vec<IfStatement<'a>>,
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
            branches,
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
