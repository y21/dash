use dash_middle::compiler::operands::InstructionSuccessorOffset;
use rustc_hash::{FxHashMap, FxHashSet};
use smallvec::SmallVec;

use crate::frame::Ip;
use crate::jit::NoValueExtractContext;
use crate::localscope::LocalScope;

pub type PredecessorMap = FxHashMap<Ip, SmallVec<[Ip; 2]>>;

#[derive(Debug)]
pub enum BranchKind {
    /// The branch is the next instruction.
    Sequential,
    /// The branch is an explicit jump.
    Jump,
}

pub type SuccessorMap = FxHashMap<Ip, SmallVec<[(Ip, BranchKind); 2]>>;

pub type BlockStartSet = FxHashSet<Ip>;

pub fn successors_predecessors(bytecode: &[u8], scope: &LocalScope<'_>) -> (SuccessorMap, PredecessorMap) {
    fn mark_branch(
        successors: &mut SuccessorMap,
        predecessors: &mut PredecessorMap,
        from: Ip,
        to: Ip,
        kind: BranchKind,
    ) {
        successors.entry(from).or_default().push((to, kind));
        predecessors.entry(to).or_default().push(from);
    }

    let mut successors: SuccessorMap = FxHashMap::default();
    let mut predecessors: PredecessorMap = FxHashMap::default();

    let mut cx = NoValueExtractContext::new(bytecode, scope);

    while let Some((ip_before, instruction)) = cx.next_instruction_with_operands() {
        let ip_after = cx.ip();

        match instruction.successor() {
            Some(InstructionSuccessorOffset::Conditional(offset)) => {
                mark_branch(
                    &mut successors,
                    &mut predecessors,
                    ip_before,
                    ip_after + offset,
                    BranchKind::Jump,
                );
                mark_branch(
                    &mut successors,
                    &mut predecessors,
                    ip_before,
                    ip_after,
                    BranchKind::Sequential,
                );
            }
            Some(InstructionSuccessorOffset::Unconditional(offset)) => {
                mark_branch(
                    &mut successors,
                    &mut predecessors,
                    ip_before,
                    ip_after + offset,
                    BranchKind::Jump,
                );
            }
            None => {
                if cx.has_next() {
                    mark_branch(
                        &mut successors,
                        &mut predecessors,
                        ip_before,
                        ip_after,
                        BranchKind::Sequential,
                    );
                }
            }
        }
    }

    (successors, predecessors)
}

pub fn find_block_starts(preds: &PredecessorMap, succs: &SuccessorMap) -> BlockStartSet {
    let mut block_starts = BlockStartSet::default();

    for (ip, preds) in preds {
        // If this is a "sequential" IP, i.e. only one predecessor that just flows sequentially into this instruction, then it's not a block start.
        let is_sequential_ip = preds.len() == 1 && matches!(*succs[&preds[0]], [(_, BranchKind::Sequential)]);

        if !is_sequential_ip {
            block_starts.insert(*ip);
        }
    }

    block_starts
}
