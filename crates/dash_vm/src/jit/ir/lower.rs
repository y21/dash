use std::collections::hash_map::Entry;

use dash_middle::compiler::instruction::Instruction;
use dash_middle::compiler::operands::{
    AddOperands, AssignKind, BinaryOperator, BindThisOperands, BitandOperands, BitnotOperands, BitorOperands,
    BitshlOperands, BitshrOperands, BitushrOperands, BitxorOperands, BooleanConstantOperands, BooleanConstantWide,
    CallOperands, CallSymbolIteratorOperands, ConditionalJumpNoPopOperands, ConditionalJumpPopOperands,
    DeletePropertyDynamicOperands, DeletePropertyStaticOperands, DivOperands, DynamicPropertyAssignOperands,
    EqOperands, ExportDefaultOperands, ExportNamedOperands, ForInIteratorOperands, FunctionConstantOperands,
    GeOperands, GtOperands, ImportDynOperands, ImportStaticOperands, InstanceofOperands, InstructionOperands,
    IntrinsicKind, IntrinsicOperands, JmpFalseNoPopOperands, JmpFalsePopOperands, JmpNullishNoPopOperands,
    JmpNullishPopOperands, JmpOperands, JmpTrueNoPopOperands, JmpTruePopOperands, JmpUndefinedNoPopOperands,
    JmpUndefinedPopOperands, LdGlobalOperands, LdLocalExtOperands, LdLocalOperands, LeOperands, LoopBackjumpOperands,
    LtOperands, MulOperands, NeOperands, NegOperands, NotOperands, NumberConstantOperands, NumberConstantWide,
    NumberInline8, NumberInline32, ObjectInOperands, OptionDiscriminatedByte, PopOperands, PosOperands, PowOperands,
    RegexConstantOperands, RemOperands, StaticPropertyAssignOperands, StoreGlobalOperands, StoreLocalExtOperands,
    StoreLocalOperands, StrictEqOperands, StrictNeOperands, StringConstantOperands, SubOperands, SymbolConstantWide,
    ThrowOperands, TypeofIdentOperands, TypeofOperands,
};
use dash_middle::compiler::scope::BackLocalId;
use dash_middle::indexvec::IndexVec;
use rustc_hash::FxHashMap;

use crate::frame::Ip;
use crate::jit::dataflow::value::ValueType;
use crate::jit::dataflow::{LocalValues, values_for_locals};
use crate::jit::ir::{
    BasicBlock, BasicBlockId, Body, BranchTarget, ConditionalGotoCondition, ConditionalGotoPopPolicy, GenericStmt,
    Stmt, Terminator, UncheckedAssignmentKind, UncheckedNumberBinOp, ValueSource,
};
use crate::jit::passes::{Assumption, Assumptions};
use crate::jit::{BufferRef, CompileError, NoValueExtractContext};
use crate::localscope::LocalScope;

struct LowerCtxt {
    ip_to_block: FxHashMap<Ip, BasicBlockId>,
    assumptions: Assumptions,
    blocks: IndexVec<BasicBlock, BasicBlockId>,
    local_values: LocalValues,
}

impl LowerCtxt {
    fn record_local_type_assumption(&mut self, local: BackLocalId) {
        let ty = self.local_values[local].ty;
        if ty != ValueType::Any {
            self.assumptions.insert(Assumption::LocalHasType { local, ty });
        }
    }
}

fn create_terminator(
    bytecode: BufferRef<'_>,
    cx: &mut LowerCtxt,
    scope: &LocalScope<'_>,
    ecx: &mut NoValueExtractContext<'_, '_>,
    offset: i16,
    pop: ConditionalGotoPopPolicy,
    condition: ConditionalGotoCondition,
) -> Result<Terminator, CompileError> {
    let true_target = ecx.ip() + offset;
    let false_target = ecx.ip();

    let true_ = lower_to_branch_target(bytecode, true_target, cx, scope)?;
    let false_ = lower_to_branch_target(bytecode, false_target, cx, scope)?;
    Ok(Terminator::ConditionalGoto {
        true_,
        false_,
        pop,
        condition,
    })
}

