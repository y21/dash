use crate::parser::{
    expr::{AssignmentExpr, BinaryExpr, Expr, GroupingExpr, LiteralExpr, UnaryExpr},
    statement::{
        BlockStatement, FunctionDeclaration, IfStatement, Print, Statement, VariableDeclaration,
        WhileLoop,
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
            Statement::Print(p) => self.visit_print_statement(p),
        }
    }

    fn accept_expr(&mut self, e: &Expr<'a>) -> V {
        match e {
            Expr::Binary(e) => self.visit_binary_expression(e),
            Expr::Assignment(e) => self.visit_assignment_expression(e),
            Expr::Grouping(e) => self.visit_grouping_expression(e),
            Expr::Literal(e) => self.visit_literal_expression(e),
            Expr::Unary(e) => self.visit_unary_expression(e),
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
    fn visit_print_statement(&mut self, p: &Print<'a>) -> V;
}
