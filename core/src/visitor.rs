use crate::parser::{
    expr::{BinaryExpr, Expr, GroupingExpr, LiteralExpr, UnaryExpr},
    statement::{BlockStatement, FunctionDeclaration, IfStatement, Statement, VariableDeclaration},
};

pub trait Visitor<'a, V> {
    fn accept(&self, e: &Statement<'a>) -> V {
        match e {
            Statement::Expression(e) => self.accept_expr(e),
            Statement::Variable(v) => self.visit_variable_declaration(v),
            Statement::If(i) => self.visit_if_statement(i),
            Statement::Block(b) => self.visit_block_statement(b),
            Statement::Function(f) => self.visit_function_declaration(f),
        }
    }

    fn accept_expr(&self, e: &Expr<'a>) -> V {
        match e {
            Expr::Binary(e) => self.visit_binary_expression(e),
            Expr::Assignment(e) => unreachable!(),
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
}