fn lower_to_branch_target(
    bytecode: BufferRef<'_>,
    ip: Ip,
    cx: &mut LowerCtxt,
    scope: &LocalScope<'_>,
) -> Result<BranchTarget, CompileError> {
    if ip.0 == bytecode.len() {
        Ok(BranchTarget::Break)
    } else {
        Ok(BranchTarget::Block(lower_from_ip(bytecode, ip, cx, scope)?))
    }
}

fn lower_from_ip(
    bytecode: BufferRef<'_>,
    ip: Ip,
    cx: &mut LowerCtxt,
    scope: &LocalScope<'_>,
) -> Result<BasicBlockId, CompileError> {
    let block = match cx.ip_to_block.entry(ip) {
        Entry::Occupied(entry) => {
            return Ok(*entry.get());
        }
        Entry::Vacant(entry) => {
            let block = cx.blocks.push(BasicBlock::dummy());
            entry.insert(block);
            block
        }
    };

    let mut stmts = IndexVec::new();
    let mut terminator = None;

    let mut ecx = NoValueExtractContext::new(bytecode.as_slice(), ip, scope);
    while let Some((instr_ip, instr)) = ecx.next_instruction_with_operands() {
        match instr {
            InstructionOperands::LdLocal(LdLocalOperands(local)) => {
                cx.record_local_type_assumption(local);
                stmts.push(Stmt::LdLocal(local));
            }
            InstructionOperands::IntrinsicOp(IntrinsicOperands(kind)) => match kind {
                IntrinsicKind::LtNumLConstR32((), NumberInline32(num))
                | IntrinsicKind::LtNumLConstR((), NumberInline8(num)) => {
                    stmts.push(Stmt::UncheckedNumberBinOp {
                        op: UncheckedNumberBinOp::Lt,
                        lhs: ValueSource::Stack,
                        rhs: ValueSource::Float(num),
                    });
                }
                IntrinsicKind::PostfixIncLocalNum(local) => {
                    stmts.push(Stmt::UncheckedStoreLocal {
                        kind: UncheckedAssignmentKind::NumberPostfixIncrement,
                        local,
                    });
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
                    stmts.push(Stmt::UncheckedNumberBinOp {
                        op: UncheckedNumberBinOp::Add,
                        lhs: ValueSource::Stack,
                        rhs: ValueSource::Stack,
                    });
                }
                IntrinsicKind::SubNumLR((), ()) => {
                    stmts.push(Stmt::UncheckedNumberBinOp {
                        op: UncheckedNumberBinOp::Sub,
                        lhs: ValueSource::Stack,
                        rhs: ValueSource::Stack,
                    });
                }
                IntrinsicKind::MulNumLR((), ()) => {
                    stmts.push(Stmt::UncheckedNumberBinOp {
                        op: UncheckedNumberBinOp::Mul,
                        lhs: ValueSource::Stack,
                        rhs: ValueSource::Stack,
                    });
                }
                IntrinsicKind::DivNumLR((), ()) => {
                    stmts.push(Stmt::UncheckedNumberBinOp {
                        op: UncheckedNumberBinOp::Div,
                        lhs: ValueSource::Stack,
                        rhs: ValueSource::Stack,
                    });
                }
                IntrinsicKind::RemNumLR((), ()) => {
                    stmts.push(Stmt::Generic(GenericStmt::RemNumLR, instr_ip));
                }
                IntrinsicKind::PowNumLR((), ()) => {
                    stmts.push(Stmt::Generic(GenericStmt::PowNumLR, instr_ip));
                }
                IntrinsicKind::GtNumLR((), ()) => {
                    stmts.push(Stmt::UncheckedNumberBinOp {
                        op: UncheckedNumberBinOp::Gt,
                        lhs: ValueSource::Stack,
                        rhs: ValueSource::Stack,
                    });
                }
                IntrinsicKind::GeNumLR((), ()) => {
                    stmts.push(Stmt::UncheckedNumberBinOp {
                        op: UncheckedNumberBinOp::Ge,
                        lhs: ValueSource::Stack,
                        rhs: ValueSource::Stack,
                    });
                }
                IntrinsicKind::LtNumLR((), ()) => {
                    stmts.push(Stmt::UncheckedNumberBinOp {
                        op: UncheckedNumberBinOp::Lt,
                        lhs: ValueSource::Stack,
                        rhs: ValueSource::Stack,
                    });
                }
                IntrinsicKind::LeNumLR((), ()) => {
                    stmts.push(Stmt::UncheckedNumberBinOp {
                        op: UncheckedNumberBinOp::Le,
                        lhs: ValueSource::Stack,
                        rhs: ValueSource::Stack,
                    });
                }
                IntrinsicKind::EqNumLR((), ()) => {
                    stmts.push(Stmt::UncheckedNumberBinOp {
                        op: UncheckedNumberBinOp::Eq,
                        lhs: ValueSource::Stack,
                        rhs: ValueSource::Stack,
                    });
                }
                IntrinsicKind::NeNumLR((), ()) => {
                    stmts.push(Stmt::UncheckedNumberBinOp {
                        op: UncheckedNumberBinOp::Ne,
                        lhs: ValueSource::Stack,
                        rhs: ValueSource::Stack,
                    });
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
                IntrinsicKind::GtNumLConstR((), NumberInline8(num))
                | IntrinsicKind::GtNumLConstR32((), NumberInline32(num)) => {
                    stmts.push(Stmt::UncheckedNumberBinOp {
                        op: UncheckedNumberBinOp::Gt,
                        lhs: ValueSource::Stack,
                        rhs: ValueSource::Float(num),
                    });
                }
                IntrinsicKind::GeNumLConstR((), NumberInline8(num))
                | IntrinsicKind::GeNumLConstR32((), NumberInline32(num)) => {
                    stmts.push(Stmt::UncheckedNumberBinOp {
                        op: UncheckedNumberBinOp::Ge,
                        lhs: ValueSource::Stack,
                        rhs: ValueSource::Float(num),
                    });
                }
                IntrinsicKind::LeNumLConstR((), NumberInline8(num))
                | IntrinsicKind::LeNumLConstR32((), NumberInline32(num)) => {
                    stmts.push(Stmt::UncheckedNumberBinOp {
                        op: UncheckedNumberBinOp::Le,
                        lhs: ValueSource::Stack,
                        rhs: ValueSource::Float(num),
                    });
                }
                _ => unimplemented!("Intrinsic {kind:?} at {instr_ip:?} is not implemented"),
            },
            InstructionOperands::Number(NumberConstantOperands(NumberConstantWide(num))) => {
                stmts.push(Stmt::Number(num));
            }
            InstructionOperands::StoreLocal(StoreLocalOperands {
                kind: AssignKind::Assignment(_),
                local,
            }) => {
                stmts.push(Stmt::UncheckedStoreLocal {
                    kind: UncheckedAssignmentKind::Assignment,
                    local,
                });
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
                stmts.push(Stmt::Pop);
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
            InstructionOperands::Boolean(BooleanConstantOperands(BooleanConstantWide(value))) => {
                stmts.push(Stmt::Boolean(value));
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
                stmts.push(Stmt::Number(f64::NAN));
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
            InstructionOperands::ImportDyn(ImportDynOperands {
                value: (),
                options: OptionDiscriminatedByte(_),
            }) => {
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
                let target = ecx.ip() + offset;
                let succ = lower_to_branch_target(bytecode, target, cx, scope)?;
                terminator = Some(Terminator::Goto(succ));
                break;
            }
            InstructionOperands::JmpFalseNP(JmpFalseNoPopOperands(ConditionalJumpNoPopOperands {
                offset,
                value: _,
            })) => {
                terminator = Some(create_terminator(
                    bytecode,
                    cx,
                    scope,
                    &mut ecx,
                    offset,
                    ConditionalGotoPopPolicy::Peek,
                    ConditionalGotoCondition::IsFalsy,
                )?);
                break;
            }
            InstructionOperands::JmpTrueNP(JmpTrueNoPopOperands(ConditionalJumpNoPopOperands { offset, value: _ })) => {
                terminator = Some(create_terminator(
                    bytecode,
                    cx,
                    scope,
                    &mut ecx,
                    offset,
                    ConditionalGotoPopPolicy::Peek,
                    ConditionalGotoCondition::IsTruthy,
                )?);
                break;
            }
            InstructionOperands::JmpNullishNP(JmpNullishNoPopOperands(ConditionalJumpNoPopOperands {
                offset,
                value: _,
            })) => {
                terminator = Some(create_terminator(
                    bytecode,
                    cx,
                    scope,
                    &mut ecx,
                    offset,
                    ConditionalGotoPopPolicy::Peek,
                    ConditionalGotoCondition::IsNullish,
                )?);
                break;
            }
            InstructionOperands::JmpUndefinedNP(JmpUndefinedNoPopOperands(ConditionalJumpNoPopOperands {
                offset,
                value: _,
            })) => {
                terminator = Some(create_terminator(
                    bytecode,
                    cx,
                    scope,
                    &mut ecx,
                    offset,
                    ConditionalGotoPopPolicy::Peek,
                    ConditionalGotoCondition::IsUndefined,
                )?);
                break;
            }
            InstructionOperands::JmpFalseP(JmpFalsePopOperands(ConditionalJumpPopOperands { offset, value: _ })) => {
                terminator = Some(create_terminator(
                    bytecode,
                    cx,
                    scope,
                    &mut ecx,
                    offset,
                    ConditionalGotoPopPolicy::Pop,
                    ConditionalGotoCondition::IsFalsy,
                )?);
                break;
            }
            InstructionOperands::JmpTrueP(JmpTruePopOperands(ConditionalJumpPopOperands { offset, value: _ })) => {
                terminator = Some(create_terminator(
                    bytecode,
                    cx,
                    scope,
                    &mut ecx,
                    offset,
                    ConditionalGotoPopPolicy::Pop,
                    ConditionalGotoCondition::IsTruthy,
                )?);
                break;
            }
            InstructionOperands::JmpNullishP(JmpNullishPopOperands(ConditionalJumpPopOperands {
                offset,
                value: _,
            })) => {
                terminator = Some(create_terminator(
                    bytecode,
                    cx,
                    scope,
                    &mut ecx,
                    offset,
                    ConditionalGotoPopPolicy::Pop,
                    ConditionalGotoCondition::IsNullish,
                )?);
                break;
            }
            InstructionOperands::JmpUndefinedP(JmpUndefinedPopOperands(ConditionalJumpPopOperands {
                offset,
                value: _,
            })) => {
                terminator = Some(create_terminator(
                    bytecode,
                    cx,
                    scope,
                    &mut ecx,
                    offset,
                    ConditionalGotoPopPolicy::Pop,
                    ConditionalGotoCondition::IsUndefined,
                )?);
                break;
            }
            _ => {
                let instr = Instruction::from_repr(bytecode[ip]).unwrap();
                return Err(CompileError::UnimplementedInstruction(instr, ip));
            }
        }
    }

    cx.blocks[block] = BasicBlock { stmts, terminator };
    Ok(block)
}

pub fn lower_body(bytecode: BufferRef<'_>, scope: &LocalScope<'_>) -> Result<Body, CompileError> {
    let local_values = values_for_locals(scope);

    let mut cx = LowerCtxt {
        blocks: IndexVec::new(),
        local_values,
        ip_to_block: FxHashMap::default(),
        assumptions: Assumptions::default(),
    };

    let root = lower_from_ip(bytecode, Ip::ZERO, &mut cx, scope)?;

    Ok(Body {
        blocks: cx.blocks,
        root,
        assumptions: cx.assumptions,
        initial_local_values: cx.local_values,
    })
}
