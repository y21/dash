use crate::parser::expr::Expr;
use crate::parser::statement::FunctionDeclaration;
use crate::parser::statement::Statement;

use super::accept_default;
use super::accept_expr_default;
use super::AstWalker;

/// A tree walker that walks a function-local AST, meaning that it only visits statements and expressions that are
/// defined in this function, and not walk other functions.
pub struct FunctionLocalStatementWalker<F>(pub F);

impl<'a, F> AstWalker<'a> for FunctionLocalStatementWalker<F>
where
    F: Fn(&Statement<'a>),
{
    fn accept(&mut self, e: Statement<'a>) {
        (self.0)(&e);
        accept_default(self, e)
    }

    fn visit_function_declaration(&mut self, _f: FunctionDeclaration<'a>) {
        // Do nothing
    }
}

/// A tree walker that walks a function-local AST, meaning that it only visits statements and expressions that are
/// defined in this function, and not walk other functions.
pub struct FunctionLocalExpressionWalker<F, T>(pub F, pub T);

impl<'a, F, T> AstWalker<'a> for FunctionLocalExpressionWalker<F, T>
where
    F: Fn(&Expr<'a>, &mut T),
{
    fn accept_expr(&mut self, e: Expr<'a>) {
        (self.0)(&e, &mut self.1);
        accept_expr_default(self, e)
    }

    fn visit_function_declaration(&mut self, _f: FunctionDeclaration<'a>) {
        // Do nothing
    }
}
