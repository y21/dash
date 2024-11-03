use crate::interner::Symbol;
use crate::parser::expr::{
    ArrayLiteral, AssignmentExpr, BinaryExpr, ConditionalExpr, Expr, ExprKind, FunctionCall, GroupingExpr, LiteralExpr,
    ObjectLiteral, Postfix, Prefix, PropertyAccessExpr, Seq, UnaryExpr,
};
use crate::parser::statement::{
    BlockStatement, Class, DoWhileLoop, ExportKind, ForInLoop, ForLoop, ForOfLoop, FunctionDeclaration, IfStatement,
    ImportKind, Loop, ReturnStatement, Statement, StatementKind, SwitchStatement, TryCatch, VariableDeclarations,
    WhileLoop,
};
use crate::sourcemap::Span;

pub trait VisitorExt: Visitor<()> {
    fn accept(&mut self, e: StatementKind);
    fn accept_expr(&mut self, e: StatementKind);
}

/// A visitor trait that helps walking an AST
pub trait Visitor<V> {
    /// Accepts a parsed statement
    fn accept(&mut self, e: Statement) -> V;

    /// Accepts a parsed expression
    fn accept_expr(&mut self, e: Expr) -> V;

    /// Visits an expression statement
    fn visit_expression_statement(&mut self, e: Expr) -> V;

    /// Visits a binary expression
    fn visit_binary_expression(&mut self, span: Span, e: BinaryExpr) -> V;

    /// Visits a grouping expression
    fn visit_grouping_expression(&mut self, span: Span, e: GroupingExpr) -> V;

    /// Visits a literal expression
    fn visit_literal_expression(&mut self, span: Span, e: LiteralExpr) -> V;

    /// Visits an identifier
    fn visit_identifier_expression(&mut self, span: Span, i: Symbol) -> V;

    /// Visits an unary expression
    fn visit_unary_expression(&mut self, s: Span, e: UnaryExpr) -> V;

    /// Visits a variable declaration
    fn visit_variable_declaration(&mut self, span: Span, v: VariableDeclarations) -> V;

    /// Visits an if statement
    fn visit_if_statement(&mut self, span: Span, i: IfStatement) -> V;

    /// Visits a block statement
    fn visit_block_statement(&mut self, span: Span, b: BlockStatement) -> V;

    /// Visits a function declaration
    fn visit_function_declaration(&mut self, span: Span, f: FunctionDeclaration) -> V;

    /// Visits a while loop
    fn visit_while_loop(&mut self, span: Span, label: Option<Symbol>, l: WhileLoop) -> V;

    /// Visits a do while loop
    fn visit_do_while_loop(&mut self, span: Span, label: Option<Symbol>, d: DoWhileLoop) -> V;

    /// Visits an assignment expression
    fn visit_assignment_expression(&mut self, span: Span, e: AssignmentExpr) -> V;

    /// Visits a function call
    fn visit_function_call(&mut self, span: Span, c: FunctionCall) -> V;

    /// Visits a return statement
    fn visit_return_statement(&mut self, span: Span, s: ReturnStatement) -> V;

    /// Visits a conditional expression
    fn visit_conditional_expr(&mut self, span: Span, c: ConditionalExpr) -> V;

    /// Visits a property access expression
    ///
    /// This includes both computed access and static access
    fn visit_property_access_expr(&mut self, span: Span, e: PropertyAccessExpr, preserve_this: bool) -> V;

    /// Visits a sequence expression
    fn visit_sequence_expr(&mut self, span: Span, s: Seq) -> V;

    /// Visits any prefix expression
    fn visit_prefix_expr(&mut self, spna: Span, p: Prefix) -> V;

    /// Visits any postfix expression
    fn visit_postfix_expr(&mut self, span: Span, p: Postfix) -> V;

    /// Visits a function expression
    fn visit_function_expr(&mut self, span: Span, f: FunctionDeclaration) -> V;

    fn visit_class_expr(&mut self, span: Span, c: Class) -> V;

    /// Visits an array literal
    fn visit_array_literal(&mut self, span: Span, a: ArrayLiteral) -> V;

    /// Visits an object literal
    fn visit_object_literal(&mut self, span: Span, o: ObjectLiteral) -> V;

    /// Visits a try catch statement
    fn visit_try_catch(&mut self, span: Span, t: TryCatch) -> V;

    /// Visits a throw statement
    fn visit_throw(&mut self, span: Span, e: Expr) -> V;

    /// Visits a for loop
    fn visit_for_loop(&mut self, span: Span, label: Option<Symbol>, f: ForLoop) -> V;

    /// Visits a for..of loop
    fn visit_for_of_loop(&mut self, span: Span, label: Option<Symbol>, f: ForOfLoop) -> V;

