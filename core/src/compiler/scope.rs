use crate::{
    parser::statement::{VariableBinding, VariableDeclarationKind},
    vm::stack::OwnedStack,
};

/// A local binding
#[derive(Debug)]
pub enum LocalBinding<'a> {
    /// An unnamed binding, usually used for reserving stack space by the VM
    Unnamed,
    /// A named binding (variables, functions)
    Named {
        /// The identifier
        ident: &'a [u8],
        /// The type of the variable
        kind: VariableDeclarationKind,
    },
}

impl<'a> LocalBinding<'a> {
    /// Attempts to return a reference to the identifier
    pub fn ident(&self) -> Option<&'a [u8]> {
        match self {
            Self::Named { ident, .. } => Some(ident),
            _ => None,
        }
    }
}

impl<'a> From<VariableBinding<'a>> for LocalBinding<'a> {
    fn from(b: VariableBinding<'a>) -> Self {
        Self::Named {
            ident: b.name,
            kind: b.kind,
        }
    }
}

/// A variable declaration.
#[derive(Debug)]
pub struct Local<'a> {
    /// The depth of this local
    pub depth: u16,
    /// The binding of this local
    pub binding: LocalBinding<'a>,
}

impl<'a> Local<'a> {
    /// Creates a new variable
    pub fn new(depth: u16, binding: LocalBinding<'a>) -> Self {
        Self { depth, binding }
    }

    /// Checks whether this variable is a `const` and can't be assigned to
    pub fn read_only(&self) -> bool {
        matches!(
            self.binding,
            LocalBinding::Named {
                kind: VariableDeclarationKind::Const,
                ..
            }
        )
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
        self.locals
            .find(|local| local.depth <= depth && local.binding.ident() == Some(name))
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
