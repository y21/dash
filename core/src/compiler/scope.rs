use crate::parser::statement::VariableBinding;
use std::convert::TryFrom;

pub struct ScopeLocal<'a> {
    binding: VariableBinding<'a>,
}

impl<'a> ScopeLocal<'a> {
    pub fn binding(&self) -> &VariableBinding<'a> {
        &self.binding
    }
}

pub struct LimitExceededError;

pub struct Scope<'a> {
    depth: u16,
    // length limited to u16
    locals: Vec<ScopeLocal<'a>>,
}

impl<'a> Scope<'a> {
    pub fn new() -> Self {
        Self {
            depth: 0,
            locals: Vec::new(),
        }
    }

    pub fn find_local(&self, identifier: &[u8]) -> Option<(u16, &ScopeLocal<'a>)> {
        self.locals
            .iter()
            .enumerate()
            .find(|(_, l)| l.binding.name == identifier)
            .map(|(i, l)| (i as u16, l))
    }

    pub fn add_local(&mut self, binding: VariableBinding<'a>) -> Result<u16, LimitExceededError> {
        self.locals.push(ScopeLocal { binding });
        u16::try_from(self.locals.len() - 1).map_err(|_| LimitExceededError)
    }

    pub fn enter(&mut self) {
        self.depth += 1;
    }

    pub fn exit(&mut self) {
        self.depth -= 1;
    }

    pub fn locals(&self) -> &[ScopeLocal] {
        self.locals.as_ref()
    }
}
