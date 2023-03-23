use std::collections::HashMap;

use dash_middle::compiler::instruction::Instruction;

use crate::error::Error;
use crate::util::DecodeCtxt;

#[derive(Debug)]
pub enum LabelKind {
    UnconditionalJumpTarget { target: usize },
    ConditionalJumpTarget { target_true: usize, target_false: usize },
}

#[derive(Debug)]
pub struct Labels(pub Vec<LabelKind>);

pub type BasicBlockKey = usize;
pub type BasicBlockMap = HashMap<usize, BasicBlock>;

#[derive(Debug)]
pub struct BasicBlock {
    pub index: usize,
    pub end: usize,
    pub predecessors: Vec<BasicBlockKey>,
    pub successor: Option<BasicBlockSuccessor>,
}

#[derive(Debug, Clone, Copy)]
pub enum BasicBlockSuccessor {
    Unconditional(usize),
    Conditional {
        true_ip: usize,
        false_ip: usize,
        /// [`None`] is used for actions that are not yet known.
        action: Option<ConditionalBranchAction>,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum ConditionalBranchAction {
    Taken,
    NotTaken,
    Either,
}

pub trait BBGenerationQuery {
    fn conditional_branch_at(&self, ip: usize) -> Option<ConditionalBranchAction>;
}

#[derive(Debug)]
pub struct BBGeneration {
    /// A "pool" of basic blocks
    pub bbs: Vec<BasicBlock>,
}
/// Identifies labels (i.e. the target of jump instructions, either
/// conditional jumps or unconditional jumps) in bytecode
pub fn find_labels(bytecode: &[u8]) -> Result<Labels, Error> {
    let mut labels = Vec::new();
    let mut dcx = DecodeCtxt::new(bytecode);

    while let Some((index, instr)) = dcx.next_instruction() {
        match instr {
            // Instructions we care about:
            Instruction::Jmp => {
                let count = dcx.next_wide_signed();
                let target_ip = usize::try_from(index as i16 + count + 3).unwrap();
                labels.push(LabelKind::UnconditionalJumpTarget { target: target_ip });
            }
            Instruction::JmpFalseP
            | Instruction::JmpNullishP
            | Instruction::JmpTrueP
            | Instruction::JmpUndefinedP
            | Instruction::JmpFalseNP
            | Instruction::JmpNullishNP
            | Instruction::JmpTrueNP
            | Instruction::JmpUndefinedNP => {
                let count = dcx.next_wide_signed();
                let target_ip = usize::try_from(index as i16 + count + 3).unwrap();
                labels.push(LabelKind::ConditionalJumpTarget {
                    target_true: target_ip,
                    target_false: index + 3,
                });
            }

            // Remaining instructions we do not care about but still need to decode
            other => dcx.decode_ignore(other),
        }
    }

    labels.sort_by_key(|label| match label {
        LabelKind::UnconditionalJumpTarget { target } => *target,
        LabelKind::ConditionalJumpTarget {
            target_true,
            target_false,
        } => usize::min(*target_true, *target_false),
    });

    Ok(Labels(labels))
}

#[derive(Debug)]
pub struct BBGenerationCtxt<'a, 'q, Q> {
    pub bytecode: &'a [u8],
    pub labels: Vec<LabelKind>,
    pub bbs: BasicBlockMap,
    pub query: &'q mut Q,
}

impl<'a, 'q, Q: BBGenerationQuery> BBGenerationCtxt<'a, 'q, Q> {
    pub fn find_bbs(&mut self) {
        self.bbs.insert(
            0,
            BasicBlock {
                index: 0,
                end: self.bytecode.len(),
                predecessors: Vec::new(),
                successor: None,
            },
        );

        for label in &self.labels {
            match label {
                LabelKind::UnconditionalJumpTarget { target } => {
                    self.bbs.insert(
                        *target,
                        BasicBlock {
                            index: *target,
                            end: self.bytecode.len(),
                            predecessors: Vec::new(),
                            successor: None,
                        },
                    );
                }
                LabelKind::ConditionalJumpTarget {
                    target_true,
                    target_false,
                } => {
                    self.bbs.insert(
                        *target_true,
                        BasicBlock {
                            index: *target_true,
                            end: self.bytecode.len(),
                            predecessors: Vec::new(),
                            successor: None,
                        },
                    );
                    self.bbs.insert(
                        *target_false,
                        BasicBlock {
                            index: *target_false,
                            end: self.bytecode.len(),
                            predecessors: Vec::new(),
                            successor: None,
                        },
                    );
                }
            }
        }
    }

    /// Resolves predecessors and successors of every basic block
    pub fn resolve_edges(&mut self) {
        let mut dcx = DecodeCtxt::new(self.bytecode);
        let mut current_bb_ip = 0;

        while let Some((index, instr)) = dcx.next_instruction() {
            if index != 0 {
                if let Some(..) = self.bbs.get(&index) {
                    let current_bb = self.bbs.get_mut(&current_bb_ip).unwrap();
                    if current_bb.successor.is_none() {
                        current_bb.successor = Some(BasicBlockSuccessor::Unconditional(index));
                    }
                    current_bb.end = index;
                    current_bb_ip = index;
                }
            }

            match instr {
                // Instructions we care about:
                Instruction::Jmp => {
                    let count = dcx.next_wide_signed();
                    let target_ip = usize::try_from(index as i16 + count + 3).unwrap();

                    let this = self.bbs.get_mut(&current_bb_ip).unwrap();
                    assert!(this.successor.is_none());
                    this.successor = Some(BasicBlockSuccessor::Unconditional(target_ip));
                    this.end = index;

                    let bb = self.bbs.get_mut(&target_ip).unwrap();
                    bb.predecessors.push(current_bb_ip);
                }
                Instruction::JmpFalseP
                | Instruction::JmpNullishP
                | Instruction::JmpTrueP
                | Instruction::JmpUndefinedP
                | Instruction::JmpFalseNP
                | Instruction::JmpNullishNP
                | Instruction::JmpTrueNP
                | Instruction::JmpUndefinedNP => {
                    let count = dcx.next_wide_signed();
                    let target_ip = usize::try_from(index as i16 + count + 3).unwrap();
                    let action = self.query.conditional_branch_at(index);

                    let this = self.bbs.get_mut(&current_bb_ip).unwrap();
                    assert!(this.successor.is_none());
                    this.successor = Some(BasicBlockSuccessor::Conditional {
                        true_ip: target_ip,
                        false_ip: index + 3,
                        action,
                    });
                    this.end = index;

                    let bb_true = self.bbs.get_mut(&target_ip).unwrap();
                    bb_true.predecessors.push(current_bb_ip);

                    let bb_false = self.bbs.get_mut(&(index + 3)).unwrap();
                    bb_false.predecessors.push(current_bb_ip);
                }

                // Remaining instructions we do not care about but still need to decode
                other => dcx.decode_ignore(other),
            }
        }
    }
}
