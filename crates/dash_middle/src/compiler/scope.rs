use std::cell::RefCell;
use std::iter;
use std::ops::{Index, IndexMut};

use crate::interner::Symbol;
use crate::parser::statement::{ScopeId, VariableDeclarationKind};
use crate::util::Counter;
use crate::with;

#[derive(Debug)]
pub struct LimitExceededError;

#[derive(Debug, Clone)]
pub struct FunctionScope {
    // TODO: document the difference between this and `declarations`
    pub locals: Vec<Local>,
}
impl FunctionScope {
    pub fn add_local(&mut self, local: Local) -> Result<u16, LimitExceededError> {
        let id = self.locals.len();
        self.locals.push(local);
        id.try_into().map_err(|_| LimitExceededError)
    }
}

#[derive(Debug, Clone)]
pub struct BlockScope {
    pub enclosing_function: ScopeId,
}

#[derive(Debug, Clone)]
pub enum ScopeKind {
    Function(FunctionScope),
    Block(BlockScope),
    Uninit,
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub kind: ScopeKind,
    pub parent: Option<ScopeId>,
    pub subscopes: Vec<ScopeId>,
    pub declarations: Vec<(Symbol, u16)>,
}

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
pub struct Local {
    /// The binding of this variable
    // binding: VariableBinding,
    pub name: Symbol,
    pub kind: VariableDeclarationKind,
    pub inferred_type: RefCell<Option<CompileValueType>>,
}

impl Local {
    /// Sets the inferred type of this local variable
    pub fn infer(&self, value: CompileValueType) {
        *self.inferred_type.borrow_mut() = Some(value);
    }

    pub fn inferred_type(&self) -> &RefCell<Option<CompileValueType>> {
        &self.inferred_type
    }
}
impl Scope {
    pub fn new(kind: ScopeKind, parent: Option<ScopeId>) -> Self {
        Self {
            kind,
            parent,
            subscopes: Vec::new(),
            declarations: Vec::new(),
        }
    }
    pub fn with_parent(mut self, parent: ScopeId) -> Self {
        self.parent = Some(parent);
        self
    }
    pub fn expect_function_mut(&mut self) -> &mut FunctionScope {
        match &mut self.kind {
            ScopeKind::Function(f) => f,
            other => panic!("expected function, got {other:?}"),
        }
    }
    pub fn expect_function(&self) -> &FunctionScope {
        match &self.kind {
            ScopeKind::Function(f) => f,
            other => panic!("expected function, got {other:?}"),
        }
    }
    /// Looks for a local **in this scope** (only): it does not look in upper scopes.
    /// Consider using `ScopeGraph::find_local` instead if you need that
    pub fn find_local(&self, name: Symbol) -> Option<u16> {
        self.declarations
            .iter()
            .find_map(|&(name2, slot)| (name2 == name).then_some(slot))
    }
}
impl Index<ScopeId> for ScopeGraph {
    type Output = Scope;

    fn index(&self, index: ScopeId) -> &Self::Output {
        &self.scopes[index.0]
    }
}
impl IndexMut<ScopeId> for ScopeGraph {
    fn index_mut(&mut self, index: ScopeId) -> &mut Self::Output {
        &mut self.scopes[index.0]
    }
}

#[derive(Debug)]
pub struct ScopeGraph {
    scopes: Vec<Scope>,
}

impl ScopeGraph {
    pub fn new(count: usize) -> Self {
        Self {
            scopes: iter::repeat(Scope::new(ScopeKind::Uninit, None)).take(count).collect(),
        }
    }

    /// If this is a block, returns the enclosing function.
    /// If it's a function, returns itself
    pub fn enclosing_function_of(&self, of: ScopeId) -> ScopeId {
        match self[of].kind {
            ScopeKind::Function { .. } => of,
            ScopeKind::Block(BlockScope { enclosing_function }) => enclosing_function,
            ScopeKind::Uninit => panic!("cannot get enclosing function of uninit scope {of:?}"),
        }
    }

    pub fn add_empty_function_scope(&mut self, at: ScopeId, counter: &mut Counter<ScopeId>) -> ScopeId {
        let id = counter.inc();
        assert_eq!(self.scopes.len(), id.0);

        self[at].subscopes.push(id);
        self.scopes.push(Scope {
            kind: ScopeKind::Function(FunctionScope { locals: Vec::new() }),
            declarations: Vec::new(),
            parent: Some(at),
            subscopes: Vec::new(),
        });

        id
    }

    pub fn add_empty_block_scope(&mut self, at: ScopeId, counter: &mut Counter<ScopeId>) -> ScopeId {
        let enclosing_function = self.enclosing_function_of(at);

        let id = counter.inc();
        assert_eq!(self.scopes.len(), id.0);
        self[at].subscopes.push(id);

        self.scopes.push(Scope {
            kind: ScopeKind::Block(BlockScope { enclosing_function }),
            parent: Some(at),
            subscopes: vec![],
            declarations: vec![],
        });

        id
    }

    /// See [VariableDeclarationKind::Unnameable] for the exact semantics here.
    pub fn add_unnameable_local(
        &mut self,
        at: ScopeId,
        name: Symbol,
        ty: Option<CompileValueType>,
    ) -> Result<u16, LimitExceededError> {
        let enclosing_function = self.enclosing_function_of(at);

        with!(self[enclosing_function].expect_function_mut(), |fun| {
            fun.add_local(Local {
                inferred_type: RefCell::new(ty),
                kind: VariableDeclarationKind::Unnameable,
                name,
            })
        })
    }

    pub fn find(&self, at: ScopeId, name: Symbol) -> Option<FindResult> {
        if let Some(slot) = self[at].find_local(name) {
            Some(FindResult { slot, scope: at })
        } else {
            let parent = self[at].parent?;
            self.find(parent, name)
                .map(|FindResult { slot, scope }| FindResult { slot, scope })
        }
    }

    pub fn find_local(&self, at: ScopeId, name: Symbol) -> Option<&Local> {
        self.find(at, name).map(|FindResult { slot, scope, .. }| {
            let function = self.enclosing_function_of(scope);
            &self[function].expect_function().locals[slot as usize]
        })
    }
}

pub struct FindResult {
    pub slot: u16,
    pub scope: ScopeId,
}
