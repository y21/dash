use crate::parser::{
    expr::{
        AssignmentExpr, BinaryExpr, ConditionalExpr, Expr, FunctionCall, GroupingExpr, LiteralExpr,
        Postfix, PropertyAccessExpr, Seq, UnaryExpr,
    },
    statement::{
        BlockStatement, FunctionDeclaration, IfStatement, ReturnStatement, Statement,
        VariableDeclaration, WhileLoop,
    },
};

pub trait Visitor<'a, V> {
    fn accept(&mut self, e: &Statement<'a>) -> V {
        match e {
            Statement::Expression(e) => self.visit_expression_statement(e),
            Statement::Variable(v) => self.visit_variable_declaration(v),
            Statement::If(i) => self.visit_if_statement(i),
            Statement::Block(b) => self.visit_block_statement(b),
            Statement::Function(f) => self.visit_function_declaration(f),
            Statement::While(l) => self.visit_while_loop(l),
            Statement::Return(r) => self.visit_return_statement(r),
        }
    }

    fn accept_expr(&mut self, e: &Expr<'a>) -> V {
        match e {
            Expr::Binary(e) => self.visit_binary_expression(e),
            Expr::Assignment(e) => self.visit_assignment_expression(e),
            Expr::Grouping(e) => self.visit_grouping_expression(e),
            Expr::Literal(e) => self.visit_literal_expression(e),
            Expr::Unary(e) => self.visit_unary_expression(e),
            Expr::Call(e) => self.visit_function_call(e),
            Expr::Conditional(e) => self.visit_conditional_expr(e),
            Expr::PropertyAccess(e) => self.visit_property_access_expr(e),
            Expr::Sequence(e) => self.visit_sequence_expr(e),
            Expr::Postfix(e) => self.visit_postfix_expr(e),
        }
    }

    fn visit_expression_statement(&mut self, e: &Expr<'a>) -> V;
    fn visit_binary_expression(&mut self, e: &BinaryExpr<'a>) -> V;
    fn visit_grouping_expression(&mut self, e: &GroupingExpr<'a>) -> V;
    fn visit_literal_expression(&mut self, e: &LiteralExpr<'a>) -> V;
    fn visit_unary_expression(&mut self, e: &UnaryExpr<'a>) -> V;
    fn visit_variable_declaration(&mut self, v: &VariableDeclaration<'a>) -> V;
    fn visit_if_statement(&mut self, i: &IfStatement<'a>) -> V;
    fn visit_block_statement(&mut self, b: &BlockStatement<'a>) -> V;
    fn visit_function_declaration(&mut self, f: &FunctionDeclaration<'a>) -> V;
    fn visit_while_loop(&mut self, l: &WhileLoop<'a>) -> V;
    fn visit_assignment_expression(&mut self, e: &AssignmentExpr<'a>) -> V;
    fn visit_function_call(&mut self, c: &FunctionCall<'a>) -> V;
    fn visit_return_statement(&mut self, s: &ReturnStatement<'a>) -> V;
    fn visit_conditional_expr(&mut self, c: &ConditionalExpr<'a>) -> V;
    fn visit_property_access_expr(&mut self, e: &PropertyAccessExpr<'a>) -> V;
    fn visit_sequence_expr(&mut self, s: &Seq<'a>) -> V;
    fn visit_postfix_expr(&mut self, p: &Postfix<'a>) -> V;
}
