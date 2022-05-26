use super::parser::Parser;

pub struct TypeParser<'a, 'ctx>(&'ctx mut Parser<'a>);

impl<'a, 'ctx> TypeParser<'a, 'ctx> {
    pub fn new(parser: &'ctx mut Parser<'a>) -> Self {
        Self(parser)
    }

    pub fn parse(&mut self) -> Option<()> {
        todo!()
    }

    fn ty(&mut self) -> Option<()> {
        todo!()
    }

    fn conditional_ty(&mut self) -> Option<()> {
        todo!()
    }

    fn non_conditional_ty(&mut self) -> Option<()> {
        todo!()
    }
}
