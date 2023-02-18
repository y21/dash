use std::cell::Cell;
use std::cell::RefCell;
use std::convert::TryFrom;

use crate::parser::statement::VariableBinding;
use crate::parser::statement::VariableDeclarationKind;
use crate::parser::statement::VariableDeclarationName;

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
}

#[derive(Debug, Clone)]
pub struct ScopeLocal<'a> {
    /// The binding of this variable
    binding: VariableBinding<'a>,
    inferred_type: RefCell<Option<CompileValueType>>,
}

impl<'a> ScopeLocal<'a> {
    /// Returns the binding of this variable
    pub fn binding(&self) -> &VariableBinding<'a> {
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

#[derive(Debug)]

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
        name: &'a str,
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

    pub fn add_scope_local(&mut self, local: ScopeLocal<'a>) -> Result<u16, LimitExceededError> {
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

    pub fn locals(&self) -> &[ScopeLocal<'a>] {
        self.locals.as_ref()
    }

    pub fn into_locals(self) -> Vec<ScopeLocal<'a>> {
        self.locals
    }
}
