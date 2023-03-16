use std::collections::HashMap;
use std::collections::HashSet;

use dash_middle::compiler::instruction::Instruction;
use dash_middle::compiler::instruction::IntrinsicOperation;

use crate::error::Error;
use crate::util::DecodeCtxt;

use super::bb_generation::BasicBlockKey;
use super::bb_generation::BasicBlockMap;
use super::bb_generation::BasicBlockSuccessor;
use super::bb_generation::ConditionalBranchAction;

pub type TypeMap = HashMap<u16, Type>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    I64,
    F64,
    Boolean,
}

pub trait TypeInferQuery {
    fn type_of_local(&self, index: u16) -> Type;
    fn type_of_constant(&self, index: u16) -> Type;
}

#[derive(Clone, Default)]
pub struct TypeStack(Vec<Type>);
impl TypeStack {
    fn pop_two(&mut self) -> (Type, Type) {
        let a = self.0.pop().unwrap();
        let b = self.0.pop().unwrap();
        (b, a)
    }

    fn pop(&mut self) -> Type {
        self.0.pop().unwrap()
    }

    fn push(&mut self, ty: Type) {
        self.0.push(ty);
    }
}

pub struct TypeInferCtxt<'a, 'q, Q> {
    pub bytecode: &'a [u8],
    pub bbs: BasicBlockMap,
    pub local_tys: TypeMap,
    pub query: &'q mut Q,
    /// Basic blocks we've already visited,
    /// to prevent getting into an infinite loop
    /// while following BB successors
    pub visited: HashSet<usize>,
}

