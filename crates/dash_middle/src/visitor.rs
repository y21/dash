use crate::interner::Symbol;
use crate::parser::expr::ArrayLiteral;
use crate::parser::expr::AssignmentExpr;
use crate::parser::expr::BinaryExpr;
use crate::parser::expr::ConditionalExpr;
use crate::parser::expr::Expr;
use crate::parser::expr::ExprKind;
use crate::parser::expr::FunctionCall;
use crate::parser::expr::GroupingExpr;
use crate::parser::expr::LiteralExpr;
use crate::parser::expr::ObjectLiteral;
use crate::parser::expr::Postfix;
use crate::parser::expr::Prefix;
use crate::parser::expr::PropertyAccessExpr;
use crate::parser::expr::Seq;
use crate::parser::expr::UnaryExpr;
use crate::parser::statement::BlockStatement;
use crate::parser::statement::Class;
use crate::parser::statement::DoWhileLoop;
use crate::parser::statement::ExportKind;
use crate::parser::statement::ForInLoop;
use crate::parser::statement::ForLoop;
use crate::parser::statement::ForOfLoop;
use crate::parser::statement::FunctionDeclaration;
use crate::parser::statement::IfStatement;
use crate::parser::statement::ImportKind;
use crate::parser::statement::Loop;
use crate::parser::statement::ReturnStatement;
use crate::parser::statement::Statement;
use crate::parser::statement::StatementKind;
use crate::parser::statement::SwitchStatement;
use crate::parser::statement::TryCatch;
use crate::parser::statement::VariableDeclarations;
use crate::parser::statement::WhileLoop;

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
    fn visit_binary_expression(&mut self, e: BinaryExpr) -> V;

    /// Visits a grouping expression
    fn visit_grouping_expression(&mut self, e: GroupingExpr) -> V;

    /// Visits a literal expression
    fn visit_literal_expression(&mut self, e: LiteralExpr) -> V;

    /// Visits an identifier
    fn visit_identifier_expression(&mut self, i: Symbol) -> V;

    /// Visits an unary expression
    fn visit_unary_expression(&mut self, e: UnaryExpr) -> V;

    /// Visits a variable declaration
    fn visit_variable_declaration(&mut self, v: VariableDeclarations) -> V;

    /// Visits an if statement
    fn visit_if_statement(&mut self, i: IfStatement) -> V;

    /// Visits a block statement
    fn visit_block_statement(&mut self, b: BlockStatement) -> V;

    /// Visits a function declaration
    fn visit_function_declaration(&mut self, f: FunctionDeclaration) -> V;

    /// Visits a while loop
    fn visit_while_loop(&mut self, l: WhileLoop) -> V;

    /// Visits a do while loop
    fn visit_do_while_loop(&mut self, d: DoWhileLoop) -> V;

    /// Visits an assignment expression
    fn visit_assignment_expression(&mut self, e: AssignmentExpr) -> V;

    /// Visits a function call
    fn visit_function_call(&mut self, c: FunctionCall) -> V;

    /// Visits a return statement
    fn visit_return_statement(&mut self, s: ReturnStatement) -> V;

    /// Visits a conditional expression
    fn visit_conditional_expr(&mut self, c: ConditionalExpr) -> V;

    /// Visits a property access expression
    ///
    /// This includes both computed access and static access
    fn visit_property_access_expr(&mut self, e: PropertyAccessExpr, preserve_this: bool) -> V;

    /// Visits a sequence expression
    fn visit_sequence_expr(&mut self, s: Seq) -> V;

    /// Visits any prefix expression
    fn visit_prefix_expr(&mut self, p: Prefix) -> V;

    /// Visits any postfix expression
    fn visit_postfix_expr(&mut self, p: Postfix) -> V;

    /// Visits a function expression
    fn visit_function_expr(&mut self, f: FunctionDeclaration) -> V;

    /// Visits an array literal
    fn visit_array_literal(&mut self, a: ArrayLiteral) -> V;

    /// Visits an object literal
    fn visit_object_literal(&mut self, o: ObjectLiteral) -> V;

    /// Visits a try catch statement
    fn visit_try_catch(&mut self, t: TryCatch) -> V;

    /// Visits a throw statement
    fn visit_throw(&mut self, e: Expr) -> V;

    /// Visits a for loop
    fn visit_for_loop(&mut self, f: ForLoop) -> V;

    /// Visits a for..of loop
    fn visit_for_of_loop(&mut self, f: ForOfLoop) -> V;

    /// Visits a for..in loop
    fn visit_for_in_loop(&mut self, f: ForInLoop) -> V;

    /// Visits an import statement
    fn visit_import_statement(&mut self, i: ImportKind) -> V;

    /// Visits an export statement
    fn visit_export_statement(&mut self, e: ExportKind) -> V;

    /// Visits an empty statement
    fn visit_empty_statement(&mut self) -> V;

    /// Visits a break statement
    fn visit_break(&mut self) -> V;

    /// Visits a continue statement
    fn visit_continue(&mut self) -> V;

    /// Visits a debugger statement
    fn visit_debugger(&mut self) -> V;

    /// Visits an empty expression
    fn visit_empty_expr(&mut self) -> V;

    /// Visits a class declaration
    fn visit_class_declaration(&mut self, c: Class) -> V;

    /// Visits a switch statement
    fn visit_switch_statement(&mut self, s: SwitchStatement) -> V;
}

