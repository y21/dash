use super::{
    builder::{force_utf8, InstructionBuilder, Label},
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
/// Jumps to the given label
pub const CJMPFALSEP: u8 = 0x1F;
pub const JMPFALSEP: u8 = 0x1F;
pub const CJMPFALSEWP: u8 = 0x20;
pub const JMPFALSEWP: u8 = 0x20;
pub const CJMP: u8 = 0x21;
pub const JMP: u8 = 0x21;
pub const CJMPW: u8 = 0x22;
pub const JMPW: u8 = 0x22;

pub trait InstructionWriter {
    /// Builds the [ADD] instruction
    fn build_add(&mut self);
    /// Builds the [SUB] instruction
    fn build_sub(&mut self);
    /// Builds the [MUL] instruction
    fn build_mul(&mut self);
    /// Builds the [DIV] instruction
    fn build_div(&mut self);
    /// Builds the [REM] instruction
    fn build_rem(&mut self);
    /// Builds the [POW] instruction
    fn build_pow(&mut self);
    /// Builds the [GT] instruction
    fn build_gt(&mut self);
    /// Builds the [GE] instruction
    fn build_ge(&mut self);
    /// Builds the [LT] instruction
    fn build_lt(&mut self);
    /// Builds the [LE] instruction
    fn build_le(&mut self);
    /// Builds the [EQ] instruction
    fn build_eq(&mut self);
    /// Builds the [NE] instruction
    fn build_ne(&mut self);
    /// Builds the [POP] instruction
    fn build_pop(&mut self);
    /// Builds the [RET] instruction
    fn build_ret(&mut self);
    /// Builds the [JMPFALSEP] and [JMPFALSEWP] instructions
    fn build_jmpfalsep(&mut self, label: Label) -> Result<(), LimitExceededError>;
    /// Builds the [JMP] and [JMPW] instructions
    fn build_jmp(&mut self, label: Label) -> Result<(), LimitExceededError>;
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

    fn build_jmpfalsep(&mut self, label: Label) -> Result<(), LimitExceededError> {
        let id = self.add_jump(label)?;
        self.write_wide_instr(CJMPFALSEP, CJMPFALSEWP, id);

        Ok(())
    }

    fn build_jmp(&mut self, label: Label) -> Result<(), LimitExceededError> {
        let id = self.add_jump(label)?;
        self.write_wide_instr(JMP, JMPW, id);

        Ok(())
    }
}