    /// Visits a for..in loop
    fn visit_for_in_loop(&mut self, span: Span, label: Option<Symbol>, f: ForInLoop) -> V;

    /// Visits an import statement
    fn visit_import_statement(&mut self, span: Span, i: ImportKind) -> V;

    /// Visits an export statement
    fn visit_export_statement(&mut self, span: Span, e: ExportKind) -> V;

    /// Visits an empty statement
    fn visit_empty_statement(&mut self) -> V;

    /// Visits a break statement
    fn visit_break(&mut self, span: Span, sym: Option<Symbol>) -> V;

    /// Visits a continue statement
    fn visit_continue(&mut self, span: Span, sym: Option<Symbol>) -> V;

    /// Visits a debugger statement
    fn visit_debugger(&mut self, span: Span) -> V;

    /// Visits an empty expression
    fn visit_empty_expr(&mut self) -> V;

    /// Visits a class declaration
    fn visit_class_declaration(&mut self, span: Span, c: Class) -> V;

    /// Visits a switch statement
    fn visit_switch_statement(&mut self, span: Span, s: SwitchStatement) -> V;

    /// Visits a labelled statement.
    fn visit_labelled(&mut self, span: Span, label: Symbol, stmt: Box<Statement>) -> V;
}

pub fn accept_default<T, V: Visitor<T>>(this: &mut V, Statement { kind, span }: Statement) -> T {
    match kind {
        StatementKind::Expression(e) => this.visit_expression_statement(e),
        StatementKind::Variable(v) => this.visit_variable_declaration(span, v),
        StatementKind::If(i) => this.visit_if_statement(span, i),
        StatementKind::Block(b) => this.visit_block_statement(span, b),
        StatementKind::Function(f) => this.visit_function_declaration(span, f),
        StatementKind::Loop(Loop::For(f)) => this.visit_for_loop(span, None, f),
        StatementKind::Loop(Loop::While(w)) => this.visit_while_loop(span, None, w),
        StatementKind::Loop(Loop::ForOf(f)) => this.visit_for_of_loop(span, None, f),
        StatementKind::Loop(Loop::ForIn(f)) => this.visit_for_in_loop(span, None, f),
        StatementKind::Loop(Loop::DoWhile(d)) => this.visit_do_while_loop(span, None, d),
        StatementKind::Return(r) => this.visit_return_statement(span, r),
        StatementKind::Try(t) => this.visit_try_catch(span, t),
        StatementKind::Throw(t) => this.visit_throw(span, t),
        StatementKind::Import(i) => this.visit_import_statement(span, i),
        StatementKind::Export(e) => this.visit_export_statement(span, e),
        StatementKind::Class(c) => this.visit_class_declaration(span, c),
        StatementKind::Continue(sym) => this.visit_continue(span, sym),
        StatementKind::Break(sym) => this.visit_break(span, sym),
        StatementKind::Debugger => this.visit_debugger(span),
        StatementKind::Empty => this.visit_empty_statement(),
        StatementKind::Switch(s) => this.visit_switch_statement(span, s),
        StatementKind::Labelled(l, s) => this.visit_labelled(span, l, s),
    }
}

pub fn accept_expr_default<T, V: Visitor<T>, F>(this: &mut V, Expr { kind, span }: Expr, on_empty: F) -> T
where
    F: FnOnce(&mut V) -> T,
{
    match kind {
        ExprKind::Binary(e) => this.visit_binary_expression(span, e),
        ExprKind::Assignment(e) => this.visit_assignment_expression(span, e),
        ExprKind::Grouping(e) => this.visit_grouping_expression(span, e),
        ExprKind::Literal(LiteralExpr::Identifier(i)) => this.visit_identifier_expression(span, i),
        ExprKind::Literal(l) => this.visit_literal_expression(span, l),
        ExprKind::Unary(e) => this.visit_unary_expression(span, e),
        ExprKind::Call(e) => this.visit_function_call(span, e),
        ExprKind::Conditional(e) => this.visit_conditional_expr(span, e),
        ExprKind::PropertyAccess(e) => this.visit_property_access_expr(span, e, false),
        ExprKind::Sequence(e) => this.visit_sequence_expr(span, e),
        ExprKind::Postfix(e) => this.visit_postfix_expr(span, e),
        ExprKind::Prefix(e) => this.visit_prefix_expr(span, e),
        ExprKind::Function(e) => this.visit_function_expr(span, e),
        ExprKind::Class(e) => this.visit_class_expr(span, e),
        ExprKind::Array(e) => this.visit_array_literal(span, e),
        ExprKind::Object(e) => this.visit_object_literal(span, e),
        ExprKind::Compiled(..) => on_empty(this),
        ExprKind::Empty => this.visit_empty_expr(),
    }
}
