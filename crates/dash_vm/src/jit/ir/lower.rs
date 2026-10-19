use std::collections::hash_map::Entry;

use dash_middle::compiler::operands::{
    AddOperands, BinaryOperator, BindThisOperands, BitandOperands, BitnotOperands, BitorOperands, BitshlOperands,
    BitshrOperands, BitushrOperands, BitxorOperands, BooleanConstantOperands, BooleanConstantWide, CallOperands,
    CallSymbolIteratorOperands, ConditionalJumpNoPopOperands, ConditionalJumpPopOperands,
    DeletePropertyDynamicOperands, DeletePropertyStaticOperands, DivOperands, DynamicPropertyAssignOperands,
    EqOperands, ExportDefaultOperands, ExportNamedOperands, ForInIteratorOperands, FunctionConstantOperands,
    GeOperands, GtOperands, ImportDynOperands, ImportStaticOperands, InstanceofOperands, InstructionOperands,
    IntrinsicKind, IntrinsicOperands, JmpFalseNoPopOperands, JmpFalsePopOperands, JmpNullishNoPopOperands,
    JmpNullishPopOperands, JmpOperands, JmpTrueNoPopOperands, JmpTruePopOperands, JmpUndefinedNoPopOperands,
    JmpUndefinedPopOperands, LdGlobalOperands, LdLocalExtOperands, LdLocalOperands, LeOperands, LoopBackjumpOperands,
    LtOperands, MulOperands, NeOperands, NegOperands, NotOperands, NumberConstantOperands, NumberConstantWide,
    NumberInline8, NumberInline32, ObjectInOperands, PopOperands, PosOperands, PowOperands, RegexConstantOperands,
    RemOperands, StaticPropertyAssignOperands, StoreGlobalOperands, StoreLocalExtOperands, StoreLocalOperands,
    StrictEqOperands, StrictNeOperands, StringConstantOperands, SubOperands, SymbolConstantWide, ThrowOperands,
    TypeofIdentOperands, TypeofOperands,
};
use dash_middle::indexvec::IndexVec;
use rustc_hash::FxHashMap;

