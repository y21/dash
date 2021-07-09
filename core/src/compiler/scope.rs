use crate::{parser::statement::VariableDeclarationKind, vm::stack::OwnedStack};

/// A variable declaration.
#[derive(Debug)]
pub struct Local<'a> {
    /// Identifier
    pub ident: &'a [u8],
    /// Depth of this declaration
    pub depth: u16,
    /// Variable kind
    pub kind: VariableDeclarationKind,
}

impl<'a> Local<'a> {
    /// Creates a new variable
    pub fn new(ident: &'a [u8], depth: u16, kind: VariableDeclarationKind) -> Self {
        Self { ident, depth, kind }
    }

    /// Checks whether this variable is a `const` and can't be assigned to
    pub fn read_only(&self) -> bool {
        matches!(self.kind, VariableDeclarationKind::Const)
    }
}

/// Manages scopes
#[derive(Debug)]
pub struct ScopeGuard<T, const N: usize> {
    /// Current depth
    pub depth: u16,
    locals: OwnedStack<T, N>,
}

impl<'a, const N: usize> ScopeGuard<Local<'a>, N> {
    /// Tries to find a variable
    pub fn find_variable(&self, name: &'a [u8]) -> Option<(usize, &Local)> {
        let depth = self.depth;
        self.locals.find(|local| {
            local.depth <= depth && local.ident.len() == name.len() && local.ident.eq(name)
        })
    }

    /// Stores a variable declaration
    pub fn push_local(&mut self, local: Local<'a>) -> usize {
        self.locals.push(local);
        self.local_count() - 1
    }
}

impl<T, const N: usize> ScopeGuard<T, N> {
    /// Creates a new ScopeGuard
    pub fn new() -> Self {
        Self {
            locals: OwnedStack::new(),
            depth: 0,
        }
    }

    /// Returns the number of local variables
    pub fn local_count(&self) -> usize {
        self.locals.len()
    }

    /// Enters a new scope
    pub fn enter_scope(&mut self) {
        self.depth += 1;
    }

    /// Leaves the current scope
    pub fn leave_scope(&mut self) {
        self.depth -= 1;
    }

    /// Checks whether this scope is the global scope
    pub fn is_global(&self) -> bool {
        self.depth == 0
    }
}
