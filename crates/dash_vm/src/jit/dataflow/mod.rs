use std::cmp::Ordering;
use std::iter;

use dash_middle::compiler::FunctionCallKind;
use dash_middle::compiler::instruction::AssignKind;
use dash_middle::compiler::scope::BackLocalId;
use dash_middle::indexvec::IndexVec;

use crate::jit::dataflow::join::{Join, JoinChange, join_slice};
use crate::jit::dataflow::value::{ValueMetadata, ValueType};
use crate::jit::ir::{
    BasicBlockId, Body, BranchTarget, ConditionalGotoPopPolicy, GenericStmt, Stmt, StmtId, Terminator, ValueSource,
};
use crate::localscope::LocalScope;
use crate::value::{Unpack, ValueKind};

pub mod join;
pub mod value;

#[derive(Debug, Clone)]
pub struct BlockState {
    // TODO: bump allocate these
    pub stack: Vec<ValueMetadata>,
    pub locals: IndexVec<ValueMetadata, BackLocalId>,
}

impl Join for BlockState {
    fn join(&self, other: &Self) -> (Self, JoinChange) {
        assert_eq!(self.stack.len(), other.stack.len());

        let (stack, mut changed) = join_slice(&self.stack, &other.stack);

        let (locals, locals_changed) = join_slice(&self.locals, &other.locals);
        changed |= locals_changed;

        (Self { stack, locals }, changed)
    }
}

#[derive(Debug)]
pub struct DataflowEngine {
    entry_states: IndexVec<Option<BlockState>, BasicBlockId>,
}

fn values_for_locals(scope: &LocalScope<'_>) -> IndexVec<ValueMetadata, BackLocalId> {
    let local_count = scope.frames.current_fn().locals;
    let mut locals = IndexVec::with_capacity(local_count);

    for local in 0..local_count {
        let stack_index = (scope.get_frame_sp() + local).0 as usize;
        let value = scope.stack[stack_index];
        locals.push(match value.unpack() {
            ValueKind::Boolean(_) => ValueMetadata { ty: ValueType::Boolean },
            ValueKind::Number(_) => ValueMetadata { ty: ValueType::Number },
            _ => ValueMetadata { ty: ValueType::Any },
        });
    }

    locals
}

impl DataflowEngine {
    pub fn compute(body: &Body, scope: &LocalScope<'_>) -> Self {
        let entry_states = IndexVec::from_fn(
            |id| {
                if id == body.root {
                    Some(BlockState {
                        stack: Vec::new(),
                        locals: values_for_locals(scope),
                    })
                } else {
                    None
                }
            },
            body.blocks.len(),
        );

        let mut engine = DataflowEngine { entry_states };
        engine.run_analysis(body, &[body.root]);
        engine
    }

    pub fn block_cursor(&self, body: &Body, block: BasicBlockId) -> DataflowBlockCursor {
        let state = self.entry_states[block].clone().unwrap();
        DataflowBlockCursor {
            state,
            block,
            pos: StmtOrTerminator::start(body, block),
        }
    }

    pub fn block_cursor_mut(&mut self, body: &Body, block: BasicBlockId) -> DataflowBlockCursorMut<'_> {
        let cursor = self.block_cursor(body, block);
        DataflowBlockCursorMut {
            engine: self,
            dirty: false,
            cursor,
        }
    }

    fn run_analysis(&mut self, body: &Body, blocks: &[BasicBlockId]) {
        let mut queue = blocks.to_vec();

        while let Some(block) = queue.pop() {
            let mut state = self.entry_states[block].clone().unwrap();
            let block_data = &body.blocks[block];

            for stmt in block_data.stmts.iter() {
                apply_stmt(&mut state, stmt);
            }

            let terminator = block_data.terminator();
            apply_terminator(&mut state, terminator);

            terminator.for_each_branch_target(|branch_target| {
                if let BranchTarget::Block(block) = branch_target {
                    if let Some(existing_state) = &self.entry_states[block] {
                        // Merge the current state with the existing state.
                        let (merged_state, changed) = existing_state.join(&state);
                        self.entry_states[block] = Some(merged_state);
                        if let JoinChange::Changed = changed {
                            queue.push(block);
                        }
                    } else {
                        // No state for this successor yet, so it becomes the current state.
                        self.entry_states[block] = Some(state.clone());
                        queue.push(block);
                    }
                } else {
                    assert!(state.stack.is_empty());
                }
            });
        }
    }
}

