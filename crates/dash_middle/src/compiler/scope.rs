use std::cell::RefCell;
use std::convert::TryFrom;

use crate::interner::Symbol;
use crate::parser::statement::{VariableBinding, VariableDeclarationKind, VariableDeclarationName};

use super::external::External;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileValueType {
    Boolean,
    Null,
    Undefined,
    Uninit,
    Number,
    String,
    Array,
    Either(Box<CompileValueType>, Box<CompileValueType>),
    Maybe(Box<CompileValueType>),
    Extern,
}

#[derive(Debug, Clone)]
pub struct ScopeLocal {
    /// The binding of this variable
    binding: VariableBinding,
    inferred_type: RefCell<Option<CompileValueType>>,
}

impl ScopeLocal {
    /// Returns the binding of this variable
    pub fn binding(&self) -> &VariableBinding {
        &self.binding
    }

    /// Sets the inferred type of this local variable
    pub fn infer(&self, value: CompileValueType) {
        *self.inferred_type.borrow_mut() = Some(value);
    }

    pub fn inferred_type(&self) -> &RefCell<Option<CompileValueType>> {
        &self.inferred_type
    }
}

pub struct LimitExceededError;

#[derive(Debug, Default)]

pub struct Scope {
    depth: u16,
    // length limited to u16
    locals: Vec<ScopeLocal>,
    /// A vector of external values
    externals: Vec<External>,
}

impl Scope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn externals(&self) -> &[External] {
        &self.externals
    }

    pub fn externals_mut(&mut self) -> &mut Vec<External> {
        &mut self.externals
    }

    pub fn find_local(&self, identifier: Symbol) -> Option<(u16, &ScopeLocal)> {
        self.locals
            .iter()
            .enumerate()
            .find(|(_, l)| match l.binding() {
                VariableBinding {
                    name: VariableDeclarationName::Identifier(name),
                    kind,
                    ..
                } => *name == identifier && kind.is_nameable(),
                _ => panic!("Only identifiers can be registered"),
            })
            .map(|(i, l)| (i as u16, l))
    }

    pub fn add_local(
        &mut self,
        name: Symbol,
        kind: VariableDeclarationKind,
        inferred_type: Option<CompileValueType>,
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
            inferred_type: RefCell::new(inferred_type),
        });

        u16::try_from(self.locals.len() - 1).map_err(|_| LimitExceededError)
    }

    pub fn add_scope_local(&mut self, local: ScopeLocal) -> Result<u16, LimitExceededError> {
        // TODO: check if it exists already
        self.locals.push(local);
        u16::try_from(self.locals.len() - 1).map_err(|_| LimitExceededError)
    }

    pub fn enter(&mut self) {
        self.depth += 1;
    }

    pub fn exit(&mut self) {
        self.depth -= 1;
    }

    pub fn reset_depth(&mut self) {
        self.depth = 0;
    }

    pub fn depth(&self) -> u16 {
        self.depth
    }

    pub fn locals(&self) -> &[ScopeLocal] {
        self.locals.as_ref()
    }

    pub fn into_locals(self) -> Vec<ScopeLocal> {
        self.locals
    }
}
