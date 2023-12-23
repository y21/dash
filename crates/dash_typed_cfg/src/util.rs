use std::iter::Enumerate;
use std::slice::Iter;

use dash_middle::compiler::instruction::{Instruction, IntrinsicOperation};

#[derive(Debug)]
pub struct DecodeCtxt<'a> {
    iter: Enumerate<Iter<'a, u8>>,
}

impl<'a> DecodeCtxt<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            iter: bytes.iter().enumerate(),
        }
    }

    pub fn next_instruction(&mut self) -> Option<(usize, Instruction)> {
        self.iter.next().map(|(a, &b)| (a, Instruction::from_repr(b).unwrap()))
    }

    pub fn next_byte(&mut self) -> u8 {
        self.iter.next().map(|(_, &b)| b).unwrap()
    }

    pub fn skip(&mut self, n: usize) {
        for _ in 0..n {
            self.next_byte();
        }
    }

    pub fn next_wide(&mut self) -> u16 {
        let a = self.next_byte();
        let b = self.next_byte();
        u16::from_ne_bytes([a, b])
    }

    pub fn next_u32(&mut self) -> u32 {
        let a = self.next_byte();
        let b = self.next_byte();
        let c = self.next_byte();
        let d = self.next_byte();
        u32::from_ne_bytes([a, b, c, d])
    }

    pub fn next_wide_signed(&mut self) -> i16 {
        self.next_wide() as i16
    }

    /// Decodes an instruction and does nothing with it apart from advancing the iterator.
    /// Useful for passes that are only interested in a few instructions
    /// and do not care about the rest. For the other instructions, they can call this method.
    pub fn decode_ignore(&mut self, instr: Instruction) {
        match instr {
            Instruction::Add
            | Instruction::Sub
            | Instruction::Mul
            | Instruction::Div
            | Instruction::Rem
            | Instruction::BitAnd => {}
            Instruction::Constant => drop(self.next_byte()),
            Instruction::ConstantW => drop(self.next_wide()),
            Instruction::LdLocal => drop(self.next_byte()),
            Instruction::LdLocalW => drop(self.next_wide()),
            Instruction::StoreLocal => {
                self.next_byte();
                self.next_byte();
            }
            Instruction::StoreLocalW => {
                self.next_wide();
                self.next_byte();
            }
            Instruction::Not
            | Instruction::Lt
            | Instruction::Le
            | Instruction::Gt
            | Instruction::Ge
            | Instruction::Eq
            | Instruction::Ne
            | Instruction::StrictEq
            | Instruction::StrictNe => {}
            Instruction::Jmp => drop(self.next_wide()),
            Instruction::JmpFalseP | Instruction::JmpNullishP | Instruction::JmpTrueP | Instruction::JmpUndefinedP => {
                panic!("Conditional jumps cannot be ignored")
            }
            Instruction::IntrinsicOp => {
                let op = IntrinsicOperation::from_repr(self.next_byte()).unwrap();
                match op {
                    IntrinsicOperation::AddNumLR
                    | IntrinsicOperation::SubNumLR
                    | IntrinsicOperation::MulNumLR
                    | IntrinsicOperation::PowNumLR => {}
                    IntrinsicOperation::DivNumLR | IntrinsicOperation::RemNumLR => {}
                    IntrinsicOperation::GtNumLR
                    | IntrinsicOperation::GeNumLR
                    | IntrinsicOperation::LtNumLR
                    | IntrinsicOperation::LeNumLR
                    | IntrinsicOperation::EqNumLR
                    | IntrinsicOperation::NeNumLR => {}
                    IntrinsicOperation::BitOrNumLR
                    | IntrinsicOperation::BitXorNumLR
                    | IntrinsicOperation::BitAndNumLR
                    | IntrinsicOperation::BitShlNumLR
                    | IntrinsicOperation::BitShrNumLR
                    | IntrinsicOperation::BitUshrNumLR => {}

                    IntrinsicOperation::PostfixIncLocalNum
                    | IntrinsicOperation::PostfixDecLocalNum
                    | IntrinsicOperation::PrefixIncLocalNum
                    | IntrinsicOperation::PrefixDecLocalNum => drop(self.next_byte()),

                    IntrinsicOperation::GtNumLConstR
                    | IntrinsicOperation::GeNumLConstR
                    | IntrinsicOperation::LtNumLConstR
                    | IntrinsicOperation::LeNumLConstR => drop(self.next_byte()),

                    IntrinsicOperation::GtNumLConstR32
                    | IntrinsicOperation::GeNumLConstR32
                    | IntrinsicOperation::LtNumLConstR32
                    | IntrinsicOperation::LeNumLConstR32 => drop(self.next_u32()),
                    other => todo!("{other:?}"),
                }
            }
            Instruction::Pop => {}
            Instruction::Ret => drop(self.next_wide()),
            other => todo!("{other:?}"),
        }
    }
}