pub fn accept_default<T, V: Visitor<T>>(this: &mut V, s: StatementKind) -> T {
    match s {
        StatementKind::Expression(e) => this.visit_expression_statement(e),
        StatementKind::Variable(v) => this.visit_variable_declaration(v),
        StatementKind::If(i) => this.visit_if_statement(i),
        StatementKind::Block(b) => this.visit_block_statement(b),
        StatementKind::Function(f) => this.visit_function_declaration(f),
        StatementKind::Loop(Loop::For(f)) => this.visit_for_loop(f),
        StatementKind::Loop(Loop::While(w)) => this.visit_while_loop(w),
        StatementKind::Loop(Loop::ForOf(f)) => this.visit_for_of_loop(f),
        StatementKind::Loop(Loop::ForIn(f)) => this.visit_for_in_loop(f),
        StatementKind::Loop(Loop::DoWhile(d)) => this.visit_do_while_loop(d),
        StatementKind::Return(r) => this.visit_return_statement(r),
        StatementKind::Try(t) => this.visit_try_catch(t),
        StatementKind::Throw(t) => this.visit_throw(t),
        StatementKind::Import(i) => this.visit_import_statement(i),
        StatementKind::Export(e) => this.visit_export_statement(e),
        StatementKind::Class(c) => this.visit_class_declaration(c),
        StatementKind::Continue => this.visit_continue(),
        StatementKind::Break => this.visit_break(),
        StatementKind::Debugger => this.visit_debugger(),
        StatementKind::Empty => this.visit_empty_statement(),
        StatementKind::Switch(s) => this.visit_switch_statement(s),
    }
}

pub fn accept_expr_default<T, V: Visitor<T>, F>(this: &mut V, e: ExprKind, on_empty: F) -> T
where
    F: FnOnce(&mut V) -> T,
{
    match e {
        ExprKind::Binary(e) => this.visit_binary_expression(e),
        ExprKind::Assignment(e) => this.visit_assignment_expression(e),
        ExprKind::Grouping(e) => this.visit_grouping_expression(e),
        ExprKind::Literal(LiteralExpr::Identifier(i)) => this.visit_identifier_expression(i),
        ExprKind::Literal(l) => this.visit_literal_expression(l),
        ExprKind::Unary(e) => this.visit_unary_expression(e),
        ExprKind::Call(e) => this.visit_function_call(e),
        ExprKind::Conditional(e) => this.visit_conditional_expr(e),
        ExprKind::PropertyAccess(e) => this.visit_property_access_expr(e, false),
        ExprKind::Sequence(e) => this.visit_sequence_expr(e),
        ExprKind::Postfix(e) => this.visit_postfix_expr(e),
        ExprKind::Prefix(e) => this.visit_prefix_expr(e),
        ExprKind::Function(e) => this.visit_function_expr(e),
        ExprKind::Array(e) => this.visit_array_literal(e),
        ExprKind::Object(e) => this.visit_object_literal(e),
        ExprKind::Compiled(..) => on_empty(this),
        ExprKind::Empty => this.visit_empty_expr(),
    }
}
