use std::convert::TryInto;

use dash_middle::{
    compiler::{
        constant::{Constant, LimitExceededError},
        instruction::Instruction,
        FunctionCallMetadata, ObjectMemberKind as CompilerObjectMemberKind, StaticImportKind,
    },
    parser::expr::ObjectMemberKind,
};

use super::{
    builder::{InstructionBuilder, Label},
    error::CompileError,
};

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
    /// Builds the [STRICTNE] instruction
    fn build_strict_ne(&mut self);
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
    /// Builds the [UNDEF] instruction
    fn build_undef(&mut self);
    /// Builds the [RET] instruction
    fn build_ret(&mut self, tc_depth: u16);
    /// Builds the [THIS] instruction
    fn build_this(&mut self);
    /// Builds the [SUPER] instruction
    fn build_super(&mut self);
    /// Builds the [GLOBAL] instruction
    fn build_global(&mut self);
    /// Builds the [JMPFALSEP] instructions
    fn build_jmpfalsep(&mut self, label: Label);
    /// Builds the [JMPFALSENP] instructions
    fn build_jmpfalsenp(&mut self, label: Label);
    /// Builds the [JMPTRUEP] instructions
    fn build_jmptruep(&mut self, label: Label);
    /// Builds the [JMPTRUENP] instructions
    fn build_jmptruenp(&mut self, label: Label);
    /// Builds the [JMPNULLISHP] instructions
    fn build_jmpnullishp(&mut self, label: Label);
    /// Builds the [JMPNULLISHNP] instructions
    fn build_jmpnullishnp(&mut self, label: Label);
    /// Builds the [ARRAYLIT] and [ARRAYLITW] instructions
    fn build_arraylit(&mut self, len: u16);
    /// Builds the [OBJLIT] and [OBJLITW] instructions
    fn build_objlit(&mut self, constants: Vec<ObjectMemberKind>) -> Result<(), CompileError>;
    /// Builds the [JMP] instructions
    fn build_jmp(&mut self, label: Label);
    fn build_call(&mut self, meta: FunctionCallMetadata);
    fn build_static_prop_access(&mut self, ident: &str, preserve_this: bool) -> Result<(), LimitExceededError>;
    fn build_dynamic_prop_access(&mut self, preserve_this: bool);
    fn build_static_prop_set(&mut self, ident: &str) -> Result<(), LimitExceededError>;
    fn build_dynamic_prop_set(&mut self);
    fn build_constant(&mut self, constant: Constant) -> Result<(), LimitExceededError>;
    fn build_local_load(&mut self, index: u16, is_extern: bool);
    fn build_global_load(&mut self, ident: &str) -> Result<(), LimitExceededError>;
    fn build_global_store(&mut self, ident: &str) -> Result<(), LimitExceededError>;
    fn build_local_store(&mut self, id: u16, is_extern: bool);
    fn build_try_block(&mut self);
    fn build_try_end(&mut self);
    fn build_throw(&mut self);
    fn build_yield(&mut self);
    fn build_await(&mut self);
    fn build_bitor(&mut self);
    fn build_bitxor(&mut self);
    fn build_bitand(&mut self);
    fn build_bitshl(&mut self);
    fn build_bitshr(&mut self);
    fn build_bitushr(&mut self);
    fn build_objin(&mut self);
    fn build_instanceof(&mut self);
    fn build_dynamic_import(&mut self);
    fn build_static_import(&mut self, import: StaticImportKind, local_id: u16, path_id: u16);
    fn build_default_export(&mut self);
    fn build_named_export(&mut self, it: &[NamedExportKind]) -> Result<(), CompileError>;
    fn build_debugger(&mut self);
    fn build_revstck(&mut self, n: u8);
    fn build_break(&mut self);
    fn build_infinity(&mut self);
    fn build_nan(&mut self);
    fn build_symbol_iterator(&mut self);
    fn build_for_in_iterator(&mut self);
    fn build_static_delete(&mut self, id: u16);
    fn build_dynamic_delete(&mut self);
}

macro_rules! impl_instruction_writer {
    ($($fname:ident $value:expr),*) => {
        $(
            fn $fname(&mut self) {
                self.write($value as u8);
            }
        )*
    }
}

