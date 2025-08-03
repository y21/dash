use std::cell::RefCell;
use std::ops::{Index, IndexMut};

use crate::indexvec::{self, IndexVec};
use crate::interner::Symbol;
use crate::parser::statement::{ScopeId, VariableDeclarationKind};
use crate::util::Counter;
use crate::{index_type, with};

#[derive(Debug)]
pub struct LimitExceededError;

index_type! {
    /// Local indices "in the backend", i.e. used by the compiler and the VM.
    /// As opposed to [`FrontLocalId`], which is given out for "syntactic locals" during parsing.
    /// Frontend locals are lowered to backend locals during name res.
    #[derive(derive_more::Display, Debug, Copy, Clone, PartialEq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct BackLocalId(pub u16);
}

#[derive(Debug, Clone)]
pub struct FunctionScope {
    /// All locals that are declared in this function.
    /// Every scope that this function encloses (including the function's `Scope` itself) will have its own declarations.
    /// Some scopes may not have any declarations at all, such as unnameable locals.
    /// There can also be multiple locals with the same name.
    pub locals: IndexVec<Local, BackLocalId>,
}
impl FunctionScope {
    pub fn add_local(&mut self, local: Local) -> Result<BackLocalId, LimitExceededError> {
        self.locals.try_push(local).ok_or(LimitExceededError)
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
    /// All variable declarations in this scope.
    /// It contains the identifier, as well as the local slot
    /// (index into `functions` of the enclosing function scope).
    pub declarations: Vec<(Symbol, BackLocalId)>,
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
    pub fn find_local(&self, name: Symbol) -> Option<BackLocalId> {
        self.declarations
            .iter()
            .find_map(|&(name2, slot)| (name2 == name).then_some(slot))
    }
}

impl Index<ScopeId> for ScopeGraph {
    type Output = Scope;

    fn index(&self, index: ScopeId) -> &Self::Output {
        &self.scopes[index]
    }
}

impl IndexMut<ScopeId> for ScopeGraph {
    fn index_mut(&mut self, index: ScopeId) -> &mut Self::Output {
        &mut self.scopes[index]
    }
}

#[derive(Debug)]
pub struct ScopeGraph {
    scopes: IndexVec<Scope, ScopeId>,
}

impl ScopeGraph {
    pub fn new(count: <ScopeId as indexvec::Index>::Repr) -> Self {
        Self {
            scopes: IndexVec::repeat_n(Scope::new(ScopeKind::Uninit, None), count),
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
            kind: ScopeKind::Function(FunctionScope {
                locals: IndexVec::new(),
            }),
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
    ) -> Result<BackLocalId, LimitExceededError> {
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
            &self[function].expect_function().locals[slot]
        })
    }
}

pub struct FindResult {
    pub slot: BackLocalId,
    pub scope: ScopeId,
}
