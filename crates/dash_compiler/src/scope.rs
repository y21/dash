use std::cell::Cell;
use std::convert::TryFrom;

use dash_middle::parser::statement::VariableBinding;
use dash_middle::parser::statement::VariableDeclarationKind;
use dash_middle::parser::statement::VariableDeclarationName;

#[derive(Debug, Clone)]
pub struct ScopeLocal<'a> {
    /// The binding of this variable
    binding: VariableBinding<'a>,
    /// Whether this local variable is used by inner functions and as such may outlive the frame when returned
    is_extern: Cell<bool>,
}

impl<'a> ScopeLocal<'a> {
    pub fn binding(&self) -> &VariableBinding<'a> {
        &self.binding
    }

    /// Marks this local variable as "extern"
    pub fn set_extern(&self) {
        self.is_extern.set(true);
    }

    /// Checks whether this local variable is marked extern
    pub fn is_extern(&self) -> bool {
        self.is_extern.get()
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

    pub fn find_local(&self, identifier: &str) -> Option<(u16, &ScopeLocal<'a>)> {
        self.locals
            .iter()
            .enumerate()
            .find(|(_, l)| match l.binding.name {
                VariableDeclarationName::Identifier(i) => i == identifier && l.binding.kind.is_nameable(),
                _ => panic!("Only identifiers can be registered"),
            })
            .map(|(i, l)| (i as u16, l))
    }

    pub fn find_binding(&self, binding: &VariableBinding<'a>) -> Option<(u16, &ScopeLocal<'a>)> {
        self.locals
            .iter()
            .enumerate()
            .find(|(_, l)| &l.binding == binding)
            .map(|(i, l)| (i as u16, l))
    }

    pub fn add_local(
        &mut self,
        name: &'a str,
        kind: VariableDeclarationKind,
        is_extern: bool,
    ) -> Result<u16, LimitExceededError> {
        // if there's already a local with the same name, we should use that
        if let Some((id, _)) = self.find_local(name) {
            return Ok(id);
        }

        self.locals.push(ScopeLocal {
            binding: VariableBinding {
                name: VariableDeclarationName::Identifier(name),
                kind,
                ty: None,
            },
            is_extern: Cell::new(is_extern),
        });

        u16::try_from(self.locals.len() - 1).map_err(|_| LimitExceededError)
    }

    pub fn enter(&mut self) {
        self.depth += 1;
    }

    pub fn exit(&mut self) {
        self.depth -= 1;
    }

    pub fn locals(&self) -> &[ScopeLocal<'a>] {
        self.locals.as_ref()
    }
}
