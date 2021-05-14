use crate::vm::stack::Stack;

#[derive(Debug)]
pub struct Local<'a> {
    pub ident: &'a [u8],
    pub depth: u16,
}

impl<'a> Local<'a> {
    pub fn new(ident: &'a [u8], depth: u16) -> Self {
        Self { ident, depth }
    }
}

#[derive(Debug)]
pub struct ScopeGuard<T, const N: usize> {
    pub depth: u16,
    locals: Stack<T, N>,
}

impl<'a, const N: usize> ScopeGuard<Local<'a>, N> {
    pub fn find_variable(&self, name: &'a [u8]) -> Option<usize> {
        let depth = self.depth;
        self.locals
            .find(|local| {
                local.depth <= depth && local.ident.len() == name.len() && local.ident.eq(name)
            })
            .map(|(idx, _)| idx)
    }

    pub fn push_local(&mut self, local: Local<'a>) -> usize {
        self.locals.push(local);
        self.local_count() - 1
    }
}

impl<T, const N: usize> ScopeGuard<T, N> {
    pub fn new() -> Self {
        Self {
            locals: Stack::new(),
            depth: 0,
        }
    }

    pub fn local_count(&self) -> usize {
        self.locals.len()
    }

    pub fn enter_scope(&mut self) {
        self.depth += 1;
    }

    pub fn leave_scope(&mut self) {
        self.depth -= 1;
    }

    pub fn is_global(&self) -> bool {
        self.depth == 0
    }
}
