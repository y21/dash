use std::convert::TryInto;

use dash_middle::{
    compiler::{
        constant::{Constant, LimitExceededError},
        instruction::{AssignKind, Instruction, IntrinsicOperation},
        FunctionCallMetadata, ObjectMemberKind as CompilerObjectMemberKind, StaticImportKind,
    },
    parser::expr::ObjectMemberKind,
};

use super::{
    builder::{InstructionBuilder, Label},
    error::CompileError,
};

macro_rules! simple_instruction {
    ($($fname:ident $value:expr),*) => {
        $(
            pub fn $fname(&mut self) {
                self.write($value as u8);
            }
        )*
    }
}

impl<'cx, 'inp> InstructionBuilder<'cx, 'inp> {
    pub fn build_jmp_header(&mut self, label: Label, is_local_label: bool) {
        self.write_all(&[0, 0]);
        match is_local_label {
            true => self.add_local_jump(label),
            false => self.current_function_mut().add_global_jump(label),
        }
    }

    simple_instruction! {
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

    pub fn build_ret(&mut self, tc_depth: u16) {
        self.write_instr(Instruction::Ret);
        self.writew(tc_depth);
    }

    pub fn build_constant(&mut self, constant: Constant) -> Result<(), LimitExceededError> {
        let id = self.current_function_mut().cp.add(constant)?;
        self.write_wide_instr(Instruction::Constant, Instruction::ConstantW, id);
        Ok(())
    }

    pub fn build_try_block(&mut self) {
        self.write_instr(Instruction::Try);
        self.write_all(&[0, 0]);
        self.add_local_jump(Label::Catch);
    }

    pub fn build_local_load(&mut self, index: u16, is_extern: bool) {
        compile_local_load_into(&mut self.current_function_mut().buf, index, is_extern);
    }

    pub fn build_global_load(&mut self, ident: &str) -> Result<(), LimitExceededError> {
        let id = self.current_function_mut().cp.add(Constant::Identifier(ident.into()))?;
        self.write_wide_instr(Instruction::LdGlobal, Instruction::LdGlobalW, id);
        Ok(())
    }

    pub fn build_global_store(&mut self, kind: AssignKind, ident: &str) -> Result<(), LimitExceededError> {
        let id = self.current_function_mut().cp.add(Constant::Identifier(ident.into()))?;
        self.write_wide_instr(Instruction::StoreGlobal, Instruction::StoreGlobalW, id);
        self.write(kind as u8);
        Ok(())
    }

    pub fn build_local_store(&mut self, kind: AssignKind, id: u16, is_extern: bool) {
        let (thin, wide) = if is_extern {
            (Instruction::StoreLocalExt, Instruction::StoreLocalExtW)
        } else {
            (Instruction::StoreLocal, Instruction::StoreLocalW)
        };

        self.write_wide_instr(thin, wide, id);
        self.write(kind as u8);
    }

    pub fn build_call(&mut self, meta: FunctionCallMetadata) {
        self.write_instr(Instruction::Call);
        self.write(meta.into());
    }

    pub fn build_jmpfalsep(&mut self, label: Label, is_local_label: bool) {
        self.write_instr(Instruction::JmpFalseP);
        self.build_jmp_header(label, is_local_label);
    }

    pub fn build_jmpfalsenp(&mut self, label: Label, is_local_label: bool) {
        self.write_instr(Instruction::JmpFalseNP);
        self.build_jmp_header(label, is_local_label);
    }

    pub fn build_jmptruep(&mut self, label: Label, is_local_label: bool) {
        self.write_instr(Instruction::JmpTrueP);
        self.build_jmp_header(label, is_local_label);
    }

    pub fn build_jmptruenp(&mut self, label: Label, is_local_label: bool) {
        self.write_instr(Instruction::JmpTrueNP);
        self.build_jmp_header(label, is_local_label);
    }

    pub fn build_jmpnullishp(&mut self, label: Label, is_local_label: bool) {
        self.write_instr(Instruction::JmpNullishP);
        self.build_jmp_header(label, is_local_label);
    }

    pub fn build_jmpnullishnp(&mut self, label: Label, is_local_label: bool) {
        self.write_instr(Instruction::JmpNullishNP);
        self.build_jmp_header(label, is_local_label);
    }

    pub fn build_jmpundefinednp(&mut self, label: Label, is_local_label: bool) {
        self.write_instr(Instruction::JmpUndefinedNP);
        self.build_jmp_header(label, is_local_label);
    }

    pub fn build_jmpundefinedp(&mut self, label: Label, is_local_label: bool) {
        self.write_instr(Instruction::JmpUndefinedP);
        self.build_jmp_header(label, is_local_label);
    }

    pub fn build_jmp(&mut self, label: Label, is_local_label: bool) {
        self.write_instr(Instruction::Jmp);
        self.build_jmp_header(label, is_local_label);
    }

    pub fn build_static_prop_access(&mut self, ident: &str, preserve_this: bool) -> Result<(), LimitExceededError> {
        let id = self.current_function_mut().cp.add(Constant::Identifier(ident.into()))?;
        self.write_wide_instr(Instruction::StaticPropAccess, Instruction::StaticPropAccessW, id);
        self.write(preserve_this.into());

        Ok(())
    }

    pub fn build_dynamic_prop_access(&mut self, preserve_this: bool) {
        self.write_instr(Instruction::DynamicPropAccess);
        self.write(preserve_this.into());
    }

    pub fn build_static_prop_assign(&mut self, kind: AssignKind, ident: &str) -> Result<(), LimitExceededError> {
        let id = self.current_function_mut().cp.add(Constant::Identifier(ident.into()))?;
        self.write_instr(Instruction::StaticPropAssign);
        self.write(kind as u8);
        self.writew(id);

        Ok(())
    }

    pub fn build_dynamic_prop_assign(&mut self, kind: AssignKind) {
        self.write_instr(Instruction::DynamicPropAssign);
        self.write(kind as u8);
    }

    pub fn build_arraylit(&mut self, len: u16) {
        self.write_wide_instr(Instruction::ArrayLit, Instruction::ArrayLitW, len);
    }

    pub fn build_objlit(&mut self, constants: Vec<ObjectMemberKind>) -> Result<(), CompileError> {
        let len = constants
            .len()
            .try_into()
            .map_err(|_| CompileError::ObjectLitLimitExceeded)?;

        self.write_wide_instr(Instruction::ObjLit, Instruction::ObjLitW, len);

        fn compile_object_member_kind(
            ib: &mut InstructionBuilder,
            name: &str,
            kind_id: u8,
        ) -> Result<(), CompileError> {
            let id = ib
                .current_function_mut()
                .cp
                .add(Constant::Identifier(name.into()))?
                .try_into()
                .map_err(|_| CompileError::ConstantPoolLimitExceeded)?;

            ib.write(kind_id);
            ib.write(id);
            Ok(())
        }

        // Push in reverse order to match order in which the compiler pushes values onto the stack
        for member in constants.into_iter().rev() {
            let kind_id = CompilerObjectMemberKind::from(&member) as u8;
            match member {
                ObjectMemberKind::Dynamic(..) => self.write(CompilerObjectMemberKind::Dynamic as u8),
                ObjectMemberKind::Static(name) => compile_object_member_kind(self, name, kind_id)?,
                ObjectMemberKind::Getter(name) | ObjectMemberKind::Setter(name) => {
                    compile_object_member_kind(self, &name, kind_id)?
                }
            }
        }

        Ok(())
    }

    pub fn build_static_import(&mut self, import: StaticImportKind, local_id: u16, path_id: u16) {
        self.write_instr(Instruction::ImportStatic);
        self.write(import as u8);
        self.writew(local_id);
        self.writew(path_id);
    }

    pub fn build_dynamic_import(&mut self) {
        self.write_instr(Instruction::ImportDyn);
    }

    pub fn build_static_delete(&mut self, id: u16) {
        self.write_instr(Instruction::DeletePropertyStatic);
        self.writew(id);
    }

    pub fn build_named_export(&mut self, it: &[NamedExportKind]) -> Result<(), CompileError> {
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

    pub fn build_switch(&mut self, case_count: u16, has_default: bool) {
        self.write(Instruction::Switch as u8);
        self.writew(case_count);
        self.write(has_default.into());
    }

    pub fn build_objdestruct(&mut self, count: u16) {
        self.write_instr(Instruction::ObjDestruct);
        self.writew(count);
    }

    pub fn build_arraydestruct(&mut self, count: u16) {
        self.write_instr(Instruction::ArrayDestruct);
        self.writew(count);
    }

    pub fn build_intrinsic_op(&mut self, op: IntrinsicOperation) {
        self.write_instr(Instruction::IntrinsicOp);
        self.write(op as u8);
    }

    pub fn build_postfix_dec_local_num(&mut self, id: u8) {
        self.build_intrinsic_op(IntrinsicOperation::PostfixDecLocalNum);
        self.write(id);
    }

    pub fn build_postfix_inc_local_num(&mut self, id: u8) {
        self.build_intrinsic_op(IntrinsicOperation::PostfixIncLocalNum);
        self.write(id);
    }

    pub fn build_prefix_dec_local_num(&mut self, id: u8) {
        self.build_intrinsic_op(IntrinsicOperation::PrefixDecLocalNum);
        self.write(id);
    }

    pub fn build_prefix_inc_local_num(&mut self, id: u8) {
        self.build_intrinsic_op(IntrinsicOperation::PrefixIncLocalNum);
        self.write(id);
    }

    pub fn build_ge_numl_constr(&mut self, right: u8) {
        self.build_intrinsic_op(IntrinsicOperation::GeNumLConstR);
        self.write(right);
    }

    pub fn build_gt_numl_constr(&mut self, right: u8) {
        self.build_intrinsic_op(IntrinsicOperation::GtNumLConstR);
        self.write(right);
    }

    pub fn build_le_numl_constr(&mut self, right: u8) {
        self.build_intrinsic_op(IntrinsicOperation::LeNumLConstR);
        self.write(right);
    }

    pub fn build_lt_numl_constr(&mut self, right: u8) {
        self.build_intrinsic_op(IntrinsicOperation::LtNumLConstR);
        self.write(right);
    }

    pub fn build_ge_numl_constr32(&mut self, right: u32) {
        self.build_intrinsic_op(IntrinsicOperation::GeNumLConstR32);
        self.write_all(&right.to_ne_bytes());
    }

    pub fn build_gt_numl_constr32(&mut self, right: u32) {
        self.build_intrinsic_op(IntrinsicOperation::GtNumLConstR32);
        self.write_all(&right.to_ne_bytes());
    }

    pub fn build_le_numl_constr32(&mut self, right: u32) {
        self.build_intrinsic_op(IntrinsicOperation::LeNumLConstR32);
        self.write_all(&right.to_ne_bytes());
    }

    pub fn build_lt_numl_constr32(&mut self, right: u32) {
        self.build_intrinsic_op(IntrinsicOperation::LtNumLConstR32);
        self.write_all(&right.to_ne_bytes());
    }

    pub fn build_exp(&mut self, args: u8) {
        self.build_intrinsic_op(IntrinsicOperation::Exp);
        self.write(args);
    }

    pub fn build_log2(&mut self, args: u8) {
        self.build_intrinsic_op(IntrinsicOperation::Log2);
        self.write(args);
    }

    pub fn build_expm1(&mut self, args: u8) {
        self.build_intrinsic_op(IntrinsicOperation::Expm1);
        self.write(args);
    }

    pub fn build_cbrt(&mut self, args: u8) {
        self.build_intrinsic_op(IntrinsicOperation::Cbrt);
        self.write(args);
    }

    pub fn build_clz32(&mut self, args: u8) {
        self.build_intrinsic_op(IntrinsicOperation::Clz32);
        self.write(args);
    }

    pub fn build_atanh(&mut self, args: u8) {
        self.build_intrinsic_op(IntrinsicOperation::Atanh);
        self.write(args);
    }

    pub fn build_atanh2(&mut self, args: u8) {
        self.build_intrinsic_op(IntrinsicOperation::Atan2);
        self.write(args);
    }

    pub fn build_round(&mut self, args: u8) {
        self.build_intrinsic_op(IntrinsicOperation::Round);
        self.write(args);
    }

    pub fn build_acosh(&mut self, args: u8) {
        self.build_intrinsic_op(IntrinsicOperation::Acosh);
        self.write(args);
    }

    pub fn build_abs(&mut self, args: u8) {
        self.build_intrinsic_op(IntrinsicOperation::Abs);
        self.write(args);
    }

    pub fn build_sinh(&mut self, args: u8) {
        self.build_intrinsic_op(IntrinsicOperation::Sinh);
        self.write(args);
    }

    pub fn build_sin(&mut self, args: u8) {
        self.build_intrinsic_op(IntrinsicOperation::Sin);
        self.write(args);
    }

    pub fn build_ceil(&mut self, args: u8) {
        self.build_intrinsic_op(IntrinsicOperation::Ceil);
        self.write(args);
    }

    pub fn build_tan(&mut self, args: u8) {
        self.build_intrinsic_op(IntrinsicOperation::Tan);
        self.write(args);
    }

    pub fn build_trunc(&mut self, args: u8) {
        self.build_intrinsic_op(IntrinsicOperation::Trunc);
        self.write(args);
    }

    pub fn build_asinh(&mut self, args: u8) {
        self.build_intrinsic_op(IntrinsicOperation::Asinh);
        self.write(args);
    }

    pub fn build_log10(&mut self, args: u8) {
        self.build_intrinsic_op(IntrinsicOperation::Log10);
        self.write(args);
    }

    pub fn build_asin(&mut self, args: u8) {
        self.build_intrinsic_op(IntrinsicOperation::Asin);
        self.write(args);
    }

    pub fn build_random(&mut self, args: u8) {
        self.build_intrinsic_op(IntrinsicOperation::Random);
        self.write(args);
    }

    pub fn build_log1p(&mut self, args: u8) {
        self.build_intrinsic_op(IntrinsicOperation::Log1p);
        self.write(args);
    }

    pub fn build_sqrt(&mut self, args: u8) {
        self.build_intrinsic_op(IntrinsicOperation::Sqrt);
        self.write(args);
    }

    pub fn build_atan(&mut self, args: u8) {
        self.build_intrinsic_op(IntrinsicOperation::Atan);
        self.write(args);
    }

    pub fn build_log(&mut self, args: u8) {
        self.build_intrinsic_op(IntrinsicOperation::Log);
        self.write(args);
    }

    pub fn build_floor(&mut self, args: u8) {
        self.build_intrinsic_op(IntrinsicOperation::Floor);
        self.write(args);
    }

    pub fn build_cosh(&mut self, args: u8) {
        self.build_intrinsic_op(IntrinsicOperation::Cosh);
        self.write(args);
    }

    pub fn build_acos(&mut self, args: u8) {
        self.build_intrinsic_op(IntrinsicOperation::Acos);
        self.write(args);
    }

    pub fn build_cos(&mut self, args: u8) {
        self.build_intrinsic_op(IntrinsicOperation::Cos);
        self.write(args);
    }
}

#[derive(Copy, Clone)]
pub enum NamedExportKind {
    Local { loc_id: u16, ident_id: u16 },
    Global { ident_id: u16 },
}

pub fn compile_local_load_into(out: &mut Vec<u8>, index: u16, is_extern: bool) {
    let (thin, wide) = if is_extern {
        (Instruction::LdLocalExt, Instruction::LdLocalExtW)
    } else {
        (Instruction::LdLocal, Instruction::LdLocalW)
    };

    if let Ok(index) = u8::try_from(index) {
        out.push(thin as u8);
        out.push(index);
    } else {
        out.push(wide as u8);
        out.extend_from_slice(&index.to_ne_bytes());
    }
}

/// Convenience function for creating a vec and calling `compile_local_load_into`.
pub fn compile_local_load(index: u16, is_extern: bool) -> Vec<u8> {
    let mut out = Vec::new();
    compile_local_load_into(&mut out, index, is_extern);
    out
}
