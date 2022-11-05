use dash_middle::compiler::scope::Scope;

pub struct OptimizerContext<'a> {
    scope: Scope<'a>,
}

impl<'a> OptimizerContext<'a> {
    pub fn new() -> Self {
        Self { scope: Scope::new() }
    }

    pub fn scope_mut(&mut self) -> &mut Scope<'a> {
        &mut self.scope
    }
}