impl<'a, 'q, Q: TypeInferQuery> TypeInferCtxt<'a, 'q, Q> {
    fn get_or_insert_local_ty(&mut self, index: u16) -> Type {
        match self.local_tys.get(&index) {
            Some(ty) => ty.clone(),
            None => {
                let ty = self.query.type_of_local(index);
                self.local_tys.insert(index, ty.clone());
                ty
            }
        }
    }
    pub fn resolve_types(&mut self, mut ty_stack: TypeStack, bbk: BasicBlockKey) -> Result<(), Error> {
        // If this BB is in the list of visited BBs
        // do not resolve it again
        if self.visited.contains(&bbk) {
            return Ok(());
        }
        self.visited.insert(bbk);

        let (mut dcx, succ, block_offset) = {
            let bb = &self.bbs[&bbk];
            let bytecode = &self.bytecode[bb.index..bb.end];

            (DecodeCtxt::new(bytecode), bb.successor, bb.index)
        };

        while let Some((index, instr)) = dcx.next_instruction() {
            let index = index + block_offset;

            match instr {
                Instruction::Add | Instruction::Sub | Instruction::Mul => match ty_stack.pop_two() {
                    (Type::I64, Type::I64) => ty_stack.push(Type::I64),
                    (Type::I64, Type::F64) => ty_stack.push(Type::F64),
                    (Type::F64, Type::I64) => ty_stack.push(Type::F64),
                    (Type::F64, Type::F64) => ty_stack.push(Type::F64),
                    (Type::Boolean, Type::Boolean) => ty_stack.push(Type::Boolean),
                    _ => todo!(),
                },
                Instruction::Div | Instruction::Rem | Instruction::Pow => match ty_stack.pop_two() {
                    (Type::F64 | Type::I64, Type::F64 | Type::I64) => ty_stack.push(Type::F64),
                    (Type::Boolean, Type::Boolean) => ty_stack.push(Type::Boolean),
                    _ => todo!(),
                },
                Instruction::Gt
                | Instruction::Ge
                | Instruction::Lt
                | Instruction::Le
                | Instruction::Eq
                | Instruction::Ne => {
                    ty_stack.pop_two();
                    ty_stack.push(Type::Boolean);
                }
                Instruction::Pop => drop(ty_stack.pop()),
                Instruction::LdLocal | Instruction::LdLocalW => {
                    let index = match instr {
                        Instruction::LdLocal => dcx.next_byte().into(),
                        Instruction::LdLocalW => dcx.next_wide(),
                        _ => unreachable!(),
                    };

                    let ty = self.get_or_insert_local_ty(index);
                    ty_stack.push(ty);
                }
                Instruction::Constant | Instruction::ConstantW => {
                    let index = match instr {
                        Instruction::Constant => dcx.next_byte().into(),
                        Instruction::ConstantW => dcx.next_wide(),
                        _ => unreachable!(),
                    };

                    let ty = self.query.type_of_constant(index);
                    ty_stack.push(ty);
                }
                Instruction::StoreLocal | Instruction::StoreLocalW => {
                    let index = match instr {
                        Instruction::StoreLocal => dcx.next_byte().into(),
                        Instruction::StoreLocalW => dcx.next_wide(),
                        _ => unreachable!(),
                    };
                    let _kind = dcx.next_byte();

                    let ty = ty_stack.pop();
                    let ty_local = self.get_or_insert_local_ty(index);
                    assert_eq!(ty, ty_local, "type must not change");
                    ty_stack.push(ty);
                    // Do nothing (for now); types cannot (must not) change in JIT
                }
                Instruction::Pos => match ty_stack.pop() {
                    Type::I64 => ty_stack.push(Type::I64),
                    Type::F64 => ty_stack.push(Type::F64),
                    _ => todo!(),
                },
                Instruction::Neg => match ty_stack.pop() {
                    Type::I64 => ty_stack.push(Type::I64),
                    Type::F64 => ty_stack.push(Type::F64),
                    _ => todo!(),
                },
                Instruction::Not => {
                    ty_stack.pop();
                    ty_stack.push(Type::Boolean);
                }
                Instruction::Ret => {
                    dcx.next_wide();
                    ty_stack.pop();
                }
                Instruction::Jmp => {
                    let count = dcx.next_wide_signed();
                    let _target_ip = usize::try_from(index as i16 + count + 3).unwrap();

                    let bb = &self.bbs[&bbk];
                    let Some(BasicBlockSuccessor::Unconditional(succ)) = bb.successor else {
                        panic!("unmatched basic block successor");
                    };
                    self.resolve_types(ty_stack.clone(), succ)?;
                    return Ok(());
                }
                Instruction::StrictEq => todo!(),
                Instruction::StrictNe => todo!(),
                Instruction::JmpFalseP
                | Instruction::JmpFalseNP
                | Instruction::JmpTrueP
                | Instruction::JmpTrueNP
                | Instruction::JmpNullishP
                | Instruction::JmpNullishNP
                | Instruction::JmpUndefinedNP
                | Instruction::JmpUndefinedP => {
                    match instr {
                        Instruction::JmpFalseP
                        | Instruction::JmpNullishP
                        | Instruction::JmpTrueP
                        | Instruction::JmpUndefinedP => {
                            ty_stack.pop();
                        }
                        _ => {}
                    }
                    let count = dcx.next_wide_signed();
                    let _target_ip = usize::try_from(index as i16 + count + 3).unwrap();

                    let bb = &self.bbs[&bbk];
                    let Some(BasicBlockSuccessor::Conditional { true_ip: true_, false_ip: false_, action }) = bb.successor else {
                        panic!("unmatched basic block successor");
                    };

                    if let Some(ConditionalBranchAction::Either | ConditionalBranchAction::Taken) = action {
                        self.resolve_types(ty_stack.clone(), true_)?;
                    }

                    if let Some(ConditionalBranchAction::Either | ConditionalBranchAction::NotTaken) = action {
                        self.resolve_types(ty_stack.clone(), false_)?;
                    }

                    return Ok(());
                }
                Instruction::BitOr
                | Instruction::BitXor
                | Instruction::BitAnd
                | Instruction::BitShl
                | Instruction::BitShr
                | Instruction::BitUshr
                | Instruction::BitNot => {
                    ty_stack.pop();
                    ty_stack.push(Type::I64); // TODO: U/I32 actually
                }
                Instruction::Nan => ty_stack.push(Type::F64),
                Instruction::Infinity => ty_stack.push(Type::F64),
                Instruction::IntrinsicOp => {
                    let op = IntrinsicOperation::from_repr(dcx.next_byte()).unwrap();
                    match op {
                        IntrinsicOperation::AddNumLR
                        | IntrinsicOperation::SubNumLR
                        | IntrinsicOperation::MulNumLR
                        | IntrinsicOperation::PowNumLR => match ty_stack.pop_two() {
                            (Type::I64, Type::I64) => ty_stack.push(Type::I64),
                            (Type::I64 | Type::F64, Type::I64 | Type::F64) => ty_stack.push(Type::F64),
                            _ => unreachable!(),
                        },
                        IntrinsicOperation::DivNumLR | IntrinsicOperation::RemNumLR => {
                            let _ = ty_stack.pop_two();
                            ty_stack.push(Type::F64);
                        }
                        IntrinsicOperation::GtNumLR
                        | IntrinsicOperation::GeNumLR
                        | IntrinsicOperation::LtNumLR
                        | IntrinsicOperation::LeNumLR
                        | IntrinsicOperation::EqNumLR
                        | IntrinsicOperation::NeNumLR => {
                            let _ = ty_stack.pop_two();
                            ty_stack.push(Type::Boolean);
                        }
                        IntrinsicOperation::BitOrNumLR
                        | IntrinsicOperation::BitXorNumLR
                        | IntrinsicOperation::BitAndNumLR
                        | IntrinsicOperation::BitShlNumLR
                        | IntrinsicOperation::BitShrNumLR
                        | IntrinsicOperation::BitUshrNumLR => {
                            let _ = ty_stack.pop_two();
                            ty_stack.push(Type::I64);
                        }

                        IntrinsicOperation::PostfixIncLocalNum
                        | IntrinsicOperation::PostfixDecLocalNum
                        | IntrinsicOperation::PrefixIncLocalNum
                        | IntrinsicOperation::PrefixDecLocalNum => {
                            let id = dcx.next_byte();
                            let ty = self.get_or_insert_local_ty(id.into());
                            ty_stack.push(ty);
                        }

                        IntrinsicOperation::GtNumLConstR
                        | IntrinsicOperation::GeNumLConstR
                        | IntrinsicOperation::LtNumLConstR
                        | IntrinsicOperation::LeNumLConstR => {
                            let _ = ty_stack.pop();
                            let _ = dcx.next_byte();
                            ty_stack.push(Type::Boolean);
                        }

                        IntrinsicOperation::GtNumLConstR32
                        | IntrinsicOperation::GeNumLConstR32
                        | IntrinsicOperation::LtNumLConstR32
                        | IntrinsicOperation::LeNumLConstR32 => {
                            let _ = ty_stack.pop();
                            let _ = dcx.next_u32();
                            ty_stack.push(Type::Boolean);
                        }
                        _ => return Err(Error::UnsupportedInstruction { instr }),
                    }
                }
                Instruction::Nop => {}
                _ => return Err(Error::UnsupportedInstruction { instr }),
            }
        }

        // End of basic block was not reached in the block,
        // which means that this basic block was terminated
        // early not by a conditional jump but by another label
        if let Some(succ) = succ {
            let BasicBlockSuccessor::Unconditional(target) = succ else {
                panic!("mismatching basic block successor");
            };
            self.resolve_types(ty_stack.clone(), target)?;
        }

        Ok(())
    }
}
