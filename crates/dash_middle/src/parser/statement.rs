#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::index_type;
use crate::interner::{Symbol, sym};
use crate::lexer::token::TokenType;
use crate::sourcemap::Span;

use super::expr::{Expr, ExprKind};
use super::types::TypeSegment;

/// A JavaScript statement
#[derive(Debug, Clone)]
pub enum StatementKind {
    /// Expression statement
    // TODO: this could _technically_ be just ExprKind since the span is the exact same,
    // but we wouldn't really save on anything because the enum is big either way
    Expression(Expr),
    /// Variable declaration
    Variable(VariableDeclarations),
    /// If statement
    If(IfStatement),
    /// Block statement
    Block(BlockStatement),
    /// Function declaration
    Function(FunctionDeclaration),
    /// Any loop
    Loop(Loop),
    /// Return statement
    Return(ReturnStatement),
    /// Try catch block
    Try(TryCatch),
    /// Throw statement
    Throw(Expr),
    /// Import statement
    Import(ImportKind),
    /// Export statement
    Export(ExportKind),
    /// Class declaration
    Class(Class),
    /// A switch statement
    Switch(SwitchStatement),
    /// Continue loop statement
    Continue(Option<Symbol>),
    /// Break loop statement
    Break(Option<Symbol>),
    /// Debugger statement
    Debugger,
    /// A labelled statement:
    /// ```js
    /// foo: { break foo; }
    /// ```
    ///
    /// is represented as Labelled(foo, Expr(Block(Break(foo))))
    Labelled(Symbol, Box<Statement>),
    /// An empty statement
    ///
    /// This is impossible to occur in JavaScript code, however a statement may be folded to an empty statement
    /// if it does not have any side effects.
    Empty,
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Span,
}

impl Statement {
    /// Creates a dummy empty statement.
    /// This is usually used when const eval'ing a statement to nothing but still needing to have some kind of statement.
    /// NOTE: this should not end up in diagnostics!
    pub fn dummy_empty() -> Self {
        Statement {
            kind: StatementKind::Empty,
            span: Span::COMPILER_GENERATED,
        }
    }

    /// Creates a dummy return statement.
    /// NOTE: this should not end up in diagnostics!
    pub fn dummy_return() -> Self {
        Statement {
            kind: StatementKind::Return(ReturnStatement(Expr {
                kind: ExprKind::undefined_literal(),
                span: Span::COMPILER_GENERATED,
            })),
            span: Span::COMPILER_GENERATED,
        }
    }
}

impl StatementKind {
    pub fn enters_scope(&self) -> bool {
        matches!(
            self,
            StatementKind::Block(_)
                | StatementKind::Function(_)
                | StatementKind::Loop(_)
                | StatementKind::Try(_)
                | StatementKind::Class(_)
        )
    }
}

/// The type of a specifier
///
/// This is used in import/export statements, as well as variable declaration
/// in the future. When destructuring is implemented, this enum will make more sense.
#[derive(Debug, Clone)]
pub enum SpecifierKind {
    /// A raw identifier
    Ident(Binding),
}

/// Type of import statement
#[derive(Debug, Clone)]
pub enum ImportKind {
    /// import foo from "bar"
    DefaultAs(SpecifierKind, Symbol),
    /// import * as foo from "bar"
    AllAs(SpecifierKind, Symbol),
}

/// Type of export statement
#[derive(Debug, Clone)]
pub enum ExportKind {
    /// export default foo
    Default(Expr),
    /// export { foo, bar }
    Named(Vec<Symbol>),
    /// export let foo = "bar"
    NamedVar(VariableDeclarations),
}

impl ImportKind {
    /// Attempts to return the underlying [SpecifierKind], if present
    pub fn get_specifier(&self) -> Option<&SpecifierKind> {
        match self {
            Self::DefaultAs(s, _) => Some(s),
            Self::AllAs(s, _) => Some(s),
        }
    }
}

