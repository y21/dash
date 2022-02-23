use super::{
    builder::{force_utf8, InstructionBuilder},
    constant::{Constant, ConstantPool, LimitExceededError},
};

/// Adds two values together
pub const ADD: u8 = 0x01;
pub const SUB: u8 = 0x02;
pub const MUL: u8 = 0x03;
pub const DIV: u8 = 0x04;
pub const REM: u8 = 0x05;
pub const POW: u8 = 0x06;
pub const GT: u8 = 0x07;
pub const GE: u8 = 0x08;
pub const LT: u8 = 0x09;
pub const LE: u8 = 0x0A;
pub const EQ: u8 = 0x0B;
pub const NE: u8 = 0x0C;
/// Discards the last value on the stack
pub const POP: u8 = 0x0D;
/// Loads a local value
pub const LDLOCAL: u8 = 0x0E;
pub const LDLOCALW: u8 = 0x0F;
pub const LDGLOBAL: u8 = 0x10;
pub const LDGLOBALW: u8 = 0x11;
pub const CONSTANT: u8 = 0x12;
pub const CONSTANTW: u8 = 0x13;
pub const POS: u8 = 0x14;
/// Negates the last value on the stack
pub const NEG: u8 = 0x15;
pub const TYPEOF: u8 = 0x16;
pub const BITNOT: u8 = 0x17;
pub const NOT: u8 = 0x18;
pub const STORELOCAL: u8 = 0x19;
pub const STORELOCALW: u8 = 0x1A;
pub const STOREGLOBAL: u8 = 0x1B;
pub const STOREGLOBALW: u8 = 0x1C;
pub const RET: u8 = 0x1D;
pub const CALL: u8 = 0x1E;

pub trait InstructionWriter {
    fn build_add(&mut self);
    fn build_sub(&mut self);
    fn build_mul(&mut self);
    fn build_div(&mut self);
    fn build_rem(&mut self);
    fn build_pow(&mut self);
    fn build_gt(&mut self);
    fn build_ge(&mut self);
    fn build_lt(&mut self);
    fn build_le(&mut self);
    fn build_eq(&mut self);
    fn build_ne(&mut self);
    fn build_pop(&mut self);
    fn build_ret(&mut self);
    fn build_call(&mut self, argc: u8, is_constructor: bool);
    fn build_constant(
        &mut self,
        cp: &mut ConstantPool,
        constant: Constant,
    ) -> Result<(), LimitExceededError>;
    fn build_local_load(&mut self, index: u16);
    fn build_global_load(
        &mut self,
        cp: &mut ConstantPool,
        ident: &[u8],
    ) -> Result<(), LimitExceededError>;
    fn build_pos(&mut self);
    fn build_neg(&mut self);
    fn build_typeof(&mut self);
    fn build_bitnot(&mut self);
    fn build_not(&mut self);
    fn build_global_store(
        &mut self,
        cp: &mut ConstantPool,
        ident: &[u8],
    ) -> Result<(), LimitExceededError>;
    fn build_local_store(&mut self, id: u16);
}

macro_rules! impl_instruction_writer {
    ($($fname:ident $value:expr),*) => {
        $(
            fn $fname(&mut self) {
                self.write($value);
            }
        )*
    }
}

impl InstructionWriter for InstructionBuilder {
    impl_instruction_writer! {
        build_add ADD,
        build_sub SUB,
        build_mul MUL,
        build_div DIV,
        build_rem REM,
        build_pow POW,
        build_gt GT,
        build_ge GE,
        build_lt LT,
        build_le LE,
        build_eq EQ,
        build_ne NE,
        build_pop POP,
        build_pos POS,
        build_neg NEG,
        build_typeof TYPEOF,
        build_bitnot BITNOT,
        build_not NOT,
        build_ret RET
    }

    fn build_constant(
        &mut self,
        cp: &mut ConstantPool,
        constant: Constant,
    ) -> Result<(), LimitExceededError> {
        self.write_wide_instr(CONSTANT, CONSTANTW, cp.add(constant)?);
        Ok(())
    }

    fn build_local_load(&mut self, index: u16) {
        self.write_wide_instr(LDLOCAL, LDLOCALW, index);
    }

    fn build_global_load(
        &mut self,
        cp: &mut ConstantPool,
        ident: &[u8],
    ) -> Result<(), LimitExceededError> {
        let id = cp.add(Constant::Identifier(force_utf8(ident)))?;
        self.write_wide_instr(LDGLOBAL, LDGLOBALW, id);
        Ok(())
    }

    fn build_global_store(
        &mut self,
        cp: &mut ConstantPool,
        ident: &[u8],
    ) -> Result<(), LimitExceededError> {
        let id = cp.add(Constant::Identifier(force_utf8(ident)))?;
        self.write_wide_instr(STOREGLOBAL, STOREGLOBALW, id);
        Ok(())
    }

    fn build_local_store(&mut self, id: u16) {
        self.write_wide_instr(STORELOCAL, STORELOCALW, id);
    }

    fn build_call(&mut self, argc: u8, is_constructor: bool) {
        self.write_arr([CALL, argc, is_constructor as u8]);
    }
}
