use std::ops::Add;

use dash_middle::compiler::FunctionCallKind;
use dash_middle::compiler::instruction::AssignKind;
use dash_middle::compiler::scope::BackLocalId;
use dash_middle::index_type;
use dash_middle::indexvec::IndexVec;

mod lower;
pub use lower::lower_body;

use crate::frame::Ip;
use crate::jit::dataflow::LocalValues;
use crate::jit::passes::Assumptions;

index_type! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct BasicBlockId(pub u32);
}

#[derive(Debug)]
#[expect(unused)]
pub enum GenericStmt {
    StoreLocal {
        kind: AssignKind,
        local: BackLocalId,
    },
    Add,
    Mul,
    Lt,
    Sub,
    Div,
    Rem,
    Pow,
    Gt,
    Ge,
    Le,
    Eq,
    Ne,
    StrictEq,
    StrictNe,
    BitOr,
    BitXor,
    BitAnd,
    BitShl,
    BitShr,
    BitUshr,
    ObjIn,
    InstanceOf,
    Pos,
    Neg,
    Not,
    BitNot,
    TypeOf,
    CallSymbolIterator,
    CallForInIterator,
    DeletePropertyStatic,
    DeletePropertyDynamic,
    String,
    Regex,
    Function,
    Null,
    Undefined,
    Undef,
    Infinity,
    Global,
    This,
    NewTarget,
    LdGlobal,
    TypeOfGlobalIdent,
    LdLocalExt,
    Nop,
    Debugger,
    ExportNamed,
    ImportStatic {
        local: BackLocalId,
    },
    BindThis,
    ExportDefault,
    ImportDyn,
    Throw,
    Call {
        argc: u8,
        has_this: bool,
        kind: FunctionCallKind,
    },
    StoreGlobal {
        kind: AssignKind,
    },
    StoreLocalExt {
        kind: AssignKind,
    },
    StaticPropAssign {
        kind: AssignKind,
    },
    DynamicPropAssign {
        kind: AssignKind,
    },
    StaticPropAccess {
        preserve_this: bool,
    },
    DynamicPropAccess {
        preserve_this: bool,
    },
    RemNumLR,
    PowNumLR,
    BitOrNumLR,
    BitXorNumLR,
    BitAndNumLR,
    BitShlNumLR,
    BitShrNumLR,
    BitUshrNumLR,
    BuiltinIntrinsicCall {
        argc: u8,
    },
    PostfixDecLocalNum(BackLocalId),
    PrefixIncLocalNum(BackLocalId),
    PrefixDecLocalNum(BackLocalId),
}

index_type! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct StmtId(pub u32);
}

impl Add<u32> for StmtId {
    type Output = StmtId;

    fn add(self, rhs: u32) -> Self::Output {
        StmtId(self.0 + rhs)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ValueSource {
    Float(f64),
    Stack,
}

#[derive(Debug, Copy, Clone)]
pub enum UncheckedAssignmentKind {
    NumberPostfixIncrement,
    // TODO: accept a ValueSource here and specialize looking for a preceding Stmt::Number
    Assignment,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UncheckedNumberBinOp {
    Add,
    Sub,
    Mul,
    Div,
    Gt,
    Ge,
    Lt,
    Le,
    Eq,
    Ne,
}

#[derive(Debug)]
pub enum Stmt {
    Number(f64),
    Boolean(bool),
    /// Applies a numeric binary operation without any type checks.
    UncheckedNumberBinOp {
        op: UncheckedNumberBinOp,
        lhs: ValueSource,
        rhs: ValueSource,
    },
    Pop,
    LdLocal(BackLocalId),
    UncheckedStoreLocal {
        kind: UncheckedAssignmentKind,
        local: BackLocalId,
    },
    /// Non-specialized, generic statement, directly lowered from source bytecode
    Generic(GenericStmt, Ip),
}

#[derive(Copy, Clone, Debug)]
pub enum BranchTarget {
    Block(BasicBlockId),
    Break,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum ConditionalGotoPopPolicy {
    Pop,
    Peek,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum ConditionalGotoCondition {
    /// Any falsy value
    IsFalsy,
    /// Any truthy value
    IsTruthy,
    /// The true value
    IsTrue,
    /// The false value
    IsFalse,
    /// Any nullish value (null or undefined)
    IsNullish,
    /// The undefined value
    IsUndefined,
}

#[derive(Debug)]
pub enum Terminator {
    Goto(BranchTarget),
    ConditionalGoto {
        true_: BranchTarget,
        false_: BranchTarget,
        /// Whether to pop the condition from the stack or peek and keep it
        pop: ConditionalGotoPopPolicy,
        condition: ConditionalGotoCondition,
    },
    Return {
        ip: Ip,
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
                pop: _,
                condition: _,
            } => {
                f(true_);
                f(false_);
            }
            Terminator::Return { ip: _ } => {
                // No targets
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
    pub fn terminator_mut(&mut self) -> &mut Terminator {
        self.terminator.as_mut().unwrap()
    }
}

#[derive(Debug)]
pub struct Body {
    pub blocks: IndexVec<BasicBlock, BasicBlockId>,
    pub root: BasicBlockId,
    pub assumptions: Assumptions,
    pub initial_local_values: LocalValues,
}
