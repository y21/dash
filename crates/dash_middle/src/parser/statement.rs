use std::fmt::{self, Write};

use derive_more::Display;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::interner::{Symbol, sym};
use crate::lexer::token::TokenType;
use crate::sourcemap::Span;
use crate::tree::TreeToken;

use super::expr::{Expr, ExprKind};
use super::types::TypeSegment;

/// A JavaScript statement
#[derive(Debug, Clone, Display)]
pub enum StatementKind {
    /// Expression statement
    // TODO: this could _technically_ be just ExprKind since the span is the exact same,
    // but we wouldn't really save on anything because the enum is big either way
    #[display("{_0};")]
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
    #[display("continue;")]
    Continue(Option<Symbol>),
    /// Break loop statement
    #[display("break;")]
    Break(Option<Symbol>),
    /// Debugger statement
    #[display("debugger;")]
    Debugger,
    /// A labelled statement:
    ///
    ///     foo: { break foo; }
    ///
    /// is represented as Labelled(foo, Expr(Block(Break(foo))))
    #[display("{_0}: {_1}")]
    Labelled(Symbol, Box<Statement>),
    /// An empty statement
    ///
    /// This is impossible to occur in JavaScript code, however a statement may be folded to an empty statement
    /// if it does not have any side effects.
    #[display(";")]
    Empty,
}

#[derive(Debug, Clone, Display)]
#[display("{kind}")]
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
#[derive(Debug, Clone, Display)]
pub enum SpecifierKind {
    /// A raw identifier
    #[display("{_0}")]
    Ident(Binding),
}

/// Type of import statement
#[derive(Debug, Clone, Display)]
pub enum ImportKind {
    #[display("import({_0})")]
    Dynamic(Expr),
    /// import foo from "bar"
    #[display("import {_0} from \"{_1}\"")]
    DefaultAs(SpecifierKind, Symbol),
    /// import * as foo from "bar"
    #[display("import * as {_0} from \"{_1}\"")]
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

impl fmt::Display for ExportKind {
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
                fmt_list(f, &nv.0, ",")?;
                write!(f, " }}")
            }
        }
    }
}

impl ImportKind {
    /// Attempts to return the underlying [SpecifierKind], if present
    pub fn get_specifier(&self) -> Option<&SpecifierKind> {
        match self {
            Self::Dynamic(_) => None,
            Self::DefaultAs(s, _) => Some(s),
            Self::AllAs(s, _) => Some(s),
        }
    }
}

/// A catch statement
#[derive(Debug, Clone, Display)]
#[display("catch ({}) {{ {} }}", binding.map(|b| b.ident).unwrap_or(sym::empty), body)]
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

impl fmt::Display for TryCatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "try {{ {} }} ", self.try_)?;

        if let Some(catch) = &self.catch {
            write!(f, "{catch}")?;
        }

        if let Some(finally) = &self.finally {
            write!(f, " finally {{ {finally} }}")?;
        }

        Ok(())
    }
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
#[derive(Debug, Clone, Display)]
#[display("return {_0}")]
pub struct ReturnStatement(pub Expr);

/// A loop statement
#[derive(Debug, Clone, Display)]
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

#[derive(Debug, Clone, Display)]
#[display("do {body} while ({condition})")]
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
#[derive(Debug, Clone, Display)]
#[display("for ({binding} of {expr}) {{ {body} }}")]
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

impl fmt::Display for ForLoop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "for(")?;

        if let Some(init) = &self.init {
            write!(f, "{init}")?;
        }

        write!(f, ";")?;

        if let Some(condition) = &self.condition {
            write!(f, "{condition}")?;
        }

        write!(f, ";")?;

        if let Some(finalizer) = &self.finalizer {
            write!(f, "{finalizer}")?;
        }

        write!(f, ") {{ {} }}", self.body)
    }
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
#[derive(Debug, Clone, Display)]
#[display("for ({binding} in {expr}) {{ {body} }}")]
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
#[derive(Debug, Clone, Display)]
#[display("while ({condition}) {{ {body} }}")]
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