fn apply_terminator(state: &mut BlockState, terminator: &Terminator) {
    match *terminator {
        Terminator::Goto(_) => {}
        Terminator::ConditionalGoto {
            true_: _,
            false_: _,
            pop: behavior,
            condition: _,
        } => match behavior {
            ConditionalGotoPopPolicy::Pop => drop(state.stack.pop().unwrap()),
            ConditionalGotoPopPolicy::Peek => {}
        },
    }
}

fn value_metadata_from_source(state: &mut BlockState, source: ValueSource) -> ValueMetadata {
    match source {
        ValueSource::Stack => state.stack.pop().unwrap(),
        ValueSource::Float(_) => ValueMetadata { ty: ValueType::Number },
    }
}

fn apply_stmt(state: &mut BlockState, stmt: &Stmt) {
    match *stmt {
        Stmt::Generic(ref stmt, _) => match *stmt {
            GenericStmt::LtNumLConstR32(_) => {
                state.stack.pop().unwrap();
                state.stack.push(ValueMetadata { ty: ValueType::Boolean });
            }
            GenericStmt::Number(_) => state.stack.push(ValueMetadata { ty: ValueType::Number }),
            GenericStmt::PostfixIncLocalNum(_) => state.stack.push(ValueMetadata { ty: ValueType::Number }),
            GenericStmt::StoreLocal { kind, local } => {
                let out_ty = match kind {
                    AssignKind::Assignment => {
                        let value = state.stack.pop().unwrap();
                        state.locals[local] = value.clone();
                        value
                    }
                    AssignKind::AddAssignment
                    | AssignKind::SubAssignment
                    | AssignKind::MulAssignment
                    | AssignKind::DivAssignment
                    | AssignKind::RemAssignment
                    | AssignKind::PowAssignment
                    | AssignKind::ShlAssignment
                    | AssignKind::ShrAssignment
                    | AssignKind::UshrAssignment
                    | AssignKind::BitAndAssignment
                    | AssignKind::BitOrAssignment
                    | AssignKind::BitXorAssignment => {
                        let rhs = state.stack.pop().unwrap();

                        // For now just handle the trivial, common case that we can actually make use of: lhs and rhs are both numbers
                        // Otherwise fall back to any
                        let metadata = if state.locals[local] == rhs && rhs.ty == ValueType::Number {
                            ValueMetadata { ty: ValueType::Number }
                        } else {
                            ValueMetadata { ty: ValueType::Any }
                        };
                        state.locals[local] = metadata;
                        metadata
                    }
                    AssignKind::PrefixIncrement
                    | AssignKind::PostfixIncrement
                    | AssignKind::PrefixDecrement
                    | AssignKind::PostfixDecrement => {
                        let metadata = if state.locals[local].ty == ValueType::Number {
                            ValueMetadata { ty: ValueType::Number }
                        } else {
                            ValueMetadata { ty: ValueType::Any }
                        };
                        state.locals[local] = metadata;
                        metadata
                    }
                };
                state.stack.push(out_ty);
            }
            GenericStmt::Add => {
                // Trivial case: both operands are numbers
                let rhs = state.stack.pop().unwrap();
                let lhs = state.stack.pop().unwrap();
                let out_ty = if lhs.ty == ValueType::Number && rhs.ty == ValueType::Number {
                    ValueMetadata { ty: ValueType::Number }
                } else {
                    ValueMetadata { ty: ValueType::Any }
                };
                state.stack.push(out_ty);
            }
            GenericStmt::Mul => {
                state.stack.pop().unwrap();
                state.stack.pop().unwrap();
                let out_ty = ValueMetadata { ty: ValueType::Number };
                state.stack.push(out_ty);
            }
            GenericStmt::Lt => {
                state.stack.pop().unwrap();
                state.stack.pop().unwrap();
                let out_ty = ValueMetadata { ty: ValueType::Boolean };
                state.stack.push(out_ty);
            }
            GenericStmt::Sub
            | GenericStmt::Div
            | GenericStmt::Rem
            | GenericStmt::Pow
            | GenericStmt::BitOr
            | GenericStmt::BitXor
            | GenericStmt::BitAnd
            | GenericStmt::BitShl
            | GenericStmt::BitShr
            | GenericStmt::BitUshr
            | GenericStmt::AddNumLR
            | GenericStmt::SubNumLR
            | GenericStmt::MulNumLR
            | GenericStmt::DivNumLR
            | GenericStmt::RemNumLR
            | GenericStmt::PowNumLR
            | GenericStmt::BitOrNumLR
            | GenericStmt::BitXorNumLR
            | GenericStmt::BitAndNumLR
            | GenericStmt::BitShlNumLR
            | GenericStmt::BitShrNumLR
            | GenericStmt::BitUshrNumLR => {
                state.stack.pop().unwrap();
                state.stack.pop().unwrap();
                let out_ty = ValueMetadata { ty: ValueType::Number };
                state.stack.push(out_ty);
            }
            GenericStmt::Gt
            | GenericStmt::Ge
            | GenericStmt::Le
            | GenericStmt::Eq
            | GenericStmt::Ne
            | GenericStmt::StrictEq
            | GenericStmt::StrictNe
            | GenericStmt::ObjIn
            | GenericStmt::InstanceOf
            | GenericStmt::DeletePropertyDynamic
            | GenericStmt::GtNumLR
            | GenericStmt::GeNumLR
            | GenericStmt::LtNumLR
            | GenericStmt::LeNumLR
            | GenericStmt::EqNumLR
            | GenericStmt::NeNumLR => {
                state.stack.pop().unwrap();
                state.stack.pop().unwrap();
                let out_ty = ValueMetadata { ty: ValueType::Boolean };
                state.stack.push(out_ty);
            }
            GenericStmt::Pos | GenericStmt::Neg | GenericStmt::BitNot => {
                state.stack.pop().unwrap();
                let out_ty = ValueMetadata { ty: ValueType::Number };
                state.stack.push(out_ty);
            }
            GenericStmt::Not => {
                state.stack.pop().unwrap();
                let out_ty = ValueMetadata { ty: ValueType::Boolean };
                state.stack.push(out_ty);
            }
            GenericStmt::DeletePropertyStatic
            | GenericStmt::GtNumLConstR(_)
            | GenericStmt::GtNumLConstR32(_)
            | GenericStmt::GeNumLConstR(_)
            | GenericStmt::GeNumLConstR32(_)
            | GenericStmt::LtNumLConstR(_)
            | GenericStmt::LeNumLConstR(_)
            | GenericStmt::LeNumLConstR32(_) => {
                state.stack.pop().unwrap();
                let out_ty = ValueMetadata { ty: ValueType::Boolean };
                state.stack.push(out_ty);
            }
            GenericStmt::TypeOf | GenericStmt::CallSymbolIterator | GenericStmt::CallForInIterator => {
                state.stack.pop().unwrap();
                let out_ty = ValueMetadata { ty: ValueType::Any };
                state.stack.push(out_ty);
            }
            GenericStmt::BindThis | GenericStmt::ExportDefault | GenericStmt::ImportDyn | GenericStmt::Throw => {
                state.stack.pop().unwrap();
            }
            GenericStmt::Nan
            | GenericStmt::Infinity
            | GenericStmt::PostfixDecLocalNum(_)
            | GenericStmt::PrefixIncLocalNum(_)
            | GenericStmt::PrefixDecLocalNum(_) => {
                let out_ty = ValueMetadata { ty: ValueType::Number };
                state.stack.push(out_ty);
            }
            GenericStmt::Boolean => {
                let out_ty = ValueMetadata { ty: ValueType::Boolean };
                state.stack.push(out_ty);
            }
            GenericStmt::Null
            | GenericStmt::Undefined
            | GenericStmt::Undef
            | GenericStmt::Global
            | GenericStmt::This
            | GenericStmt::NewTarget
            | GenericStmt::String
            | GenericStmt::Regex
            | GenericStmt::Function
            | GenericStmt::LdGlobal
            | GenericStmt::TypeOfGlobalIdent
            | GenericStmt::LdLocalExt => {
                let out_ty = ValueMetadata { ty: ValueType::Any };
                state.stack.push(out_ty);
            }
            GenericStmt::Nop | GenericStmt::Debugger | GenericStmt::ExportNamed => {}
            GenericStmt::ImportStatic { local } => {
                state.locals[local] = ValueMetadata { ty: ValueType::Any };
            }
            GenericStmt::Call { argc, has_this, kind } => {
                let mut stack_values = argc;

                if let FunctionCallKind::Super = kind {
                    // super() call has neither a `this` value on the stack nor a callee.
                } else if has_this {
                    // Callee + `this` on the stack
                    stack_values += 2;
                } else {
                    // Otherwise, normal function call with just the callee on the stack
                    stack_values += 1;
                }

                for _ in 0..stack_values {
                    state.stack.pop().unwrap();
                }
                let out_ty = ValueMetadata { ty: ValueType::Any };
                state.stack.push(out_ty);
            }
            GenericStmt::StoreGlobal { kind } | GenericStmt::StoreLocalExt { kind } => {
                let out_ty = match kind {
                    AssignKind::Assignment => state.stack.pop().unwrap(),
                    AssignKind::AddAssignment
                    | AssignKind::SubAssignment
                    | AssignKind::MulAssignment
                    | AssignKind::DivAssignment
                    | AssignKind::RemAssignment
                    | AssignKind::PowAssignment
                    | AssignKind::ShlAssignment
                    | AssignKind::ShrAssignment
                    | AssignKind::UshrAssignment
                    | AssignKind::BitAndAssignment
                    | AssignKind::BitOrAssignment
                    | AssignKind::BitXorAssignment => {
                        state.stack.pop().unwrap();
                        ValueMetadata { ty: ValueType::Any }
                    }
                    AssignKind::PrefixIncrement
                    | AssignKind::PostfixIncrement
                    | AssignKind::PrefixDecrement
                    | AssignKind::PostfixDecrement => ValueMetadata { ty: ValueType::Any },
                };
                state.stack.push(out_ty);
            }
            GenericStmt::StaticPropAssign { kind } => {
                let out_ty = match kind {
                    AssignKind::Assignment => state.stack.pop().unwrap(),
                    AssignKind::AddAssignment
                    | AssignKind::SubAssignment
                    | AssignKind::MulAssignment
                    | AssignKind::DivAssignment
                    | AssignKind::RemAssignment
                    | AssignKind::PowAssignment
                    | AssignKind::ShlAssignment
                    | AssignKind::ShrAssignment
                    | AssignKind::UshrAssignment
                    | AssignKind::BitAndAssignment
                    | AssignKind::BitOrAssignment
                    | AssignKind::BitXorAssignment => {
                        state.stack.pop().unwrap(); // value
                        ValueMetadata { ty: ValueType::Any }
                    }
                    AssignKind::PrefixIncrement
                    | AssignKind::PostfixIncrement
                    | AssignKind::PrefixDecrement
                    | AssignKind::PostfixDecrement => ValueMetadata { ty: ValueType::Any },
                };
                state.stack.pop().unwrap(); // target
                state.stack.push(out_ty);
            }
            GenericStmt::DynamicPropAssign { kind } => {
                let out_ty = match kind {
                    AssignKind::Assignment => state.stack.pop().unwrap(),
                    AssignKind::AddAssignment
                    | AssignKind::SubAssignment
                    | AssignKind::MulAssignment
                    | AssignKind::DivAssignment
                    | AssignKind::RemAssignment
                    | AssignKind::PowAssignment
                    | AssignKind::ShlAssignment
                    | AssignKind::ShrAssignment
                    | AssignKind::UshrAssignment
                    | AssignKind::BitAndAssignment
                    | AssignKind::BitOrAssignment
                    | AssignKind::BitXorAssignment => {
                        state.stack.pop().unwrap(); // value
                        ValueMetadata { ty: ValueType::Any }
                    }
                    AssignKind::PrefixIncrement
                    | AssignKind::PostfixIncrement
                    | AssignKind::PrefixDecrement
                    | AssignKind::PostfixDecrement => ValueMetadata { ty: ValueType::Any },
                };
                state.stack.pop().unwrap(); // key
                state.stack.pop().unwrap(); // target
                state.stack.push(out_ty);
            }
        },
        Stmt::UncheckedNumberAdd { lhs, rhs } => {
            let lhs = value_metadata_from_source(state, lhs);
            let rhs = value_metadata_from_source(state, rhs);

            let out_ty = ValueMetadata { ty: ValueType::Number };
            assert!(lhs == out_ty && rhs == out_ty);
            state.stack.push(out_ty);
        }
        Stmt::UncheckedNumberLt { lhs, rhs } => {
            let lhs = value_metadata_from_source(state, lhs);
            let rhs = value_metadata_from_source(state, rhs);

            let out_ty = ValueMetadata { ty: ValueType::Boolean };
            assert!(lhs == rhs && rhs.ty == ValueType::Number);
            state.stack.push(out_ty);
        }
        Stmt::Pop => drop(state.stack.pop().unwrap()),
        Stmt::LdLocal(local) => state.stack.push(state.locals[local].clone()),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum StmtOrTerminator {
    /// Positioned before the given `StmtId`
    Stmt(StmtId),
    /// Position before the terminator
    Terminator,
    /// Positioned after the terminator
    PostTerminator,
}

impl Ord for StmtOrTerminator {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (StmtOrTerminator::Stmt(a), StmtOrTerminator::Stmt(b)) => a.cmp(b),
            (StmtOrTerminator::Terminator, StmtOrTerminator::Terminator)
            | (StmtOrTerminator::PostTerminator, StmtOrTerminator::PostTerminator) => Ordering::Equal,
            (StmtOrTerminator::Stmt(_), StmtOrTerminator::Terminator | StmtOrTerminator::PostTerminator) => {
                Ordering::Less
            }
            (StmtOrTerminator::Terminator | StmtOrTerminator::PostTerminator, StmtOrTerminator::Stmt(_)) => {
                Ordering::Greater
            }
            (StmtOrTerminator::Terminator, StmtOrTerminator::PostTerminator) => Ordering::Less,
            (StmtOrTerminator::PostTerminator, StmtOrTerminator::Terminator) => Ordering::Greater,
        }
    }
}

