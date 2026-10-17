use std::collections::hash_map::Entry;

use dash_middle::compiler::operands::{
    ConditionalJumpNoPopOperands, ConditionalJumpPopOperands, InstructionOperands, IntrinsicKind, IntrinsicOperands,
    JmpFalseNoPopOperands, JmpFalsePopOperands, JmpNullishNoPopOperands, JmpNullishPopOperands, JmpOperands,
    JmpTrueNoPopOperands, JmpTruePopOperands, JmpUndefinedNoPopOperands, JmpUndefinedPopOperands, LdLocalOperands,
    LoopBackjumpOperands, NumberConstantOperands, NumberConstantWide, NumberInline32, PopOperands, StoreLocalOperands,
};
use dash_middle::indexvec::IndexVec;
use rustc_hash::FxHashMap;

use crate::frame::Ip;
use crate::jit::ir::{
    BasicBlock, BasicBlockId, Body, BranchTarget, ConditionalGotoBehavior, GenericStmt, Stmt, Terminator,
};
use crate::jit::{BufferRef, NoValueExtractContext};
use crate::localscope::LocalScope;

fn create_terminator(
    bytecode: BufferRef<'_>,
    blocks: &mut IndexVec<BasicBlock, BasicBlockId>,
    ip_to_block: &mut FxHashMap<Ip, BasicBlockId>,
    scope: &LocalScope<'_>,
    cx: &mut NoValueExtractContext<'_, '_>,
    offset: i16,
    behavior: ConditionalGotoBehavior,
) -> Terminator {
    let true_target = cx.ip() + offset;
    let false_target = cx.ip();

    let true_ = lower_to_branch_target(bytecode, true_target, blocks, ip_to_block, scope);
    let false_ = lower_to_branch_target(bytecode, false_target, blocks, ip_to_block, scope);
    Terminator::ConditionalGoto {
        true_,
        false_,
        behavior,
    }
}

fn lower_to_branch_target(
    bytecode: BufferRef<'_>,
    ip: Ip,
    blocks: &mut IndexVec<BasicBlock, BasicBlockId>,
    ip_to_block: &mut FxHashMap<Ip, BasicBlockId>,
    scope: &LocalScope<'_>,
) -> BranchTarget {
    if ip.0 == bytecode.len() {
        BranchTarget::Break
    } else {
        BranchTarget::Block(lower_from_ip(bytecode, ip, blocks, ip_to_block, scope))
    }
}

fn lower_from_ip(
    bytecode: BufferRef<'_>,
    ip: Ip,
    blocks: &mut IndexVec<BasicBlock, BasicBlockId>,
    ip_to_block: &mut FxHashMap<Ip, BasicBlockId>,
    scope: &LocalScope<'_>,
) -> BasicBlockId {
    let block = match ip_to_block.entry(ip) {
        Entry::Occupied(entry) => {
            return *entry.get();
        }
        Entry::Vacant(entry) => {
            let block = blocks.push(BasicBlock::dummy());
            entry.insert(block);
            block
        }
    };

    let mut stmts = IndexVec::new();
    let mut terminator = None;

    let mut cx = NoValueExtractContext::new(bytecode.as_slice(), ip, scope);
    while let Some((instr_ip, instr)) = cx.next_instruction_with_operands() {
        match instr {
            InstructionOperands::LdLocal(LdLocalOperands(local)) => {
                stmts.push(Stmt::Generic(GenericStmt::LdLocal(local)));
            }
            InstructionOperands::IntrinsicOp(IntrinsicOperands(kind)) => match kind {
                IntrinsicKind::LtNumLConstR32((), NumberInline32(num)) => {
                    stmts.push(Stmt::Generic(GenericStmt::LtNumLConstR32(num)));
                }
                IntrinsicKind::PostfixIncLocalNum(local) => {
                    stmts.push(Stmt::Generic(GenericStmt::PostfixIncLocalNum(local)));
                }
                _ => unimplemented!("Intrinsic {kind:?} at {instr_ip:?} is not implemented"),
            },
            InstructionOperands::Number(NumberConstantOperands(NumberConstantWide(num))) => {
                stmts.push(Stmt::Generic(GenericStmt::Number(num)));
            }
            InstructionOperands::StoreLocal(StoreLocalOperands { kind, local }) => {
                stmts.push(Stmt::Generic(GenericStmt::StoreLocal {
                    kind: kind.into(),
                    local,
                }));
            }
            InstructionOperands::Pop(PopOperands(())) => {
                stmts.push(Stmt::Generic(GenericStmt::Pop));
            }

            InstructionOperands::Jmp(JmpOperands(offset))
            | InstructionOperands::LoopBackJmp(LoopBackjumpOperands { offset, hotness: _ }) => {
                let target = cx.ip() + offset;
                let succ = lower_to_branch_target(bytecode, target, blocks, ip_to_block, scope);
                terminator = Some(Terminator::Goto(succ));
                break;
            }
            InstructionOperands::JmpFalseNP(JmpFalseNoPopOperands(ConditionalJumpNoPopOperands {
                offset,
                value: _,
            }))
            | InstructionOperands::JmpTrueNP(JmpTrueNoPopOperands(ConditionalJumpNoPopOperands { offset, value: _ }))
            | InstructionOperands::JmpNullishNP(JmpNullishNoPopOperands(ConditionalJumpNoPopOperands {
                offset,
                value: _,
            }))
            | InstructionOperands::JmpUndefinedNP(JmpUndefinedNoPopOperands(ConditionalJumpNoPopOperands {
                offset,
                value: _,
            })) => {
                terminator = Some(create_terminator(
                    bytecode,
                    blocks,
                    ip_to_block,
                    scope,
                    &mut cx,
                    offset,
                    ConditionalGotoBehavior::Peek,
                ));
                break;
            }
            InstructionOperands::JmpFalseP(JmpFalsePopOperands(ConditionalJumpPopOperands { offset, value: _ }))
            | InstructionOperands::JmpTrueP(JmpTruePopOperands(ConditionalJumpPopOperands { offset, value: _ }))
            | InstructionOperands::JmpNullishP(JmpNullishPopOperands(ConditionalJumpPopOperands {
                offset,
                value: _,
            }))
            | InstructionOperands::JmpUndefinedP(JmpUndefinedPopOperands(ConditionalJumpPopOperands {
                offset,
                value: _,
            })) => {
                terminator = Some(create_terminator(
                    bytecode,
                    blocks,
                    ip_to_block,
                    scope,
                    &mut cx,
                    offset,
                    ConditionalGotoBehavior::Pop,
                ));
                break;
            }
            _ => unimplemented!("Instruction {instr:?} at {instr_ip:?} is not implemented"),
        }
    }

    blocks[block] = BasicBlock { stmts, terminator };
    block
}

pub fn lower_body(bytecode: BufferRef<'_>, scope: &LocalScope<'_>) -> Body {
    let mut blocks = IndexVec::new();
    let mut ip_to_block = FxHashMap::default();
    let root = lower_from_ip(bytecode, Ip::ZERO, &mut blocks, &mut ip_to_block, scope);

    Body { blocks, root }
}
