use dash_middle::compiler::instruction::AssignKind;
use dash_middle::compiler::scope::BackLocalId;
use dash_middle::index_type;
use dash_middle::indexvec::IndexVec;

mod lower;
pub use lower::lower_body;

index_type! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct BasicBlockId(pub u32);
}

#[derive(Debug)]
pub enum GenericStmt {
    LdLocal(BackLocalId),
    StoreLocal { kind: AssignKind, local: BackLocalId },
    LtNumLConstR32(f64),
    Number(f64),
    Pop,
    PostfixIncLocalNum(BackLocalId),
}

index_type! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct StmtId(pub u32);
}

#[derive(Debug)]
pub enum Stmt {
    /// Non-specialized, generic statement, directly lowered from source bytecode
    Generic(GenericStmt),
}

#[derive(Copy, Clone, Debug)]
pub enum BranchTarget {
    Block(BasicBlockId),
    Break,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConditionalGotoBehavior {
    Pop,
    Peek,
}

#[derive(Debug)]
pub enum Terminator {
    Goto(BranchTarget),
    ConditionalGoto {
        true_: BranchTarget,
        false_: BranchTarget,
        /// Whether to pop the condition from the stack or peek and keep it
        behavior: ConditionalGotoBehavior,
    },
}

impl Terminator {
    pub fn for_each_branch_target<F>(&self, mut f: F)
    where
        F: FnMut(BranchTarget),
    {
        match *self {
            Terminator::Goto(branch_target) => f(branch_target),
            Terminator::ConditionalGoto {
                true_,
                false_,
                behavior: _,
            } => {
                f(true_);
                f(false_);
            }
        }
    }
}

#[derive(Debug)]
pub struct BasicBlock {
    pub stmts: IndexVec<Stmt, StmtId>,
    pub terminator: Option<Terminator>,
}

impl BasicBlock {
    pub fn dummy() -> Self {
        Self {
            stmts: IndexVec::new(),
            terminator: None,
        }
    }
    pub fn terminator(&self) -> &Terminator {
        self.terminator.as_ref().unwrap()
    }
}

#[derive(Debug)]
pub struct Body {
    pub blocks: IndexVec<BasicBlock, BasicBlockId>,
    pub root: BasicBlockId,
}
