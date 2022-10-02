use crate::parser::statement::FunctionDeclaration;
use crate::parser::statement::Statement;

use super::accept_default;
use super::AstWalker;

/// A tree walker that walks a function-local AST, meaning that it only visits statements and expressions that are
/// defined in this function, and not walk other functions.
pub struct FunctionLocalWalker<F>(pub F);

impl<'a, F> AstWalker<'a> for FunctionLocalWalker<F>
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