impl<'cx, 'inp> InstructionWriter for InstructionBuilder<'cx, 'inp> {
    impl_instruction_writer! {
        build_add Instruction::Add,
        build_sub Instruction::Sub,
        build_mul Instruction::Mul,
        build_div Instruction::Div,
        build_rem Instruction::Rem,
        build_pow Instruction::Pow,
        build_gt Instruction::Gt,
        build_ge Instruction::Ge,
        build_lt Instruction::Lt,
        build_le Instruction::Le,
        build_eq Instruction::Eq,
        build_ne Instruction::Ne,
        build_pop Instruction::Pop,
        build_pos Instruction::Pos,
        build_neg Instruction::Neg,
        build_typeof Instruction::TypeOf,
        build_bitnot Instruction::BitNot,
        build_not Instruction::Not,
        build_this Instruction::This,
        build_strict_eq Instruction::StrictEq,
        build_strict_ne Instruction::StrictNe,
        build_try_end Instruction::TryEnd,
        build_throw Instruction::Throw,
        build_yield Instruction::Yield,
        build_await Instruction::Await,
        build_bitor Instruction::BitOr,
        build_bitxor Instruction::BitXor,
        build_bitand Instruction::BitAnd,
        build_bitshl Instruction::BitShl,
        build_bitshr Instruction::BitShr,
        build_bitushr Instruction::BitUshr,
        build_objin Instruction::ObjIn,
        build_instanceof Instruction::InstanceOf,
        build_default_export Instruction::ExportDefault,
        build_debugger Instruction::Debugger,
        build_super Instruction::Super,
        build_global Instruction::Global,
        build_infinity Instruction::Infinity,
        build_nan Instruction::Nan,
        build_undef Instruction::Undef,
        build_break Instruction::Break,
        build_symbol_iterator Instruction::CallSymbolIterator,
        build_for_in_iterator Instruction::CallForInIterator,
        build_dynamic_delete Instruction::DeletePropertyDynamic
    }

    fn build_ret(&mut self, tc_depth: u16) {
        self.write_instr(Instruction::Ret);
        self.writew(tc_depth);
    }

    fn build_constant(&mut self, constant: Constant) -> Result<(), LimitExceededError> {
        let id = self.cp.add(constant)?;
        self.write_wide_instr(Instruction::Constant, Instruction::ConstantW, id);
        Ok(())
    }

    fn build_try_block(&mut self) {
        self.write_instr(Instruction::Try);
        self.write_all(&[0, 0]);
        self.add_local_jump(Label::Catch);
    }

    fn build_local_load(&mut self, index: u16, is_extern: bool) {
        let (thin, wide) = is_extern
            .then(|| (Instruction::LdLocalExt, Instruction::LdLocalExtW))
            .unwrap_or((Instruction::LdLocal, Instruction::LdLocalW));

        self.write_wide_instr(thin, wide, index);
    }

    fn build_global_load(&mut self, ident: &str) -> Result<(), LimitExceededError> {
        let id = self.cp.add(Constant::Identifier(ident.into()))?;
        self.write_wide_instr(Instruction::LdGlobal, Instruction::LdGlobalW, id);
        Ok(())
    }

    fn build_global_store(&mut self, ident: &str) -> Result<(), LimitExceededError> {
        let id = self.cp.add(Constant::Identifier(ident.into()))?;
        self.write_wide_instr(Instruction::StoreGlobal, Instruction::StoreGlobalW, id);
        Ok(())
    }

    fn build_local_store(&mut self, id: u16, is_extern: bool) {
        let (thin, wide) = is_extern
            .then(|| (Instruction::StoreLocalExt, Instruction::StoreLocalExtW))
            .unwrap_or((Instruction::StoreLocal, Instruction::StoreLocalW));

        self.write_wide_instr(thin, wide, id);
    }

    fn build_call(&mut self, meta: FunctionCallMetadata) {
        self.write_instr(Instruction::Call);
        self.write(meta.into());
    }

    fn build_jmpfalsep(&mut self, label: Label) {
        self.write_instr(Instruction::JmpFalseP);
        self.write_all(&[0, 0]);
        self.add_local_jump(label);
    }

    fn build_jmpfalsenp(&mut self, label: Label) {
        self.write_instr(Instruction::JmpFalseNP);
        self.write_all(&[0, 0]);
        self.add_local_jump(label);
    }

    fn build_jmptruep(&mut self, label: Label) {
        self.write_instr(Instruction::JmpTrueP);
        self.write_all(&[0, 0]);
        self.add_local_jump(label);
    }

