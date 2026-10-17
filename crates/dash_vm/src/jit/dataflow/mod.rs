use dash_middle::compiler::instruction::AssignKind;
use dash_middle::compiler::scope::BackLocalId;
use dash_middle::indexvec::IndexVec;

use crate::jit::dataflow::join::{Join, JoinChange, join_slice};
use crate::jit::dataflow::value::{ValueMetadata, ValueType};
use crate::jit::ir::{BasicBlockId, Body, BranchTarget, ConditionalGotoBehavior, GenericStmt, Stmt, Terminator};
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
            behavior,
        } => match behavior {
            ConditionalGotoBehavior::Pop => drop(state.stack.pop().unwrap()),
            ConditionalGotoBehavior::Peek => {}
        },
    }
}

fn apply_stmt(state: &mut BlockState, stmt: &Stmt) {
    match *stmt {
        Stmt::Generic(GenericStmt::LdLocal(local)) => state.stack.push(state.locals[local].clone()),
        Stmt::Generic(GenericStmt::LtNumLConstR32(_)) => {
            state.stack.pop().unwrap();
            state.stack.push(ValueMetadata { ty: ValueType::Boolean });
        }
        Stmt::Generic(GenericStmt::Number(_)) => state.stack.push(ValueMetadata { ty: ValueType::Number }),
        Stmt::Generic(GenericStmt::Pop) => drop(state.stack.pop().unwrap()),
        Stmt::Generic(GenericStmt::PostfixIncLocalNum(_)) => state.stack.push(ValueMetadata { ty: ValueType::Number }),
        Stmt::Generic(GenericStmt::StoreLocal { kind, local }) => {
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
    }
}
