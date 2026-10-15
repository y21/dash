use std::collections::hash_map::Entry;
use std::convert::Infallible;
use std::ops::{BitOr, BitOrAssign};

use ahash::HashMapExt;
use arrayvec::ArrayVec;
use dash_middle::compiler::constant::ConstantPool;
use dash_middle::compiler::extract::{ExtractBack, ExtractFront, ExtractSource, ForwardSequence};
use dash_middle::compiler::operands::{
    AssignKind, ConditionalJumpNoPopOperands, ConditionalJumpPopOperands, InstructionOperands,
    InstructionSuccessorOffset, IntrinsicKind, IntrinsicOperands, JmpFalseNoPopOperands, JmpFalsePopOperands,
    JmpNullishNoPopOperands, JmpNullishPopOperands, JmpOperands, JmpTrueNoPopOperands, JmpTruePopOperands,
    JmpUndefinedNoPopOperands, JmpUndefinedPopOperands, LdLocalOperands, LoopBackjumpOperands, NumberConstantOperands,
    NumberConstantWide, StoreLocalOperands,
};
use dash_middle::compiler::scope::BackLocalId;
use dash_middle::indexvec::IndexVec;
use rustc_hash::FxHashMap;

use crate::frame::Ip;
use crate::jit::BufferRef;
use crate::jit::basic_blocks::BlockStartSet;
use crate::localscope::LocalScope;
use crate::value::{Unpack, ValueKind};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum ValueType {
    Number,
    Boolean,
    /// The bottom value, no assumptions possible
    Any,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct ValueMetadata {
    ty: ValueType,
}

#[derive(Debug, Copy, Clone)]
enum JoinChange {
    Unchanged,
    Changed,
}

impl BitOr for JoinChange {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (JoinChange::Unchanged, JoinChange::Unchanged) => JoinChange::Unchanged,
            _ => JoinChange::Changed,
        }
    }
}
impl BitOrAssign for JoinChange {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

trait Join: Sized {
    fn join(&self, other: &Self) -> (Self, JoinChange);
}

fn join_slice<T: Join + Clone, C: FromIterator<T>>(a: &[T], b: &[T]) -> (C, JoinChange) {
    let mut change = JoinChange::Unchanged;
    let collection = a
        .iter()
        .zip(b.iter())
        .map(|(a, b)| {
            let (out, ch) = a.join(b);
            change |= ch;
            out
        })
        .collect();

    (collection, change)
}

impl Join for ValueMetadata {
    fn join(&self, other: &ValueMetadata) -> (ValueMetadata, JoinChange) {
        match (self.ty, other.ty) {
            (ValueType::Any, ValueType::Any)
            | (ValueType::Number, ValueType::Number)
            | (ValueType::Boolean, ValueType::Boolean) => (*self, JoinChange::Unchanged),
            _ => (ValueMetadata { ty: ValueType::Any }, JoinChange::Changed),
        }
    }
}

impl ExtractBack<ValueMetadataExtractor<'_, '_>> for ValueMetadata {
    type Exception = Infallible;

    fn extract_back(source: &mut ValueMetadataExtractor<'_, '_>) -> Result<Self, Self::Exception> {
        Ok(source.pop_stack())
    }
}

impl ExtractFront<ValueMetadataExtractor<'_, '_>> for ValueMetadata {
    type Exception = Infallible;

    fn extract_front<U>(
        cx: &mut ValueMetadataExtractor<'_, '_>,
        seq: &mut ForwardSequence<U>,
    ) -> Result<Self, Self::Exception> {
        let idx = seq.next_stack_index();
        Ok(cx.state.stack[idx])
    }
}

#[derive(Debug)]
struct ValueMetadataExtractor<'a, 'b> {
    state: BlockState,
    scope: &'a LocalScope<'b>,
    full_bytecode: BufferRef<'a>,
    remaining_bytecode: BufferRef<'a>,
    start_ip: Ip,
}

impl<'a, 'b> ValueMetadataExtractor<'a, 'b> {
    pub fn ip(&self) -> Ip {
        Ip(self.start_ip.0 + (self.full_bytecode.len() - self.remaining_bytecode.len()))
    }

    pub fn push_stack(&mut self, value: ValueMetadata) {
        self.state.stack.push(value);
    }
}

impl<'a, 'b> ExtractSource for ValueMetadataExtractor<'a, 'b> {
    type Value = ValueMetadata;