    fn build_jmptruenp(&mut self, label: Label) {
        self.write_instr(Instruction::JmpTrueNP);
        self.write_all(&[0, 0]);
        self.add_local_jump(label);
    }

    fn build_jmpnullishp(&mut self, label: Label) {
        self.write_instr(Instruction::JmpNullishP);
        self.write_all(&[0, 0]);
        self.add_local_jump(label);
    }

    fn build_jmpnullishnp(&mut self, label: Label) {
        self.write_instr(Instruction::JmpNullishNP);
        self.write_all(&[0, 0]);
        self.add_local_jump(label);
    }

    fn build_jmp(&mut self, label: Label) {
        self.write_instr(Instruction::Jmp);
        self.write_all(&[0, 0]);
        self.add_local_jump(label);
    }

    fn build_static_prop_access(&mut self, ident: &str, preserve_this: bool) -> Result<(), LimitExceededError> {
        let id = self.cp.add(Constant::Identifier(ident.into()))?;
        self.write_wide_instr(Instruction::StaticPropAccess, Instruction::StaticPropAccessW, id);
        self.write(preserve_this.into());

        Ok(())
    }

    fn build_dynamic_prop_access(&mut self, preserve_this: bool) {
        self.write_instr(Instruction::DynamicPropAccess);
        self.write(preserve_this.into());
    }

    fn build_static_prop_set(&mut self, ident: &str) -> Result<(), LimitExceededError> {
        let id = self.cp.add(Constant::Identifier(ident.into()))?;
        self.write_wide_instr(Instruction::StaticPropSet, Instruction::StaticPropSetW, id);

        Ok(())
    }

    fn build_dynamic_prop_set(&mut self) {
        self.write_instr(Instruction::DynamicPropSet)
    }

    fn build_arraylit(&mut self, len: u16) {
        self.write_wide_instr(Instruction::ArrayLit, Instruction::ArrayLitW, len);
    }

    fn build_objlit(&mut self, constants: Vec<ObjectMemberKind>) -> Result<(), CompileError> {
        let len = constants
            .len()
            .try_into()
            .map_err(|_| CompileError::ObjectLitLimitExceeded)?;

        self.write_wide_instr(Instruction::ObjLit, Instruction::ObjLitW, len);

        // Push in reverse order to match order in which the compiler pushes values onto the stack
        for member in constants.into_iter().rev() {
            match member {
                ObjectMemberKind::Dynamic(..) => self.write(CompilerObjectMemberKind::Dynamic as u8),
                ObjectMemberKind::Getter(name) | ObjectMemberKind::Setter(name) | ObjectMemberKind::Static(name) => {
                    let id = self
                        .cp
                        .add(Constant::Identifier(name.into()))?
                        .try_into()
                        .map_err(|_| CompileError::ConstantPoolLimitExceeded)?;

                    let kind_id = CompilerObjectMemberKind::from(member) as u8;

                    self.write(kind_id);
                    self.write(id);
                }
            }
        }

        Ok(())
    }

    fn build_static_import(&mut self, import: StaticImportKind, local_id: u16, path_id: u16) {
        self.write_instr(Instruction::ImportStatic);
        self.write(import as u8);
        self.writew(local_id);
        self.writew(path_id);
    }

    fn build_dynamic_import(&mut self) {
        self.write_instr(Instruction::ImportDyn);
    }

    fn build_revstck(&mut self, n: u8) {
        self.write_instr(Instruction::RevStck);
        self.write(n);
    }

    fn build_static_delete(&mut self, id: u16) {
        self.write_instr(Instruction::DeletePropertyStatic);
        self.writew(id);
    }

    fn build_named_export(&mut self, it: &[NamedExportKind]) -> Result<(), CompileError> {
        self.write_instr(Instruction::ExportNamed);

        let len = it
            .len()
            .try_into()
            .map_err(|_| CompileError::ExportNameListLimitExceeded)?;

        self.writew(len);

        for kind in it.iter().copied() {
            match kind {
                NamedExportKind::Local { loc_id, ident_id } => {
                    self.write(0);
                    self.writew(loc_id);
                    self.writew(ident_id);
                }
                NamedExportKind::Global { ident_id } => {
                    self.write(1);
                    self.writew(ident_id);
                }
            }
        }

        Ok(())
    }
}

#[derive(Copy, Clone)]
pub enum NamedExportKind {
    Local { loc_id: u16, ident_id: u16 },
    Global { ident_id: u16 },
}