use crate::frame::Ip;
use crate::jit::ir::{
    BasicBlock, BasicBlockId, Body, BranchTarget, ConditionalGotoCondition, ConditionalGotoPopPolicy, GenericStmt,
    Stmt, Terminator,
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
    pop: ConditionalGotoPopPolicy,
    condition: ConditionalGotoCondition,
) -> Terminator {
    let true_target = cx.ip() + offset;
    let false_target = cx.ip();

    let true_ = lower_to_branch_target(bytecode, true_target, blocks, ip_to_block, scope);
    let false_ = lower_to_branch_target(bytecode, false_target, blocks, ip_to_block, scope);
    Terminator::ConditionalGoto {
        true_,
        false_,
        pop,
        condition,
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
                stmts.push(Stmt::Generic(GenericStmt::LdLocal(local), instr_ip));
            }
            InstructionOperands::IntrinsicOp(IntrinsicOperands(kind)) => match kind {
                IntrinsicKind::LtNumLConstR32((), NumberInline32(num)) => {
                    stmts.push(Stmt::Generic(GenericStmt::LtNumLConstR32(num), instr_ip));
                }
                IntrinsicKind::PostfixIncLocalNum(local) => {
                    stmts.push(Stmt::Generic(GenericStmt::PostfixIncLocalNum(local), instr_ip));
                }
                IntrinsicKind::PostfixDecLocalNum(local) => {
                    stmts.push(Stmt::Generic(GenericStmt::PostfixDecLocalNum(local), instr_ip));
                }
                IntrinsicKind::PrefixIncLocalNum(local) => {
                    stmts.push(Stmt::Generic(GenericStmt::PrefixIncLocalNum(local), instr_ip));
                }
                IntrinsicKind::PrefixDecLocalNum(local) => {
                    stmts.push(Stmt::Generic(GenericStmt::PrefixDecLocalNum(local), instr_ip));
                }
                IntrinsicKind::AddNumLR((), ()) => {
                    stmts.push(Stmt::Generic(GenericStmt::AddNumLR, instr_ip));
                }
                IntrinsicKind::SubNumLR((), ()) => {
                    stmts.push(Stmt::Generic(GenericStmt::SubNumLR, instr_ip));
                }
                IntrinsicKind::MulNumLR((), ()) => {
                    stmts.push(Stmt::Generic(GenericStmt::MulNumLR, instr_ip));
                }
                IntrinsicKind::DivNumLR((), ()) => {
                    stmts.push(Stmt::Generic(GenericStmt::DivNumLR, instr_ip));
                }
                IntrinsicKind::RemNumLR((), ()) => {
                    stmts.push(Stmt::Generic(GenericStmt::RemNumLR, instr_ip));
                }
                IntrinsicKind::PowNumLR((), ()) => {
                    stmts.push(Stmt::Generic(GenericStmt::PowNumLR, instr_ip));
                }
                IntrinsicKind::GtNumLR((), ()) => {
                    stmts.push(Stmt::Generic(GenericStmt::GtNumLR, instr_ip));
                }
                IntrinsicKind::GeNumLR((), ()) => {
                    stmts.push(Stmt::Generic(GenericStmt::GeNumLR, instr_ip));
                }
                IntrinsicKind::LtNumLR((), ()) => {
                    stmts.push(Stmt::Generic(GenericStmt::LtNumLR, instr_ip));
                }
                IntrinsicKind::LeNumLR((), ()) => {
                    stmts.push(Stmt::Generic(GenericStmt::LeNumLR, instr_ip));
                }
                IntrinsicKind::EqNumLR((), ()) => {
                    stmts.push(Stmt::Generic(GenericStmt::EqNumLR, instr_ip));
                }
                IntrinsicKind::NeNumLR((), ()) => {
                    stmts.push(Stmt::Generic(GenericStmt::NeNumLR, instr_ip));
                }
                IntrinsicKind::BitOrNumLR((), ()) => {
                    stmts.push(Stmt::Generic(GenericStmt::BitOrNumLR, instr_ip));
                }
                IntrinsicKind::BitXorNumLR((), ()) => {
                    stmts.push(Stmt::Generic(GenericStmt::BitXorNumLR, instr_ip));
                }
                IntrinsicKind::BitAndNumLR((), ()) => {
                    stmts.push(Stmt::Generic(GenericStmt::BitAndNumLR, instr_ip));
                }
                IntrinsicKind::BitShlNumLR((), ()) => {
                    stmts.push(Stmt::Generic(GenericStmt::BitShlNumLR, instr_ip));
                }
                IntrinsicKind::BitShrNumLR((), ()) => {
                    stmts.push(Stmt::Generic(GenericStmt::BitShrNumLR, instr_ip));
                }
                IntrinsicKind::BitUshrNumLR((), ()) => {
                    stmts.push(Stmt::Generic(GenericStmt::BitUshrNumLR, instr_ip));
                }
                IntrinsicKind::GtNumLConstR((), NumberInline8(num)) => {
                    stmts.push(Stmt::Generic(GenericStmt::GtNumLConstR(num), instr_ip));
                }
                IntrinsicKind::GtNumLConstR32((), NumberInline32(num)) => {
                    stmts.push(Stmt::Generic(GenericStmt::GtNumLConstR32(num), instr_ip));
                }
                IntrinsicKind::GeNumLConstR((), NumberInline8(num)) => {
                    stmts.push(Stmt::Generic(GenericStmt::GeNumLConstR(num), instr_ip));
                }
                IntrinsicKind::GeNumLConstR32((), NumberInline32(num)) => {
                    stmts.push(Stmt::Generic(GenericStmt::GeNumLConstR32(num), instr_ip));
                }
                IntrinsicKind::LtNumLConstR((), NumberInline8(num)) => {
                    stmts.push(Stmt::Generic(GenericStmt::LtNumLConstR(num), instr_ip));
                }
                IntrinsicKind::LeNumLConstR((), NumberInline8(num)) => {
                    stmts.push(Stmt::Generic(GenericStmt::LeNumLConstR(num), instr_ip));
                }
                IntrinsicKind::LeNumLConstR32((), NumberInline32(num)) => {
                    stmts.push(Stmt::Generic(GenericStmt::LeNumLConstR32(num), instr_ip));
                }
                _ => unimplemented!("Intrinsic {kind:?} at {instr_ip:?} is not implemented"),
            },
            InstructionOperands::Number(NumberConstantOperands(NumberConstantWide(num))) => {
                stmts.push(Stmt::Generic(GenericStmt::Number(num), instr_ip));
            }
            InstructionOperands::StoreLocal(StoreLocalOperands { kind, local }) => {
                stmts.push(Stmt::Generic(
                    GenericStmt::StoreLocal {
                        kind: kind.into(),
                        local,
                    },
                    instr_ip,
                ));
            }
            InstructionOperands::Pop(PopOperands(())) => {
                stmts.push(Stmt::Generic(GenericStmt::Pop, instr_ip));
            }
            InstructionOperands::Add(AddOperands(BinaryOperator { left: (), right: () })) => {
                stmts.push(Stmt::Generic(GenericStmt::Add, instr_ip));
            }
            InstructionOperands::Lt(LtOperands(BinaryOperator { left: (), right: () })) => {
                stmts.push(Stmt::Generic(GenericStmt::Lt, instr_ip));
            }
            InstructionOperands::Mul(MulOperands(BinaryOperator { left: (), right: () })) => {
                stmts.push(Stmt::Generic(GenericStmt::Mul, instr_ip));
            }
            InstructionOperands::Sub(SubOperands(BinaryOperator { left: (), right: () })) => {
                stmts.push(Stmt::Generic(GenericStmt::Sub, instr_ip));
            }
            InstructionOperands::Div(DivOperands(BinaryOperator { left: (), right: () })) => {
                stmts.push(Stmt::Generic(GenericStmt::Div, instr_ip));
            }
            InstructionOperands::Rem(RemOperands(BinaryOperator { left: (), right: () })) => {
                stmts.push(Stmt::Generic(GenericStmt::Rem, instr_ip));
            }
            InstructionOperands::Pow(PowOperands(BinaryOperator { left: (), right: () })) => {
                stmts.push(Stmt::Generic(GenericStmt::Pow, instr_ip));
            }
            InstructionOperands::Gt(GtOperands(BinaryOperator { left: (), right: () })) => {
                stmts.push(Stmt::Generic(GenericStmt::Gt, instr_ip));
            }
            InstructionOperands::Ge(GeOperands(BinaryOperator { left: (), right: () })) => {
                stmts.push(Stmt::Generic(GenericStmt::Ge, instr_ip));
            }
            InstructionOperands::Le(LeOperands(BinaryOperator { left: (), right: () })) => {
                stmts.push(Stmt::Generic(GenericStmt::Le, instr_ip));
            }
            InstructionOperands::Eq(EqOperands(BinaryOperator { left: (), right: () })) => {
                stmts.push(Stmt::Generic(GenericStmt::Eq, instr_ip));
            }
            InstructionOperands::Ne(NeOperands(BinaryOperator { left: (), right: () })) => {
                stmts.push(Stmt::Generic(GenericStmt::Ne, instr_ip));
            }
            InstructionOperands::StrictEq(StrictEqOperands(BinaryOperator { left: (), right: () })) => {
                stmts.push(Stmt::Generic(GenericStmt::StrictEq, instr_ip));
            }
            InstructionOperands::StrictNe(StrictNeOperands(BinaryOperator { left: (), right: () })) => {
                stmts.push(Stmt::Generic(GenericStmt::StrictNe, instr_ip));
            }
            InstructionOperands::BitOr(BitorOperands(BinaryOperator { left: (), right: () })) => {
                stmts.push(Stmt::Generic(GenericStmt::BitOr, instr_ip));
            }
            InstructionOperands::BitXor(BitxorOperands(BinaryOperator { left: (), right: () })) => {
                stmts.push(Stmt::Generic(GenericStmt::BitXor, instr_ip));
            }
            InstructionOperands::BitAnd(BitandOperands(BinaryOperator { left: (), right: () })) => {
                stmts.push(Stmt::Generic(GenericStmt::BitAnd, instr_ip));
            }
            InstructionOperands::BitShl(BitshlOperands(BinaryOperator { left: (), right: () })) => {
                stmts.push(Stmt::Generic(GenericStmt::BitShl, instr_ip));
            }
            InstructionOperands::BitShr(BitshrOperands(BinaryOperator { left: (), right: () })) => {
                stmts.push(Stmt::Generic(GenericStmt::BitShr, instr_ip));
            }
            InstructionOperands::BitUshr(BitushrOperands(BinaryOperator { left: (), right: () })) => {
                stmts.push(Stmt::Generic(GenericStmt::BitUshr, instr_ip));
            }
            InstructionOperands::ObjIn(ObjectInOperands { target: (), key: () }) => {
                stmts.push(Stmt::Generic(GenericStmt::ObjIn, instr_ip));
            }
            InstructionOperands::InstanceOf(InstanceofOperands {
                constructor: (),
                value: (),
            }) => {
                stmts.push(Stmt::Generic(GenericStmt::InstanceOf, instr_ip));
            }
            InstructionOperands::Pos(PosOperands(())) => {
                stmts.push(Stmt::Generic(GenericStmt::Pos, instr_ip));
            }
            InstructionOperands::Neg(NegOperands(())) => {
                stmts.push(Stmt::Generic(GenericStmt::Neg, instr_ip));
            }
            InstructionOperands::Not(NotOperands(())) => {
                stmts.push(Stmt::Generic(GenericStmt::Not, instr_ip));
            }
            InstructionOperands::BitNot(BitnotOperands(())) => {
                stmts.push(Stmt::Generic(GenericStmt::BitNot, instr_ip));
            }
            InstructionOperands::TypeOf(TypeofOperands { value: () }) => {
                stmts.push(Stmt::Generic(GenericStmt::TypeOf, instr_ip));
            }
            InstructionOperands::CallSymbolIterator(CallSymbolIteratorOperands { value: () }) => {
                stmts.push(Stmt::Generic(GenericStmt::CallSymbolIterator, instr_ip));
            }
            InstructionOperands::CallForInIterator(ForInIteratorOperands { value: () }) => {
                stmts.push(Stmt::Generic(GenericStmt::CallForInIterator, instr_ip));
            }
            InstructionOperands::DeletePropertyStatic(DeletePropertyStaticOperands {
                target: (),
                key: SymbolConstantWide(_),
            }) => {
                stmts.push(Stmt::Generic(GenericStmt::DeletePropertyStatic, instr_ip));
            }
            InstructionOperands::DeletePropertyDynamic(DeletePropertyDynamicOperands { target: (), key: () }) => {
                stmts.push(Stmt::Generic(GenericStmt::DeletePropertyDynamic, instr_ip));
            }
            InstructionOperands::String(StringConstantOperands(SymbolConstantWide(_))) => {
                stmts.push(Stmt::Generic(GenericStmt::String, instr_ip));
            }
            InstructionOperands::Boolean(BooleanConstantOperands(BooleanConstantWide(_))) => {
                stmts.push(Stmt::Generic(GenericStmt::Boolean, instr_ip));
            }
            InstructionOperands::Regex(RegexConstantOperands(_)) => {
                stmts.push(Stmt::Generic(GenericStmt::Regex, instr_ip));
            }
            InstructionOperands::Function(FunctionConstantOperands(_)) => {
                stmts.push(Stmt::Generic(GenericStmt::Function, instr_ip));
            }
            InstructionOperands::Null => {
                stmts.push(Stmt::Generic(GenericStmt::Null, instr_ip));
            }
            InstructionOperands::Undefined => {
                stmts.push(Stmt::Generic(GenericStmt::Undefined, instr_ip));
            }
            InstructionOperands::Undef => {
                stmts.push(Stmt::Generic(GenericStmt::Undef, instr_ip));
            }
            InstructionOperands::Nan => {
                stmts.push(Stmt::Generic(GenericStmt::Nan, instr_ip));
            }
            InstructionOperands::Infinity => {
                stmts.push(Stmt::Generic(GenericStmt::Infinity, instr_ip));
            }
            InstructionOperands::Global => {
                stmts.push(Stmt::Generic(GenericStmt::Global, instr_ip));
            }
            InstructionOperands::This => {
                stmts.push(Stmt::Generic(GenericStmt::This, instr_ip));
            }
            InstructionOperands::NewTarget => {
                stmts.push(Stmt::Generic(GenericStmt::NewTarget, instr_ip));
            }
            InstructionOperands::LdGlobal(LdGlobalOperands(SymbolConstantWide(_))) => {
                stmts.push(Stmt::Generic(GenericStmt::LdGlobal, instr_ip));
            }
            InstructionOperands::TypeOfGlobalIdent(TypeofIdentOperands(SymbolConstantWide(_))) => {
                stmts.push(Stmt::Generic(GenericStmt::TypeOfGlobalIdent, instr_ip));
            }
            InstructionOperands::LdLocalExt(LdLocalExtOperands(_)) => {
                stmts.push(Stmt::Generic(GenericStmt::LdLocalExt, instr_ip));
            }
            InstructionOperands::Nop => {
                stmts.push(Stmt::Generic(GenericStmt::Nop, instr_ip));
            }
            InstructionOperands::Debugger => {
                stmts.push(Stmt::Generic(GenericStmt::Debugger, instr_ip));
            }
            InstructionOperands::ExportNamed(ExportNamedOperands { members: _ }) => {
                stmts.push(Stmt::Generic(GenericStmt::ExportNamed, instr_ip));
            }
            InstructionOperands::ImportStatic(ImportStaticOperands {
                kind: _,
                local,
                path: SymbolConstantWide(_),
            }) => {
                stmts.push(Stmt::Generic(GenericStmt::ImportStatic { local }, instr_ip));
            }
            InstructionOperands::BindThis(BindThisOperands { value: () }) => {
                stmts.push(Stmt::Generic(GenericStmt::BindThis, instr_ip));
            }
            InstructionOperands::ExportDefault(ExportDefaultOperands { value: () }) => {
                stmts.push(Stmt::Generic(GenericStmt::ExportDefault, instr_ip));
            }
            InstructionOperands::ImportDyn(ImportDynOperands { value: () }) => {
                stmts.push(Stmt::Generic(GenericStmt::ImportDyn, instr_ip));
            }
            InstructionOperands::Throw(ThrowOperands { value: () }) => {
                stmts.push(Stmt::Generic(GenericStmt::Throw, instr_ip));
            }
            InstructionOperands::Call(CallOperands {
                argc,
                has_this,
                function_call_kind,
                spread_indices: _,
            }) => {
                stmts.push(Stmt::Generic(
                    GenericStmt::Call {
                        argc,
                        has_this,
                        kind: function_call_kind,
                    },
                    instr_ip,
                ));
            }
            InstructionOperands::StoreGlobal(StoreGlobalOperands {
                name: SymbolConstantWide(_),
                kind,
            }) => {
                stmts.push(Stmt::Generic(GenericStmt::StoreGlobal { kind: kind.into() }, instr_ip));
            }
            InstructionOperands::StoreLocalExt(StoreLocalExtOperands { local: _, kind }) => {
                stmts.push(Stmt::Generic(
                    GenericStmt::StoreLocalExt { kind: kind.into() },
                    instr_ip,
                ));
            }
            InstructionOperands::StaticPropAssign(StaticPropertyAssignOperands {
                kind,
                target: (),
                key: SymbolConstantWide(_),
            }) => {
                stmts.push(Stmt::Generic(
                    GenericStmt::StaticPropAssign { kind: kind.into() },
                    instr_ip,
                ));
            }
            InstructionOperands::DynamicPropAssign(DynamicPropertyAssignOperands {
                kind,
                key: (),
                target: (),
            }) => {
                stmts.push(Stmt::Generic(
                    GenericStmt::DynamicPropAssign { kind: kind.into() },
                    instr_ip,
                ));
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
            })) => {
                terminator = Some(create_terminator(
                    bytecode,
                    blocks,
                    ip_to_block,
                    scope,
                    &mut cx,
                    offset,
                    ConditionalGotoPopPolicy::Peek,
                    ConditionalGotoCondition::IsFalsy,
                ));
                break;
            }
            InstructionOperands::JmpTrueNP(JmpTrueNoPopOperands(ConditionalJumpNoPopOperands { offset, value: _ })) => {
                terminator = Some(create_terminator(
                    bytecode,
                    blocks,
                    ip_to_block,
                    scope,
                    &mut cx,
                    offset,
                    ConditionalGotoPopPolicy::Peek,
                    ConditionalGotoCondition::IsTruthy,
                ));
                break;
            }
            InstructionOperands::JmpNullishNP(JmpNullishNoPopOperands(ConditionalJumpNoPopOperands {
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
                    ConditionalGotoPopPolicy::Peek,
                    ConditionalGotoCondition::IsNullish,
                ));
                break;
            }
            InstructionOperands::JmpUndefinedNP(JmpUndefinedNoPopOperands(ConditionalJumpNoPopOperands {
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
                    ConditionalGotoPopPolicy::Peek,
                    ConditionalGotoCondition::IsUndefined,
                ));
                break;
            }
            InstructionOperands::JmpFalseP(JmpFalsePopOperands(ConditionalJumpPopOperands { offset, value: _ })) => {
                terminator = Some(create_terminator(
                    bytecode,
                    blocks,
                    ip_to_block,
                    scope,
                    &mut cx,
                    offset,
                    ConditionalGotoPopPolicy::Pop,
                    ConditionalGotoCondition::IsFalsy,
                ));
                break;
            }
            InstructionOperands::JmpTrueP(JmpTruePopOperands(ConditionalJumpPopOperands { offset, value: _ })) => {
                terminator = Some(create_terminator(
                    bytecode,
                    blocks,
                    ip_to_block,
                    scope,
                    &mut cx,
                    offset,
                    ConditionalGotoPopPolicy::Pop,
                    ConditionalGotoCondition::IsTruthy,
                ));
                break;
            }
            InstructionOperands::JmpNullishP(JmpNullishPopOperands(ConditionalJumpPopOperands {
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
                    ConditionalGotoPopPolicy::Pop,
                    ConditionalGotoCondition::IsNullish,
                ));
                break;
            }
            InstructionOperands::JmpUndefinedP(JmpUndefinedPopOperands(ConditionalJumpPopOperands {
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
                    ConditionalGotoPopPolicy::Pop,
                    ConditionalGotoCondition::IsUndefined,
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
