use std::rc::Rc;

use dash_middle::compiler::constant::{
    BooleanConstant, Function, FunctionConstant, LimitExceededError, NumberConstant, RegexConstant, SymbolConstant,
};
use dash_middle::compiler::instruction::{AssignKind, Instruction, IntrinsicOperation};
use dash_middle::compiler::{
    ExportPropertyKind, FunctionCallMetadata, ObjectMemberKind as CompilerObjectMemberKind, StaticImportKind,
};
use dash_middle::interner::Symbol;
use dash_middle::parser::error::Error;
use dash_middle::parser::expr::ObjectMemberKind;
use dash_middle::sourcemap::Span;

use super::builder::{InstructionBuilder, Label};

macro_rules! simple_instruction {
    ($($fname:ident $value:expr),*) => {
        $(
            pub fn $fname(&mut self) {
                self.write($value as u8);
            }
        )*
    }
}

impl InstructionBuilder<'_, '_> {
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
        build_bind_this Instruction::BindThis,
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
        build_arguments Instruction::Arguments,
        build_global Instruction::Global,
        build_infinity Instruction::Infinity,
        build_nan Instruction::Nan,
        build_undef Instruction::Undef,
        build_symbol_iterator Instruction::CallSymbolIterator,
        build_for_in_iterator Instruction::CallForInIterator,
        build_dynamic_delete Instruction::DeletePropertyDynamic
    }

    pub fn build_ret(&mut self, tc_depth: u16) {
        self.write_instr(Instruction::Ret);
        self.writew(tc_depth);
    }

    pub fn write_bool(&mut self, b: bool) {
        self.write(b.into());
    }

    pub fn build_try_block(&mut self, has_catch: bool, finally_id: Option<usize>) {
        self.write_instr(Instruction::Try);
        self.write_bool(has_catch);
        if has_catch {
            self.write_all(&[0, 0]);
        }
        // NOTE: even though we won't *really* perform a jump (we skip over the following `jump` instruction emitted by this call in the vm dispatcher)
        // we use the local jump resolving mechanism for updating the catch offset
        self.add_local_jump(Label::Catch);
        self.write_bool(finally_id.is_some());
        if let Some(finally_id) = finally_id {
            self.write_all(&[0, 0]);
            self.current_function_mut()
                .add_global_jump(Label::Finally { finally_id });
        }
    }

    pub fn build_local_load(&mut self, index: u16, is_extern: bool) {
        compile_local_load_into(&mut self.current_function_mut().buf, index, is_extern);
    }

    pub fn build_string_constant(&mut self, sym: Symbol) -> Result<(), LimitExceededError> {
        let SymbolConstant(id) = self.current_function_mut().cp.add_symbol(sym)?;
        self.write_instr(Instruction::String);
        self.writew(id);
        Ok(())
    }

    pub fn build_boolean_constant(&mut self, b: bool) -> Result<(), LimitExceededError> {
        let BooleanConstant(id) = self.current_function_mut().cp.add_boolean(b)?;
        self.write_instr(Instruction::Boolean);
        self.writew(id);
        Ok(())
    }

    pub fn build_number_constant(&mut self, n: f64) -> Result<(), LimitExceededError> {
        let NumberConstant(id) = self.current_function_mut().cp.add_number(n)?;
        self.write_instr(Instruction::Number);
        self.writew(id);
        Ok(())
    }

    pub fn build_regex_constant(
        &mut self,
        regex: dash_regex::ParsedRegex,
        flags: dash_regex::Flags,
        sym: Symbol,
    ) -> Result<(), LimitExceededError> {
        let RegexConstant(id) = self.current_function_mut().cp.add_regex((regex, flags, sym))?;
        self.write_instr(Instruction::Regex);
        self.writew(id);
        Ok(())
    }

    pub fn build_null_constant(&mut self) -> Result<(), LimitExceededError> {
        self.write_instr(Instruction::Null);
        Ok(())
    }

    pub fn build_undefined_constant(&mut self) -> Result<(), LimitExceededError> {
        self.write_instr(Instruction::Undefined);
        Ok(())
    }

    pub fn build_function_constant(&mut self, fun: Function) -> Result<(), LimitExceededError> {
        let FunctionConstant(id) = self.current_function_mut().cp.add_function(Rc::new(fun))?;
        self.write_instr(Instruction::Function);
        self.writew(id);
        Ok(())
    }

    pub fn build_global_load(&mut self, ident: Symbol) -> Result<(), LimitExceededError> {
        let SymbolConstant(id) = self.current_function_mut().cp.add_symbol(ident)?;
        self.write_instr(Instruction::LdGlobal);
        self.writew(id);
        Ok(())
    }

    pub fn build_global_store(&mut self, kind: AssignKind, ident: Symbol) -> Result<(), LimitExceededError> {
        let SymbolConstant(id) = self.current_function_mut().cp.add_symbol(ident)?;
        self.write_instr(Instruction::StoreGlobal);
        self.writew(id);
        self.write(kind as u8);
        Ok(())
    }

    pub fn build_local_store(&mut self, kind: AssignKind, id: u16, is_extern: bool) {
        if is_extern {
            self.write_instr(Instruction::StoreLocalExt);
        } else {
            self.write_instr(Instruction::StoreLocal);
        }
        self.writew(id);
        self.write(kind as u8);
    }

    pub fn build_call(&mut self, meta: FunctionCallMetadata, spread_arg_indices: Vec<u8>, target_span: Span) {
        let ip = self.current_function().buf.len();
        self.current_function_mut()
            .debug_symbols
            .add(ip.try_into().unwrap(), target_span);
        self.write_instr(Instruction::Call);
        self.write(meta.into());
        self.write(spread_arg_indices.len().try_into().unwrap());
        for index in spread_arg_indices {
            self.write(index);
        }
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

    pub fn build_static_prop_access(&mut self, ident: Symbol, preserve_this: bool) -> Result<(), LimitExceededError> {
        let SymbolConstant(id) = self.current_function_mut().cp.add_symbol(ident)?;
        self.write_instr(Instruction::StaticPropAccess);
        self.writew(id);
        self.write(preserve_this.into());

        Ok(())
    }

    pub fn build_dynamic_prop_access(&mut self, preserve_this: bool) {
        self.write_instr(Instruction::DynamicPropAccess);
        self.write(preserve_this.into());
    }

    pub fn build_static_prop_assign(&mut self, kind: AssignKind, ident: Symbol) -> Result<(), LimitExceededError> {
        let SymbolConstant(id) = self.current_function_mut().cp.add_symbol(ident)?;
        self.write_instr(Instruction::StaticPropAssign);
        self.write(kind as u8);
        self.writew(id);

        Ok(())
    }

    pub fn build_dynamic_prop_assign(&mut self, kind: AssignKind) {
        self.write_instr(Instruction::DynamicPropAssign);
        self.write(kind as u8);
    }

    pub fn build_arraylit(&mut self, len: u16, stack_values: u16) {
        self.write_instr(Instruction::ArrayLit);
        self.writew(len);
        self.writew(stack_values);
    }

    pub fn build_object_member_like_instruction(
        &mut self,
        span: Span,
        constants: Vec<ObjectMemberKind>,
        instr: Instruction,
    ) -> Result<(), Error> {
        let len: u16 = constants
            .len()
            .try_into()
            .map_err(|_| Error::ObjectLitLimitExceeded(span))?;

        self.write_instr(instr);
        self.writew(len);

        fn compile_object_member_kind(
            ib: &mut InstructionBuilder,
            span: Span, // TODO: this should not be the span of the obj literal but the member kind
            name: Symbol,
            kind_id: u8,
        ) -> Result<(), Error> {
            let SymbolConstant(id) = ib
                .current_function_mut()
                .cp
                .add_symbol(name)
                .map_err(|_| Error::ConstantPoolLimitExceeded(span))?;

            ib.write(kind_id);
            ib.writew(id);
            Ok(())
        }

        // Push in reverse order to match order in which the compiler pushes values onto the stack
        for member in constants.into_iter().rev() {
            let kind_id = CompilerObjectMemberKind::from(&member) as u8;
            match member {
                ObjectMemberKind::Dynamic(_)
                | ObjectMemberKind::DynamicGetter(_)
                | ObjectMemberKind::DynamicSetter(_) => self.write(kind_id),
                ObjectMemberKind::Static(name) => compile_object_member_kind(self, span, name, kind_id)?,
                ObjectMemberKind::Spread => self.write(kind_id),
                ObjectMemberKind::Getter(name) | ObjectMemberKind::Setter(name) => {
                    compile_object_member_kind(self, span, name, kind_id)?
                }
            }
        }

        Ok(())
    }

    pub fn build_static_import(&mut self, import: StaticImportKind, local_id: u16, path_id: SymbolConstant) {
        self.write_instr(Instruction::ImportStatic);
        self.write(import as u8);
        self.writew(local_id);
        self.writew(path_id.0);
    }

    pub fn build_dynamic_import(&mut self) {
        self.write_instr(Instruction::ImportDyn);
    }

    pub fn build_static_delete(&mut self, id: SymbolConstant) {
        self.write_instr(Instruction::DeletePropertyStatic);
        self.writew(id.0);
    }

    pub fn build_named_export(&mut self, span: Span, it: &[NamedExportKind]) -> Result<(), Error> {
        self.write_instr(Instruction::ExportNamed);

        let len = it
            .len()
            .try_into()
            .map_err(|_| Error::ExportNameListLimitExceeded(span))?;

        self.writew(len);

        for kind in it.iter().copied() {
            match kind {
                NamedExportKind::Local { loc_id, ident_id } => {
                    self.write(ExportPropertyKind::Local as u8);
                    self.writew(loc_id);
                    self.writew(ident_id.0);
                }
                NamedExportKind::Global { ident_id } => {
                    self.write(ExportPropertyKind::Global as u8);
                    self.writew(ident_id.0);
                }
            }
        }

        Ok(())
    }

    // TODO: encode this using Option<NonMaxU16>
    pub fn build_objdestruct(&mut self, count: u16, rest: Option<u16>) {
        self.write_instr(Instruction::ObjDestruct);
        self.writew(rest.map_or_else(
            || u16::MAX,
            |v| {
                assert!(v != u16::MAX);
                v
            },
        ));
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

    pub fn build_typeof_global_ident(&mut self, at: Span, ident: Symbol) -> Result<(), Error> {
        let SymbolConstant(id) = self
            .current_function_mut()
            .cp
            .add_symbol(ident)
            .map_err(|_| Error::ConstantPoolLimitExceeded(at))?;
        self.write_instr(Instruction::TypeOfGlobalIdent);
        self.writew(id);
        Ok(())
    }
}

#[derive(Copy, Clone)]
pub enum NamedExportKind {
    Local { loc_id: u16, ident_id: SymbolConstant },
    Global { ident_id: SymbolConstant },
}

pub fn compile_local_load_into(out: &mut Vec<u8>, index: u16, is_extern: bool) {
    if is_extern {
        out.push(Instruction::LdLocalExt as u8);
    } else {
        out.push(Instruction::LdLocal as u8);
    }
    out.extend_from_slice(&index.to_ne_bytes());
}

/// Convenience function for creating a vec and calling `compile_local_load_into`.
pub fn compile_local_load(index: u16, is_extern: bool) -> Vec<u8> {
    let mut out = Vec::new();
    compile_local_load_into(&mut out, index, is_extern);
    out
}