    type Unrooted = ValueMetadata;

    fn fetch_bytes<const N: usize>(&mut self) -> [u8; N] {
        let (&bytes, rest) = self.remaining_bytecode.as_slice().split_first_chunk().unwrap();
        self.remaining_bytecode = rest.into();
        bytes
    }

    fn constants(&self) -> &ConstantPool {
        &self.scope.frames.current_fn().constants
    }

    fn stack_len(&self) -> usize {
        self.state.stack.len()
    }

    fn pop_stack_rooted(&mut self) -> Self::Value {
        self.state.stack.pop().unwrap()
    }

    fn pop_stack(&mut self) -> Self::Unrooted {
        self.state.stack.pop().unwrap()
    }

    fn peek_stack_rooted(&mut self) -> Self::Value {
        *self.state.stack.last().unwrap()
    }

    fn peek_stack(&self) -> Self::Unrooted {
        *self.state.stack.last().unwrap()
    }

    fn truncate_stack(&mut self, len: usize) {
        self.state.stack.truncate(len);
    }
}

impl Iterator for ValueMetadataExtractor<'_, '_> {
    type Item = (Ip, InstructionOperands<Self>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_bytecode.is_empty() {
            return None;
        }

        let ip = self.ip();
        let Ok(operands) = InstructionOperands::extract_back(self);
        Some((ip, operands))
    }
}

#[derive(Debug, Clone)]
pub struct BlockState {
    // TODO: bump allocate these
    stack: Vec<ValueMetadata>,
    locals: IndexVec<ValueMetadata, BackLocalId>,
}

impl Join for BlockState {
    fn join(&self, other: &BlockState) -> (BlockState, JoinChange) {
        assert_eq!(self.stack.len(), other.stack.len());

        let (stack, mut changed) = join_slice(&self.stack, &other.stack);

        let (locals, locals_changed) = join_slice(&self.locals, &other.locals);
        changed |= locals_changed;

        (Self { stack, locals }, changed)
    }
}

#[derive(Debug)]
pub struct DataflowResult {
    entry_states: FxHashMap<Ip, BlockState>,
}

pub fn run_analysis(bytecode: BufferRef<'_>, scope: &LocalScope<'_>, block_starts: &BlockStartSet) -> DataflowResult {
    let mut queue = vec![Ip::ZERO];

    let initial_local_state: IndexVec<ValueMetadata, BackLocalId> = {
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
    };

    let mut entry_states = FxHashMap::with_capacity(block_starts.len());
    entry_states.insert(
        Ip::ZERO,
        BlockState {
            stack: Vec::new(),
            locals: initial_local_state,
        },
    );

    let end_of_chunk = Ip(bytecode.len());

    while let Some(block) = queue.pop() {
        let state = entry_states[&block].clone();
        let block_bytecode = bytecode.slice(block..);

        let mut cx = ValueMetadataExtractor {
            start_ip: block,
            state,
            scope,
            full_bytecode: block_bytecode,
            remaining_bytecode: block_bytecode,
        };

        let mut succ_ips: ArrayVec<_, 2> = ArrayVec::new();

        while let Some((ip, instr)) = cx.next() {
            if ip != block && block_starts.contains(&ip) {
                // This instruction belongs to the next block, so stop here.
                succ_ips.push(ip);
                break;
            }

            if let Some(successor) = handle_instruction(&mut cx, instr) {
                match successor {
                    InstructionSuccessorOffset::Unconditional(offset) => succ_ips.push(cx.ip() + offset),
                    InstructionSuccessorOffset::Conditional(offset) => {
                        succ_ips.push(cx.ip());
                        succ_ips.push(cx.ip() + offset);
                    }
                };
                break;
            }
        }

        for succ_ip in succ_ips {
            match entry_states.entry(succ_ip) {
                Entry::Occupied(mut entry) => {
                    let (new_state, change) = entry.get().join(&cx.state);
                    if let JoinChange::Changed = change {
                        entry.insert(new_state);
                        queue.push(succ_ip);
                    }
                }
                Entry::Vacant(entry) => {
                    if succ_ip == end_of_chunk {
                        // Jumping to the end of the chunk, no need to insert into entry_states.
                    } else {
                        // No need to join, just insert the new state.
                        assert!(block_starts.contains(&succ_ip));
                        entry.insert(cx.state.clone());
                        queue.push(succ_ip);
                    }
                }
            }
        }
    }

    DataflowResult { entry_states }
}

fn handle_instruction(
    cx: &mut ValueMetadataExtractor<'_, '_>,
    instr: InstructionOperands<ValueMetadataExtractor<'_, '_>>,
) -> Option<InstructionSuccessorOffset> {
    match instr {
        InstructionOperands::LdLocal(LdLocalOperands(local)) => cx.push_stack(cx.state.locals[local].clone()),
        InstructionOperands::IntrinsicOp(IntrinsicOperands(operands)) => match operands {
            IntrinsicKind::LtNumLConstR32(_, _) => cx.push_stack(ValueMetadata { ty: ValueType::Boolean }),
            IntrinsicKind::PostfixIncLocalNum(_) => cx.push_stack(ValueMetadata { ty: ValueType::Number }),
            _ => todo!("{operands:?}"),
        },
        InstructionOperands::Number(NumberConstantOperands(NumberConstantWide(_))) => {
            cx.push_stack(ValueMetadata { ty: ValueType::Number })
        }
        InstructionOperands::StoreLocal(StoreLocalOperands { kind, local }) => {
            let out_ty = match kind {
                AssignKind::Assignment(ty) => {
                    cx.state.locals[local] = ty;
                    ty
                }
                AssignKind::AddAssignment(rhs)
                | AssignKind::SubAssignment(rhs)
                | AssignKind::MulAssignment(rhs)
                | AssignKind::DivAssignment(rhs)
                | AssignKind::RemAssignment(rhs)
                | AssignKind::PowAssignment(rhs)
                | AssignKind::ShlAssignment(rhs)
                | AssignKind::ShrAssignment(rhs)
                | AssignKind::UshrAssignment(rhs)
                | AssignKind::BitAndAssignment(rhs)
                | AssignKind::BitOrAssignment(rhs)
                | AssignKind::BitXorAssignment(rhs) => {
                    // For now just handle the trivial, common case that we can actually make use of: lhs and rhs are both numbers
                    // Otherwise fall back to any
                    let metadata = if cx.state.locals[local] == rhs && rhs.ty == ValueType::Number {
                        ValueMetadata { ty: ValueType::Number }
                    } else {
                        ValueMetadata { ty: ValueType::Any }
                    };
                    cx.state.locals[local] = metadata;
                    metadata
                }
                AssignKind::PrefixIncrement
                | AssignKind::PostfixIncrement
                | AssignKind::PrefixDecrement
                | AssignKind::PostfixDecrement => {
                    let metadata = if cx.state.locals[local].ty == ValueType::Number {
                        ValueMetadata { ty: ValueType::Number }
                    } else {
                        ValueMetadata { ty: ValueType::Any }
                    };
                    cx.state.locals[local] = metadata;
                    metadata
                }
            };
            cx.push_stack(out_ty);
        }
        InstructionOperands::Pop(_) => {}
        InstructionOperands::Jmp(JmpOperands(offset))
        | InstructionOperands::LoopBackJmp(LoopBackjumpOperands { offset, hotness: _ }) => {
            return Some(InstructionSuccessorOffset::Unconditional(offset));
        }
        InstructionOperands::JmpFalseNP(JmpFalseNoPopOperands(ConditionalJumpNoPopOperands { offset, value: _ }))
        | InstructionOperands::JmpTrueNP(JmpTrueNoPopOperands(ConditionalJumpNoPopOperands { offset, value: _ }))
        | InstructionOperands::JmpNullishNP(JmpNullishNoPopOperands(ConditionalJumpNoPopOperands {
            offset,
            value: _,
        }))
        | InstructionOperands::JmpUndefinedNP(JmpUndefinedNoPopOperands(ConditionalJumpNoPopOperands {
            offset,
            value: _,
        }))
        | InstructionOperands::JmpFalseP(JmpFalsePopOperands(ConditionalJumpPopOperands { offset, value: _ }))
        | InstructionOperands::JmpTrueP(JmpTruePopOperands(ConditionalJumpPopOperands { offset, value: _ }))
        | InstructionOperands::JmpNullishP(JmpNullishPopOperands(ConditionalJumpPopOperands { offset, value: _ }))
        | InstructionOperands::JmpUndefinedP(JmpUndefinedPopOperands(ConditionalJumpPopOperands {
            offset,
            value: _,
        })) => {
            return Some(InstructionSuccessorOffset::Conditional(offset));
        }
        _ => todo!("{instr:?}"),
    };
    None
}