#[derive(Debug, Clone, Copy, Display, PartialEq)]
#[display("{ident}")]
pub struct Binding {
    pub ident: Symbol,
    pub id: LocalId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct LocalId(pub usize);

impl From<usize> for LocalId {
    fn from(value: usize) -> Self {
        LocalId(value)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ScopeId(pub usize);

impl ScopeId {
    /// The root function of
    pub const ROOT: ScopeId = ScopeId(0);
}

impl From<usize> for ScopeId {
    fn from(value: usize) -> Self {
        ScopeId(value)
    }
}

impl From<ScopeId> for usize {
    fn from(val: ScopeId) -> Self {
        val.0
    }
}

impl From<ScopeId> for TreeToken {
    fn from(value: ScopeId) -> Self {
        Self::new(value.0)
    }
}

impl From<TreeToken> for ScopeId {
    fn from(value: TreeToken) -> Self {
        Self(value.into())
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
    pub id: ScopeId,
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
    /// If this function is a desugared class constructor,
    /// then this contains all the instance members that need to be initialized.
    pub constructor_initializers: Option<Vec<ClassMember>>,
    /// Whether this function is a desugared class constructor and the class has an `extends` clause
    pub has_extends_clause: bool,
}

impl fmt::Display for FunctionDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: ty

        write!(f, "function")?;

        if let Some(name) = self.name {
            write!(f, " {name}")?;
        }

        write!(f, "(")?;

        for (id, (param, default, ty)) in self.parameters.iter().enumerate() {
            if id > 0 {
                write!(f, ",")?;
            }

            write!(f, "{param}")?;

            if let Some(ty) = ty {
                write!(f, ": {ty}")?;
            }

            if let Some(default) = default {
                write!(f, " = {default}")?;
            }
        }

        writeln!(f, ") {{")?;

        fmt_list(f, &self.statements, "\n")?;

        write!(f, "\n}}")?;

        Ok(())
    }
}

pub fn fmt_list<D>(f: &mut fmt::Formatter<'_>, it: &[D], delim: &str) -> fmt::Result
where
    D: fmt::Display,
{
    for (i, expr) in it.iter().enumerate() {
        if i > 0 {
            write!(f, "{delim} ")?;
        }
        write!(f, "{expr}")?;
    }

    Ok(())
}

/// A block statement, primarily used to enter a new scope
#[derive(Debug, Clone)]
pub struct BlockStatement(pub Vec<Statement>, pub ScopeId);

impl fmt::Display for BlockStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{{")?;

        fmt_list(f, &self.0, "\n")?;

        write!(f, "\n}}")?;

        Ok(())
    }
}

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

impl fmt::Display for IfStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "if ({}) {{ {} }} ", self.condition, self.then)?;

        for IfStatement { condition, then, .. } in &self.branches {
            write!(f, " else if ({condition}) {{ {then} }} ")?;
        }

        if let Some(el) = &self.el {
            write!(f, " else {{ {el} }}")?;
        }

        Ok(())
    }
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

impl fmt::Display for SwitchStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "switch ({}) {{", self.expr)?;

        for case in self.cases.iter() {
            writeln!(f, "case {}:", case.value)?;

            fmt_list(f, &case.body, "\n")?;

            writeln!(f)?;
        }

        if let Some(default) = &self.default {
            writeln!(f, "default:")?;
            fmt_list(f, default, "\n")?;
            writeln!(f)?;
        }

        write!(f, "}}")
    }
}

#[derive(Debug, Clone)]
pub struct SwitchCase {
    pub value: Expr,
    pub body: Vec<Statement>,
}

/// The type of a variable declaration
#[derive(Debug, Clone, Copy, Display, PartialEq, Eq, PartialOrd, Ord)]
pub enum VariableDeclarationKind {
    /// Var: lifetime extends to function scope
    #[display("var")]
    Var,

    /// Let: lifetime limited to block scope
    #[display("let")]
    Let,

    /// Const: lifetime limited to block scope and no reassigns allowed
    #[display("const")]
    Const,

