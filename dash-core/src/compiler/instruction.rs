use std::convert::TryInto;

use super::{
    builder::{force_utf8_borrowed, InstructionBuilder, Label},
    constant::{Constant, ConstantPool, LimitExceededError},
    error::CompileError,
    FunctionCallMetadata,
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
pub const STATICPROPACCESS: u8 = 0x23;
pub const STATICPROPACCESSW: u8 = 0x24;
pub const DYNAMICPROPACCESS: u8 = 0x25;
pub const ARRAYLIT: u8 = 0x26;
pub const ARRAYLITW: u8 = 0x27;
pub const OBJLIT: u8 = 0x28;
pub const OBJLITW: u8 = 0x29;
pub const THIS: u8 = 0x2A;
pub const STATICPROPSET: u8 = 0x2B;
pub const STATICPROPSETW: u8 = 0x2C;
pub const DYNAMICPROPSET: u8 = 0x2D;
/// Loads an "extern" local variable, existing in a parent scope
pub const LDLOCALEXT: u8 = 0x2E;
pub const LDLOCALEXTW: u8 = 0x2F;
pub const STORELOCALEXT: u8 = 0x30;
pub const STORELOCALEXTW: u8 = 0x31;
pub const STRICTEQ: u8 = 0x32;

#[rustfmt::skip]
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
    /// Builds the [STRICTEQ] instruction
    fn build_strict_eq(&mut self);
    /// Builds the [NE] instruction
    fn build_ne(&mut self);
    /// Builds the [POS] instruction
    fn build_pos(&mut self);
    /// Builds the [NEG] instruction
    fn build_neg(&mut self);
    /// Builds the [TYPEOF] instruction
    fn build_typeof(&mut self);
    /// Builds the [BITNOT] instruction
    fn build_bitnot(&mut self);
    /// Builds the [NOT] instruction
    fn build_not(&mut self);
    /// Builds the [POP] instruction
    fn build_pop(&mut self);
    /// Builds the [RET] instruction
    fn build_ret(&mut self);
    /// Builds the [THIS] instruction
    fn build_this(&mut self);
    /// Builds the [JMPFALSEP] and [JMPFALSEWP] instructions
    fn build_jmpfalsep(&mut self, label: Label) -> Result<(), LimitExceededError>;
    /// Builds the [ARRAYLIT] and [ARRAYLITW] instructions
    fn build_arraylit(&mut self, len: u16);
    /// Builds the [OBJLIT] and [OBJLITW] instructions
    fn build_objlit(&mut self, cp: &mut ConstantPool, constants: Vec<Constant>) -> Result<(), CompileError>;
    /// Builds the [JMP] and [JMPW] instructions
    fn build_jmp(&mut self, label: Label) -> Result<(), LimitExceededError>;
    fn build_call(&mut self, meta: FunctionCallMetadata);
    fn build_static_prop_access(&mut self, cp: &mut ConstantPool, ident: &[u8], preserve_this: bool) -> Result<(), LimitExceededError>;
    fn build_dynamic_prop_access(&mut self, preserve_this: bool);
    fn build_static_prop_set(&mut self, cp: &mut ConstantPool, ident: &[u8]) -> Result<(), LimitExceededError>;
    fn build_dynamic_prop_set(&mut self);
    fn build_constant(&mut self, cp: &mut ConstantPool, constant: Constant) -> Result<(), LimitExceededError>;
    fn build_local_load(&mut self, index: u16, is_extern: bool);
    fn build_global_load(&mut self, cp: &mut ConstantPool, ident: &[u8]) -> Result<(), LimitExceededError>;
    fn build_global_store(&mut self, cp: &mut ConstantPool, ident: &[u8]) -> Result<(), LimitExceededError>;
    fn build_local_store(&mut self, id: u16, is_extern: bool);
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
        build_ret RET,
        build_this THIS,
        build_strict_eq STRICTEQ
    }

    fn build_constant(
        &mut self,
        cp: &mut ConstantPool,
        constant: Constant,
    ) -> Result<(), LimitExceededError> {
        self.write_wide_instr(CONSTANT, CONSTANTW, cp.add(constant)?);
        Ok(())
    }

    fn build_local_load(&mut self, index: u16, is_extern: bool) {
        let (thin, wide) = is_extern
            .then(|| (LDLOCALEXT, LDLOCALEXTW))
            .unwrap_or((LDLOCAL, LDLOCALW));

        self.write_wide_instr(thin, wide, index);
    }

    fn build_global_load(
        &mut self,
        cp: &mut ConstantPool,
        ident: &[u8],
    ) -> Result<(), LimitExceededError> {
        let id = cp.add(Constant::Identifier(force_utf8_borrowed(ident).into()))?;
        self.write_wide_instr(LDGLOBAL, LDGLOBALW, id);
        Ok(())
    }

    fn build_global_store(
        &mut self,
        cp: &mut ConstantPool,
        ident: &[u8],
    ) -> Result<(), LimitExceededError> {
        let id = cp.add(Constant::Identifier(force_utf8_borrowed(ident).into()))?;
        self.write_wide_instr(STOREGLOBAL, STOREGLOBALW, id);
        Ok(())
    }

    fn build_local_store(&mut self, id: u16, is_extern: bool) {
        let (thin, wide) = is_extern
            .then(|| (STORELOCALEXT, STORELOCALEXTW))
            .unwrap_or((STORELOCAL, STORELOCALW));

        self.write_wide_instr(thin, wide, id);
    }

    fn build_call(&mut self, meta: FunctionCallMetadata) {
        self.write_arr([CALL, meta.into()]);
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

    fn build_static_prop_access(
        &mut self,
        cp: &mut ConstantPool,
        ident: &[u8],
        preserve_this: bool,
    ) -> Result<(), LimitExceededError> {
        let id = cp.add(Constant::Identifier(force_utf8_borrowed(ident).into()))?;
        self.write_wide_instr(STATICPROPACCESS, STATICPROPACCESSW, id);
        self.write(preserve_this.into());

        Ok(())
    }

    fn build_dynamic_prop_access(&mut self, preserve_this: bool) {
        self.write_arr([DYNAMICPROPACCESS, preserve_this.into()]);
    }

    fn build_static_prop_set(
        &mut self,
        cp: &mut ConstantPool,
        ident: &[u8],
    ) -> Result<(), LimitExceededError> {
        let id = cp.add(Constant::Identifier(force_utf8_borrowed(ident).into()))?;
        self.write_wide_instr(STATICPROPSET, STATICPROPSETW, id);

        Ok(())
    }

    fn build_dynamic_prop_set(&mut self) {
        self.write(DYNAMICPROPSET);
    }

    fn build_arraylit(&mut self, len: u16) {
        self.write_wide_instr(ARRAYLIT, ARRAYLITW, len);
    }

    fn build_objlit(
        &mut self,
        cp: &mut ConstantPool,
        constants: Vec<Constant>,
    ) -> Result<(), CompileError> {
        let len = constants
            .len()
            .try_into()
            .map_err(|_| CompileError::ObjectLitLimitExceeded)?;

        self.write_wide_instr(OBJLIT, OBJLITW, len);

        for constant in constants {
            // For now, we only support object literals in functions with <256 constants,
            // otherwise we would need to emit 2-byte wide instructions for every constant.
            let id = cp
                .add(constant)?
                .try_into()
                .map_err(|_| CompileError::ConstantPoolLimitExceeded)?;

            self.write(id);
        }

        Ok(())
    }
}