/// A catch statement
#[derive(Debug, Clone)]
pub struct Catch {
    pub body_span: Span,
    /// The body of a catch statement
    pub body: BlockStatement,
    /// The identifier of the variable that receives the thrown error
    pub binding: Option<Binding>,
}

impl Catch {
    /// Creates a new catch statement
    pub fn new(body_span: Span, body: BlockStatement, binding: Option<Binding>) -> Self {
        Self {
            body_span,
            body,
            binding,
        }
    }
}

/// A try catch statement
#[derive(Debug, Clone)]
pub struct TryCatch {
    /// The body of the try statement
    pub try_: Box<Statement>,
    /// Catch statement
    // TODO: make this optional. a try can exist without catch (try finally)
    pub catch: Option<Catch>,
    /// Optional finally block
    pub finally: Option<Box<Statement>>,
}

impl TryCatch {
    /// Creates a new try catch block
    pub fn new(try_: Statement, catch: Option<Catch>, finally: Option<Statement>) -> Self {
        Self {
            try_: Box::new(try_),
            catch,
            finally: finally.map(Box::new),
        }
    }
}

/// A return statement
#[derive(Debug, Clone)]
pub struct ReturnStatement(pub Expr);

/// A loop statement
#[derive(Debug, Clone)]
pub enum Loop {
    /// A for loop
    For(ForLoop),
    /// A for..of loop
    ForOf(ForOfLoop),
    /// A for..in loop
    ForIn(ForInLoop),
    /// A while loop
    While(WhileLoop),
    /// A do..whiel loop
    DoWhile(DoWhileLoop),
}

impl From<ForLoop> for Loop {
    fn from(f: ForLoop) -> Self {
        Self::For(f)
    }
}

impl From<WhileLoop> for Loop {
    fn from(f: WhileLoop) -> Self {
        Self::While(f)
    }
}

impl From<DoWhileLoop> for Loop {
    fn from(f: DoWhileLoop) -> Self {
        Self::DoWhile(f)
    }
}

#[derive(Debug, Clone)]
pub struct DoWhileLoop {
    pub body: Box<Statement>,
    pub condition: Expr,
}

impl DoWhileLoop {
    /// Creates a new do..while loop
    pub fn new(condition: Expr, body: Statement) -> Self {
        Self {
            condition,
            body: Box::new(body),
        }
    }
}

/// A for..of loop
#[derive(Debug, Clone)]
pub struct ForOfLoop {
    /// The binding of this loop
    pub binding: VariableBinding,
    /// The expression to iterate over
    pub expr: Expr,
    /// The body of this loop
    pub body: Box<Statement>,
    pub scope: ScopeId,
}

/// A for loop
#[derive(Debug, Clone)]
pub struct ForLoop {
    /// The initializer of a for loop
    pub init: Option<Box<Statement>>,
    /// The condition that is used to determine when iteration should stop
    pub condition: Option<Expr>,
    /// Final expression, evaluated after each iteration
    pub finalizer: Option<Expr>,
    /// The body of a for loop
    pub body: Box<Statement>,
    pub scope: ScopeId,
}

impl ForLoop {
    /// Creates a new for loop
    pub fn new(
        init: Option<Statement>,
        condition: Option<Expr>,
        finalizer: Option<Expr>,
        body: Statement,
        scope: ScopeId,
    ) -> Self {
        Self {
            init: init.map(Box::new),
            condition,
            finalizer,
            body: Box::new(body),
            scope,
        }
    }
}

/// A for..in loop
#[derive(Debug, Clone)]
pub struct ForInLoop {
    /// The binding of this loop
    pub binding: VariableBinding,
    /// The expression to iterate over
    pub expr: Expr,
    /// The body of this loop
    pub body: Box<Statement>,
    pub scope: ScopeId,
}

