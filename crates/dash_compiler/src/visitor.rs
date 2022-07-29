use dash_middle::parser::expr::ArrayLiteral;
use dash_middle::parser::expr::AssignmentExpr;
use dash_middle::parser::expr::BinaryExpr;
use dash_middle::parser::expr::ConditionalExpr;
use dash_middle::parser::expr::Expr;
use dash_middle::parser::expr::FunctionCall;
use dash_middle::parser::expr::GroupingExpr;
use dash_middle::parser::expr::LiteralExpr;
use dash_middle::parser::expr::ObjectLiteral;
use dash_middle::parser::expr::Postfix;
use dash_middle::parser::expr::PropertyAccessExpr;
use dash_middle::parser::expr::Seq;
use dash_middle::parser::expr::UnaryExpr;
use dash_middle::parser::statement::BlockStatement;
use dash_middle::parser::statement::Class;
use dash_middle::parser::statement::ExportKind;
use dash_middle::parser::statement::ForLoop;
use dash_middle::parser::statement::ForOfLoop;
use dash_middle::parser::statement::FunctionDeclaration;
use dash_middle::parser::statement::IfStatement;
use dash_middle::parser::statement::ImportKind;
use dash_middle::parser::statement::ReturnStatement;
use dash_middle::parser::statement::Statement;
use dash_middle::parser::statement::SwitchStatement;
use dash_middle::parser::statement::TryCatch;
use dash_middle::parser::statement::VariableBinding;
use dash_middle::parser::statement::VariableDeclaration;
use dash_middle::parser::statement::WhileLoop;

/// A visitor trait that helps walking an AST
pub trait Visitor<'a, V> {
    /// Accepts a parsed statement
    fn accept(&mut self, e: Statement<'a>) -> V;

    /// Accepts a parsed expression
    fn accept_expr(&mut self, e: Expr<'a>) -> V;

    /// Visits an expression statement
    fn visit_expression_statement(&mut self, e: Expr<'a>) -> V;

    /// Visits a binary expression
    fn visit_binary_expression(&mut self, e: BinaryExpr<'a>) -> V;

    /// Visits a grouping expression
    fn visit_grouping_expression(&mut self, e: GroupingExpr<'a>) -> V;

    /// Visits a literal expression
    fn visit_literal_expression(&mut self, e: LiteralExpr<'a>) -> V;

    /// Visits an identifier
    fn visit_identifier_expression(&mut self, i: &str) -> V;

    /// Visits a binding expression
    fn visit_binding_expression(&mut self, b: VariableBinding<'a>) -> V;

    /// Visits an unary expression
    fn visit_unary_expression(&mut self, e: UnaryExpr<'a>) -> V;

    /// Visits a variable declaration
    fn visit_variable_declaration(&mut self, v: VariableDeclaration<'a>) -> V;

    /// Visits an if statement
    fn visit_if_statement(&mut self, i: IfStatement<'a>) -> V;

    /// Visits a block statement
    fn visit_block_statement(&mut self, b: BlockStatement<'a>) -> V;

    /// Visits a function declaration
    fn visit_function_declaration(&mut self, f: FunctionDeclaration<'a>) -> V;

    /// Visits a while loop
    fn visit_while_loop(&mut self, l: WhileLoop<'a>) -> V;

    /// Visits an assignment expression
    fn visit_assignment_expression(&mut self, e: AssignmentExpr<'a>) -> V;

    /// Visits a function call
    fn visit_function_call(&mut self, c: FunctionCall<'a>) -> V;

    /// Visits a return statement
    fn visit_return_statement(&mut self, s: ReturnStatement<'a>) -> V;

    /// Visits a conditional expression
    fn visit_conditional_expr(&mut self, c: ConditionalExpr<'a>) -> V;

    /// Visits a property access expression
    ///
    /// This includes both computed access and static access
    fn visit_property_access_expr(&mut self, e: PropertyAccessExpr<'a>, preserve_this: bool) -> V;

    /// Visits a sequence expression
    fn visit_sequence_expr(&mut self, s: Seq<'a>) -> V;

    /// Visits any postfix expression
    fn visit_postfix_expr(&mut self, p: Postfix<'a>) -> V;

    /// Visits a function expression
    fn visit_function_expr(&mut self, f: FunctionDeclaration<'a>) -> V;

    /// Visits an array literal
    fn visit_array_literal(&mut self, a: ArrayLiteral<'a>) -> V;

    /// Visits an object literal
    fn visit_object_literal(&mut self, o: ObjectLiteral<'a>) -> V;

    /// Visits a try catch statement
    fn visit_try_catch(&mut self, t: TryCatch<'a>) -> V;

    /// Visits a throw statement
    fn visit_throw(&mut self, e: Expr<'a>) -> V;

    /// Visits a for loop
    fn visit_for_loop(&mut self, f: ForLoop<'a>) -> V;

    /// Visits a for..of loop
    fn visit_for_of_loop(&mut self, f: ForOfLoop<'a>) -> V;

    /// Visits an import statement
    fn visit_import_statement(&mut self, i: ImportKind<'a>) -> V;

    /// Visits an export statement
    fn visit_export_statement(&mut self, e: ExportKind<'a>) -> V;

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
    fn visit_class_declaration(&mut self, c: Class<'a>) -> V;

    /// Visits a switch statement
    fn visit_switch_statement(&mut self, s: SwitchStatement<'a>) -> V;
}