    /// Unnameable variables cannot be referred to by JavaScript code directly and are created by the compiler
    ///
    /// Multiple unnamed variables of the same name can exist in a function's `locals` vec and all `find` operations
    /// on the `ScopeGraph` will never consider unnamed variables.
    #[display("__intrinsic_var")]
    Unnameable,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    /// Object destructuring: { a } = { a: 1 }
    Object {
        /// Fields to destructure
        ///
        /// Destructured fields can also be aliased with ` { a: b } = { a: 3 }` and have default values
        fields: Vec<(LocalId, Symbol, Option<Symbol>, Option<Expr>)>,
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

#[derive(Debug, Clone, Display)]
pub enum VariableDeclarationName {
    /// Normal identifier
    Identifier(Binding),
    Pattern(Pattern),
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Pattern::Object { fields, rest } => {
                write!(f, "{{ ")?;

                for (i, (_, name, alias, default)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{name}")?;
                    if let Some(alias) = alias {
                        write!(f, ": {alias}")?;
                    }
                    if let Some(default) = default {
                        write!(f, " = {default}")?;
                    }
                }

                if let Some(rest) = rest {
                    write!(f, ", ...{rest}")?;
                }

                write!(f, " }}")
            }
            Pattern::Array { fields, rest } => {
                write!(f, "[ ")?;

                for (i, name) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    match name {
                        Some((name, default)) => {
                            write!(f, "{name}")?;
                            if let Some(default) = default {
                                write!(f, " = {default}")?;
                            }
                        }
                        None => f.write_char(',')?,
                    }
                }

                if let Some(rest) = rest {
                    write!(f, ", ...{rest}")?;
                }

                write!(f, " ]")
            }
        }
    }
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
#[derive(Debug, Clone, Display)]
#[display("{kind} {name}")]
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

impl fmt::Display for VariableDeclarations {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_list(f, &self.0, ", ")
    }
}

/// A variable declaration
#[derive(Debug, Clone)]
pub struct VariableDeclaration {
    /// Variable bindings
    pub binding: VariableBinding,
    /// The value of this variable, if it was initialized
    pub value: Option<Expr>,
}

impl fmt::Display for VariableDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.binding)?;

        if let Some(value) = &self.value {
            write!(f, " = {value}")?;
        }

        write!(f, ";")
    }
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

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "class {}", self.name.map(|b| b.ident).unwrap_or(sym::empty))?;

        if let Some(extends) = &self.extends {
            write!(f, " extends {extends}")?;
        }

        writeln!(f, " {{")?;

        fmt_list(f, &self.members, ";")?;

        write!(f, "}}")
    }
}

impl Class {
    /// Returns the constructor, if present
    pub fn constructor(&self) -> Option<FunctionDeclaration> {
        self.members.iter().find_map(|cm| cm.as_constructor()).cloned()
    }
}

#[derive(Debug, Clone)]
pub enum ClassMemberKey {
    /// [Key] = Value
    Computed(Expr),
    /// Key = Value
    Named(Symbol),
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

impl fmt::Display for ClassMember {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.static_ {
            write!(f, "static ")?;
        }

        if self.private {
            write!(f, "private ")?;
        }

        match &self.key {
            ClassMemberKey::Computed(c) => write!(f, "[{c}]")?,
            ClassMemberKey::Named(n) => write!(f, "{n}")?,
        }

        match &self.value {
            ClassMemberValue::Method(method) => write!(f, "{method}"),
            ClassMemberValue::Field(Some(field)) => write!(f, "= {field};"),
            ClassMemberValue::Field(None) => f.write_char(';'),
            ClassMemberValue::Getter(method) => write!(f, "get {method}"),
            ClassMemberValue::Setter(method) => write!(f, "set {method}"),
        }
    }
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
#[derive(Debug, Clone, Display)]
pub enum Parameter {
    Identifier(Binding),
    SpreadIdentifier(Binding),
    #[display("{_1}")]
    Pattern(LocalId, Pattern),
    #[display("{_1}")]
    SpreadPattern(LocalId, Pattern),
}