/// A while loop
#[derive(Debug, Clone)]
pub struct WhileLoop {
    /// The condition of this while loop, used to determine when to stop iterating
    pub condition: Expr,
    /// The body of this while loop
    pub body: Box<Statement>,
}

impl WhileLoop {
    /// Creates a new while loop
    pub fn new(condition: Expr, body: Statement) -> Self {
        Self {
            condition,
            body: Box::new(body),
        }
    }
}

/// The type of function
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FunctionKind {
    /// A normal function
    Function(Asyncness),
    /// A generator function
    Generator,
    /// An arrow function
    Arrow,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Binding {
    pub ident: Symbol,
    pub id: FrontLocalId,
}

index_type! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[repr(transparent)]
    pub struct FrontLocalId(u32);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[repr(transparent)]
    pub struct ScopeId(pub u32);
}

impl From<usize> for FrontLocalId {
    fn from(value: usize) -> Self {
        FrontLocalId(value as u32)
    }
}

impl ScopeId {
    /// The root function of
    pub const ROOT: ScopeId = ScopeId(0);
}

impl From<usize> for ScopeId {
    fn from(value: usize) -> Self {
        ScopeId(value as u32)
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Asyncness {
    Yes,
    No,
}

impl From<bool> for Asyncness {
    fn from(value: bool) -> Self {
        match value {
            true => Asyncness::Yes,
            false => Asyncness::No,
        }
    }
}

/// A function declaration
#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    /// The scope of the body of the function
    pub body_scope: ScopeId,
    /// The scope of the parameters of the function
    pub parameters_scope: ScopeId,
    /// The name of this function, if present
    pub name: Option<Binding>,
    /// Function parameter names
    pub parameters: Vec<(
        // Parameter
        Parameter,
        // Default value
        Option<Expr>,
        // Type segment
        Option<TypeSegment>,
    )>,
    /// Function body
    pub statements: Vec<Statement>,
    /// The type of function
    pub ty: FunctionKind,
    pub ty_segment: Option<TypeSegment>,
}

/// A block statement, primarily used to enter a new scope
#[derive(Debug, Clone)]
pub struct BlockStatement(pub Vec<Statement>, pub ScopeId);

/// An if statement
#[derive(Debug, Clone)]
pub struct IfStatement {
    /// Condition of this if statement
    pub condition: Expr,
    /// Body of this if statement
    pub then: Box<Statement>,
    /// Branches (`else if`'s)
    pub branches: Vec<IfStatement>,
    /// Last else branch that executes if no other branch matches, if present
    pub el: Option<Box<Statement>>,
}

impl IfStatement {
    /// Creates a new if statement
    pub fn new(condition: Expr, then: Statement, branches: Vec<IfStatement>, el: Option<Box<Statement>>) -> Self {
        Self {
            condition,
            then: Box::new(then),
            branches,
            el,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SwitchStatement {
    pub expr: Expr,
    pub cases: Vec<SwitchCase>,
    pub default: Option<Vec<Statement>>,
}

#[derive(Debug, Clone)]
pub struct SwitchCase {
    pub value: Expr,
    pub body: Vec<Statement>,
}

/// The type of a variable declaration
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VariableDeclarationKind {
    /// Var: lifetime extends to function scope
    Var,

    /// Let: lifetime limited to block scope
    Let,

    /// Const: lifetime limited to block scope and no reassigns allowed
    Const,

    /// Unnameable variables cannot be referred to by JavaScript code directly and are created by the compiler
    ///
    /// Multiple unnamed variables of the same name can exist in a function's `locals` vec and all `find` operations
    /// on the `ScopeGraph` will never consider unnamed variables.
    Unnameable,

    /// Implicit "arguments" object
    Arguments,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    /// Object destructuring: { a } = { a: 1 }
    Object {
        /// Fields to destructure
        ///
        /// Destructured fields can also be aliased with ` { a: b } = { a: 3 }` and have default values
        fields: Vec<(FrontLocalId, Symbol, Option<Symbol>, Option<Expr>)>,
        /// The rest element, if present
        rest: Option<Binding>,
    },
    /// Array destructuring: [ a ] = [ 1 ]
    Array {
        /// Elements to destructure.
        /// For `[a,,b = default]` this stores `[Some((a, None)), None, Some((b, Some(default)))]`
        fields: Vec<Option<(Binding, Option<Expr>)>>,
        /// The rest element, if present
        rest: Option<Binding>,
    },
}

#[derive(Debug, Clone)]
pub enum VariableDeclarationName {
    /// Normal identifier
    Identifier(Binding),
    Pattern(Pattern),
}

impl VariableDeclarationKind {
    pub fn is_nameable(&self) -> bool {
        !matches!(self, VariableDeclarationKind::Unnameable)
    }
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
#[derive(Debug, Clone)]
pub struct VariableBinding {
    /// The name/identifier of this variable
    pub name: VariableDeclarationName,
    /// The type of this variable
    pub kind: VariableDeclarationKind,
    /// The type of a variable, if present
    pub ty: Option<TypeSegment>,
}

#[derive(Debug, Clone)]
pub struct VariableDeclarations(pub Vec<VariableDeclaration>);

/// A variable declaration
#[derive(Debug, Clone)]
pub struct VariableDeclaration {
    /// Variable bindings
    pub binding: VariableBinding,
    /// The value of this variable, if it was initialized
    pub value: Option<Expr>,
}

impl VariableDeclaration {
    /// Creates a new variable declaration
    pub fn new(binding: VariableBinding, value: Option<Expr>) -> Self {
        Self { binding, value }
    }
}

/// A JavaScript class
#[derive(Debug, Clone)]
pub struct Class {
    /// The name of this class, if present
    ///
    /// Class expressions don't necessarily need to have a name
    pub name: Option<Binding>,
    /// The superclass of this class, if present
    pub extends: Option<Box<Expr>>,
    /// Members of this class
    pub members: Vec<ClassMember>,
}

impl Class {
    /// Returns and removes the constructor, if present
    pub fn take_constructor(&mut self) -> Option<FunctionDeclaration> {
        self.members
            .iter()
            .position(|cm| cm.as_constructor().is_some())
            .map(|idx| {
                let member = self.members.remove(idx);

                if let ClassMemberValue::Method(m) = member.value {
                    m
                } else {
                    unreachable!()
                }
            })
    }
}

#[derive(Debug, Clone)]
pub enum ClassMemberKey {
    /// [Key] = Value
    Computed(Expr),
    /// Key = Value
    Named(Symbol, Span),
}

/// A JavaScript class member
#[derive(Debug, Clone)]
pub struct ClassMember {
    /// Whether this class member is declared as static
    pub static_: bool,
    /// Whether this class member is declared as private
    pub private: bool,
    pub key: ClassMemberKey,
    /// The type of class member
    pub value: ClassMemberValue,
}

impl ClassMember {
    /// Returns the inner function if this member is the constructor
    pub fn as_constructor(&self) -> Option<&FunctionDeclaration> {
        // Constructor cannot be private or static
        if self.private || self.static_ {
            return None;
        }

        match &self.value {
            ClassMemberValue::Method(m) if m.name.is_some_and(|b| b.ident == sym::constructor) => Some(m),
            _ => None,
        }
    }
}

/// The value of a class member
#[derive(Debug, Clone)]
pub enum ClassMemberValue {
    /// A class method
    Method(FunctionDeclaration),
    /// A class field.
    /// The value can be `None` for `class V { Key; }`
    Field(Option<Expr>),
    Getter(FunctionDeclaration),
    Setter(FunctionDeclaration),
}
/// A function parameter
#[derive(Debug, Clone)]
pub enum Parameter {
    Identifier(Binding),
    SpreadIdentifier(Binding),
    Pattern(FrontLocalId, Pattern),
    SpreadPattern(FrontLocalId, Pattern),
}
