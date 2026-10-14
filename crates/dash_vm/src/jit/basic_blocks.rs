use dash_middle::compiler::operands::{
    ConditionalJumpNoPopOperands, ConditionalJumpPopOperands, InstructionOperands, JmpFalseNoPopOperands,
    JmpFalsePopOperands, JmpNullishNoPopOperands, JmpNullishPopOperands, JmpOperands, JmpTrueNoPopOperands,
    JmpTruePopOperands, JmpUndefinedNoPopOperands, JmpUndefinedPopOperands, LoopBackjumpOperands,
};
use rustc_hash::FxHashMap;
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
        match instruction {
            InstructionOperands::Jmp(JmpOperands(offset))
            | InstructionOperands::LoopBackJmp(LoopBackjumpOperands { offset, hotness: _ }) => {
                mark_branch(
                    &mut successors,
                    &mut predecessors,
                    ip_before,
                    ip_after + offset,
                    BranchKind::Jump,
                );
            }
            InstructionOperands::JmpFalseNP(JmpFalseNoPopOperands(ConditionalJumpNoPopOperands {
                offset,
                value: _,
            }))
            | InstructionOperands::JmpFalseP(JmpFalsePopOperands(ConditionalJumpPopOperands { offset, value: _ }))
            | InstructionOperands::JmpTrueNP(JmpTrueNoPopOperands(ConditionalJumpNoPopOperands { offset, value: _ }))
            | InstructionOperands::JmpTrueP(JmpTruePopOperands(ConditionalJumpPopOperands { offset, value: _ }))
            | InstructionOperands::JmpNullishNP(JmpNullishNoPopOperands(ConditionalJumpNoPopOperands {
                offset,
                value: _,
            }))
            | InstructionOperands::JmpNullishP(JmpNullishPopOperands(ConditionalJumpPopOperands {
                offset,
                value: _,
            }))
            | InstructionOperands::JmpUndefinedNP(JmpUndefinedNoPopOperands(ConditionalJumpNoPopOperands {
                offset,
                value: _,
            }))
            | InstructionOperands::JmpUndefinedP(JmpUndefinedPopOperands(ConditionalJumpPopOperands {
                offset,
                value: _,
            })) => {
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
            _ => {
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

pub fn find_block_starts(preds: &PredecessorMap, succs: &SuccessorMap) -> Vec<Ip> {
    let mut block_starts = Vec::new();

    for (ip, preds) in preds {
        // If this is a "sequential" IP, i.e. only one predecessor that just flows sequentially into this instruction, then it's not a block start.
        let is_sequential_ip = preds.len() == 1 && matches!(*succs[&preds[0]], [(_, BranchKind::Sequential)]);

        if !is_sequential_ip {
            block_starts.push(*ip);
        }
    }

    block_starts
}