impl StmtOrTerminator {
    pub fn start(body: &Body, block: BasicBlockId) -> StmtOrTerminator {
        if body.blocks[block].stmts.is_empty() {
            StmtOrTerminator::Terminator
        } else {
            StmtOrTerminator::Stmt(StmtId(0))
        }
    }

    pub fn next(self, block: BasicBlockId, body: &Body) -> Option<Self> {
        match self {
            StmtOrTerminator::Stmt(stmt_id) => {
                let next = stmt_id + 1;
                if next.0 == body.blocks[block].stmts.len() {
                    Some(StmtOrTerminator::Terminator)
                } else {
                    Some(StmtOrTerminator::Stmt(next))
                }
            }
            StmtOrTerminator::Terminator => Some(StmtOrTerminator::PostTerminator),
            StmtOrTerminator::PostTerminator => None,
        }
    }
}

pub struct DataflowBlockCursor {
    pos: StmtOrTerminator,
    state: BlockState,
    block: BasicBlockId,
}

impl DataflowBlockCursor {
    pub fn seek_to_before(&mut self, body: &Body, to: StmtOrTerminator) -> &BlockState {
        match self.pos.cmp(&to) {
            Ordering::Less | Ordering::Equal => {
                self.rerun_from_to(body, self.pos, to);
                &self.state
            }
            Ordering::Greater => {
                unimplemented!("Forward seeking not implemented yet")
            }
        }
    }

