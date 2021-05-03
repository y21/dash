use crate::parser::{
    expr::{AssignmentExpr, BinaryExpr, Expr, GroupingExpr, LiteralExpr, UnaryExpr},
    statement::{
        BlockStatement, FunctionDeclaration, IfStatement, Statement, VariableDeclaration, WhileLoop,
    },
};

pub trait Visitor<'a, V> {
    fn accept(&self, e: &Statement<'a>) -> V {
        match e {
            Statement::Expression(e) => self.accept_expr(e),
            Statement::Variable(v) => self.visit_variable_declaration(v),
            Statement::If(i) => self.visit_if_statement(i),
            Statement::Block(b) => self.visit_block_statement(b),
            Statement::Function(f) => self.visit_function_declaration(f),
            Statement::While(l) => self.visit_while_loop(l),
        }
    }

    fn accept_expr(&self, e: &Expr<'a>) -> V {
        match e {
            Expr::Binary(e) => self.visit_binary_expression(e),
            Expr::Assignment(e) => self.visit_assignment_expression(e),
            Expr::Grouping(e) => self.visit_grouping_expression(e),
            Expr::Literal(e) => self.visit_literal_expression(e),
            Expr::Unary(e) => self.visit_unary_expression(e),
        }
    }

    fn visit_binary_expression(&self, e: &BinaryExpr<'a>) -> V;
    fn visit_grouping_expression(&self, e: &GroupingExpr<'a>) -> V;
    fn visit_literal_expression(&self, e: &LiteralExpr<'a>) -> V;
    fn visit_unary_expression(&self, e: &UnaryExpr<'a>) -> V;
    fn visit_variable_declaration(&self, v: &VariableDeclaration<'a>) -> V;
    fn visit_if_statement(&self, i: &IfStatement) -> V;
    fn visit_block_statement(&self, b: &BlockStatement) -> V;
    fn visit_function_declaration(&self, f: &FunctionDeclaration) -> V;
    fn visit_while_loop(&self, l: &WhileLoop) -> V;
    fn visit_assignment_expression(&self, e: &AssignmentExpr) -> V;
}
