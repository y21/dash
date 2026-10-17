use dash_middle::compiler::instruction::AssignKind;
use dash_middle::compiler::scope::BackLocalId;
use dash_middle::indexvec::IndexVec;

use crate::jit::dataflow::join::{Join, JoinChange, join_slice};
use crate::jit::dataflow::value::{ValueMetadata, ValueType};
use crate::jit::ir::{
    BasicBlockId, Body, BranchTarget, ConditionalGotoPopPolicy, GenericStmt, Stmt, StmtId, Terminator,
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

    pub fn block_cursor(&self, block: BasicBlockId) -> DataflowBlockCursor {
        let state = self.entry_states[block].clone().unwrap();
        DataflowBlockCursor {
            state,
            block,
            stmt: StmtId(0),
        }
    }

    pub fn block_cursor_mut(&mut self, block: BasicBlockId) -> DataflowBlockCursorMut<'_> {
        let state = self.entry_states[block].clone().unwrap();
        DataflowBlockCursorMut {
            engine: self,
            dirty: false,
            state,
            block,
            stmt: StmtId(0),
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
                    }
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

fn apply_stmt(state: &mut BlockState, stmt: &Stmt) {
    match *stmt {
        Stmt::Generic(ref stmt, _) => match *stmt {
            GenericStmt::LdLocal(local) => state.stack.push(state.locals[local].clone()),
            GenericStmt::LtNumLConstR32(_) => {
                state.stack.pop().unwrap();
                state.stack.push(ValueMetadata { ty: ValueType::Boolean });
            }
            GenericStmt::Number(_) => state.stack.push(ValueMetadata { ty: ValueType::Number }),
            GenericStmt::Pop => drop(state.stack.pop().unwrap()),
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
        },
        Stmt::UncheckedNumberAdd => {
            let rhs = state.stack.pop().unwrap();
            let lhs = state.stack.pop().unwrap();
            let out_ty = ValueMetadata { ty: ValueType::Number };
            assert!(lhs == out_ty && rhs == out_ty);
            state.stack.push(out_ty);
        }
    }
}

pub struct DataflowBlockCursor {
    stmt: StmtId,
    state: BlockState,
    block: BasicBlockId,
}

impl DataflowBlockCursor {
    pub fn seek_to_before_stmt(&mut self, body: &Body, stmt: StmtId) -> &BlockState {
        if self.stmt == stmt {
            &self.state
        } else if stmt > self.stmt {
            self.rerun_from_to(body, self.stmt, stmt, false);
            &self.state
        } else {
            unimplemented!("Backward seeking not implemented yet")
        }
    }

    fn rerun_from_to(&mut self, body: &Body, from: StmtId, to: StmtId, inclusive: bool) {
        let stmts = body.blocks[self.block].stmts.as_slice();

        let end = match inclusive {
            true => StmtId(to.0 + 1),
            false => to,
        };

        for stmt in &stmts[(from.0 as usize)..(end.0 as usize)] {
            apply_stmt(&mut self.state, stmt);
        }

        self.stmt = end;
    }
}

// TODO: store an immutable DataflowBlockCursor and delegate to its methods instead of reimplementing them twice (seeking)
pub struct DataflowBlockCursorMut<'a> {
    /// Whether the local state has actually been mutated and needs to be committed back before dropping the cursor.
    dirty: bool,
    /// The current statement at which the cursor is positioned.
    stmt: StmtId,
    /// The current state of the block at the cursor's position, possibly mutated (if self.dirty).
    state: BlockState,
    /// The basic block in which the cursor is currently positioned.
    block: BasicBlockId,
    engine: &'a mut DataflowEngine,
}

impl<'a> DataflowBlockCursorMut<'a> {
    pub fn seek_to_before(&mut self, body: &Body, stmt: StmtId) -> &BlockState {
        if self.stmt == stmt {
            &self.state
        } else if stmt > self.stmt {
            self.rerun_from_to(body, self.stmt, stmt, false);
            &self.state
        } else {
            unimplemented!("Backward seeking not implemented yet")
        }
    }

    fn rerun_from_to(&mut self, body: &Body, from: StmtId, to: StmtId, inclusive: bool) {
        let stmts = body.blocks[self.block].stmts.as_slice();

        let end = match inclusive {
            true => StmtId(to.0 + 1),
            false => to,
        };

        for stmt in &stmts[(from.0 as usize)..(end.0 as usize)] {
            apply_stmt(&mut self.state, stmt);
        }

        self.stmt = end;
    }

    /// Re-runs dataflow analysis on this block from the the entry up to the specified end statement (exclusive).
    fn rerun_until(&mut self, body: &Body, end: StmtId) {
        let entry = self.engine.entry_states[self.block].clone().unwrap();
        self.state = entry;

        let stmts = body.blocks[self.block].stmts.as_slice();
        for stmt in &stmts[..end.0 as usize] {
            println!("[RERUN] Apply {stmt:?}");
            apply_stmt(&mut self.state, stmt);
            println!("[RERUN] After: {:?}", self.stmt);
        }
    }

    /// Synchronizes the local state with the body up until (including) `at` and marks this block as dirty, reacting to changes that were made to the body.
    /// This should be called when the body has been mutated in any way (e.g. replacing a statement or adding new ones).
    /// After a call to this function, you must call commit() before dropping the cursor to avoid losing unsynchronized changes on the local copy.
    pub fn invalidate_block(&mut self, body: &Body, at: StmtId) {
        self.dirty = true;
        // Include the statement at `at` in the re-run since callers will usually pass the just-modified statement.
        let next = StmtId(at.0 + 1);
        self.rerun_until(body, next);
        self.stmt = next;
    }

    /// Commits any changes made to the local state of this block by propagating it to the successor blocks.
    /// This should be called after changes and invalidate_block calls before dropping.
    pub fn commit(&mut self, body: &Body) {
        if self.dirty {
            self.dirty = false;
            self.engine.run_analysis(body, &[self.block]);
        }
    }
}

impl<'a> Drop for DataflowBlockCursorMut<'a> {
    fn drop(&mut self) {
        assert!(!self.dirty, "Dropping a dirty cursor without syncing the block");
    }
}