    fn rerun_from_to(&mut self, body: &Body, from: StmtOrTerminator, to: StmtOrTerminator) {
        use StmtOrTerminator::{Stmt, Terminator};
        assert!(to >= from);

        let iter = iter::successors(Some(from), |pos| pos.next(self.block, body)).take_while(|&stmt| stmt != to);

        let block = &body.blocks[self.block];
        for pos in iter {
            match pos {
                Stmt(stmt_id) => apply_stmt(&mut self.state, &block.stmts[stmt_id]),
                Terminator => {
                    apply_terminator(&mut self.state, block.terminator());
                }
                StmtOrTerminator::PostTerminator => {}
            }
        }

        self.pos = to;
    }
}

pub struct DataflowBlockCursorMut<'a> {
    /// Whether the local state has actually been mutated and needs to be committed back before dropping the cursor.
    dirty: bool,
    cursor: DataflowBlockCursor,
    engine: &'a mut DataflowEngine,
}

impl<'a> DataflowBlockCursorMut<'a> {
    pub fn seek_to_before(&mut self, body: &Body, to: StmtOrTerminator) -> &BlockState {
        self.cursor.seek_to_before(body, to);
        &self.cursor.state
    }

    /// Re-runs dataflow analysis on this block from the the entry up to the specified end statement (exclusive).
    fn rerun_until(&mut self, body: &Body, end: StmtOrTerminator) {
        let entry = self.engine.entry_states[self.cursor.block].clone().unwrap();
        self.cursor.state = entry;
        self.cursor.pos = StmtOrTerminator::start(body, self.cursor.block);
        self.cursor.seek_to_before(body, end);
    }

    /// Synchronizes the local state with the body up until (including) `at` and marks this block as dirty, reacting to changes that were made to the body.
    /// This should be called when the body has been mutated in any way (e.g. replacing a statement or adding new ones).
    /// After a call to this function, you must call commit() before dropping the cursor to avoid losing unsynchronized changes on the local copy.
    pub fn invalidate_block(&mut self, body: &Body, at: StmtOrTerminator) {
        self.dirty = true;
        // Include the statement at `at` in the re-run since callers will usually pass the just-modified statement.
        let next = at.next(self.cursor.block, body).unwrap();
        self.rerun_until(body, next);
    }

    /// Commits any changes made to the local state of this block by propagating it to the successor blocks.
    /// This should be called after changes and invalidate_block calls before dropping.
    pub fn commit(&mut self, body: &Body) {
        if self.dirty {
            self.dirty = false;
            self.engine.run_analysis(body, &[self.cursor.block]);
        }
    }
}

impl<'a> Drop for DataflowBlockCursorMut<'a> {
    fn drop(&mut self) {
        assert!(!self.dirty, "Dropping a dirty cursor without syncing the block");
    }
}
