use std::env::temp_dir;
use std::mem::offset_of;

use dash_middle::compiler::instruction::Instruction;
use dash_middle::compiler::scope::BackLocalId;
use smallvec::SmallVec;

use crate::Vm;
use crate::frame::Ip;
use crate::jit::codegen::backend::{
    PrologueData, REG_ARG1, REG_ARG2, REG_ARG3, REG_ARG4, REG_RETURN32, REG_RETURN64, REG_STACK_RELATIVE, Register,
    Scale, add_reg_imm, addsd_reg_mem_disp, addsd_reg_mem_sib, and_reg_imm, call_reg, call_reg_disp, cmp_reg_al_imm8,
    codegen_epilogue, codegen_prologue, decq_mem, incq_mem, je_branch_target, jmp_block, jmp_branch_target,
    jmp_synthetic_label, jne_synthetic_label, mov_mem_imm32, mov_mem_reg_scaled, mov_reg_imm32, mov_reg_mem_disp,
    mov_reg_mem_sib, mov_reg_reg, movsd_mem_sib_reg, movsd_reg_mem_disp, movsd_reg_mem_sib, orq_mem_reg,
    patch_sites_to, register_synthetic_block, register_user_block, seta_reg, test_reg_reg, ucomisd_reg_mem_disp,
    ucomisd_reg_mem_sib, usable_callee_saved_regs, usable_caller_saved_gprs, usable_fprs, xor_reg_reg,
};
use crate::jit::codegen::codebuffer::CodeBuffer;
use crate::jit::codegen::jumpresolver::{CpuIp, SyntheticLabel};
use crate::jit::codegen::offsets::{VecOffsets, get_sp_offset, get_vec_offsets};
use crate::jit::dataflow::{DataflowEngine, StmtOrTerminator};
use crate::jit::ir::{
    BasicBlockId, Body, BranchTarget, ConditionalGotoCondition, ConditionalGotoPopPolicy, GenericStmt, Stmt,
    Terminator, UncheckedAssignmentKind, ValueSource,
};
use crate::jit::mmap::MmapFn;
use crate::jit::{JitOutData, JitVtable};
use crate::localscope::LocalScope;
use crate::value::Value;

mod backend;
mod codebuffer;
mod jumpresolver;
mod offsets;

type Constants = Vec<(u64, SmallVec<[CpuIp; 1]>)>;

#[derive(Debug, Copy, Clone)]
struct StackLenRelative(i32);

#[derive(Debug, Copy, Clone)]
enum ResolvedValueSource {
    Stack(StackLenRelative),
    Float(f64),
}

#[derive(Debug, Copy, Clone)]
enum LengthChange {
    Dec,
    None,
    Inc,
}

struct OpStackLayout {
    lhs: ResolvedValueSource,
    rhs: ResolvedValueSource,
    result_offset: i32,
    len_change: LengthChange,
}

impl OpStackLayout {
    fn for_binop(lhs: ValueSource, rhs: ValueSource) -> Self {
        match (lhs, rhs) {
            (ValueSource::Float(lhs), ValueSource::Float(rhs)) => Self {
                lhs: ResolvedValueSource::Float(lhs),
                rhs: ResolvedValueSource::Float(rhs),
                result_offset: 0,
                len_change: LengthChange::Inc,
            },
            (ValueSource::Float(lhs), ValueSource::Stack) => Self {
                lhs: ResolvedValueSource::Float(lhs),
                rhs: ResolvedValueSource::Stack(StackLenRelative(-(size_of::<usize>() as i32))),
                result_offset: -(size_of::<usize>() as i32),
                len_change: LengthChange::None,
            },
            (ValueSource::Stack, ValueSource::Float(rhs)) => Self {
                lhs: ResolvedValueSource::Stack(StackLenRelative(-(size_of::<usize>() as i32))),
                rhs: ResolvedValueSource::Float(rhs),
                result_offset: -(size_of::<usize>() as i32),
                len_change: LengthChange::None,
            },
            (ValueSource::Stack, ValueSource::Stack) => Self {
                lhs: ResolvedValueSource::Stack(StackLenRelative(-(size_of::<usize>() as i32) * 2)),
                rhs: ResolvedValueSource::Stack(StackLenRelative(-(size_of::<usize>() as i32))),
                result_offset: -(size_of::<usize>() as i32) * 2,
                len_change: LengthChange::Dec,
            },
        }
    }
}

struct CodegenCx<'a> {
    callee_regs: Vec<Register>,
    caller_regs: Vec<Register>,
    fprs: Vec<Register>,
    body: &'a Body,
    dataflow: &'a DataflowEngine,
    buf: CodeBuffer,
    prologue: PrologueData,
    /// Start IP of the codegen region *relative to the full function's bytecode*.
    start: Ip,
    stack_vec_offsets: VecOffsets,
    sp_field_offset: i32,
    constants: Constants,
    max_stack_depth: usize,
}

impl<'a> CodegenCx<'a> {
    fn caller_gpr64(&mut self) -> Register {
        self.caller_regs
            .pop()
            .expect("No more caller-saved registers available")
    }
    pub fn fpr(&mut self) -> Register {
        self.fprs.pop().expect("No more floating-point registers available")
    }
    #[expect(dead_code)]
    fn callee_reg64(&mut self) -> Register {
        self.callee_regs
            .pop()
            .expect("No more callee-saved registers available")
    }
    fn add_caller_reg(&mut self, reg: Register) {
        self.caller_regs.push(reg);
    }
    #[expect(dead_code)]
    fn add_callee_reg(&mut self, reg: Register) {
        self.callee_regs.push(reg);
    }
    fn add_fpr(&mut self, reg: Register) {
        self.fprs.push(reg);
    }

    /// Emits a call to the handler stub that forwards to the dispatch handler. Used as a fallback for unspecialized instructions.
    fn codegen_handler_stub(&mut self, instr: Instruction, instruction_ip: Ip) {
        // TODO: make all of these methods on CodeBuffer?
        mov_reg_reg(&mut self.buf, REG_ARG1, self.prologue.vm_csreg);
        mov_reg_mem_disp(
            &mut self.buf,
            REG_ARG2,
            REG_STACK_RELATIVE,
            -self.prologue.out_data_rbp_offset,
        );
        mov_reg_imm32(&mut self.buf, REG_ARG3, instr as u32);
        mov_reg_imm32(&mut self.buf, REG_ARG4, (self.start + instruction_ip).0 + 1);
        call_reg(&mut self.buf, self.prologue.stub_csreg);
        test_reg_reg(&mut self.buf, REG_RETURN32, REG_RETURN32);
        jne_synthetic_label(&mut self.buf, SyntheticLabel::StubStatusHandler);
    }

    fn codegen_ld_stack_base(&mut self, dest: Register) -> usize {
        let stack_ptr_offset = offset_of!(Vm, stack) + self.stack_vec_offsets.ptr;

        mov_reg_mem_disp(
            &mut self.buf,
            dest,
            self.prologue.vm_csreg,
            stack_ptr_offset.try_into().unwrap(),
        );

        stack_ptr_offset
    }

    fn stack_len_offset(&self) -> i32 {
        (offset_of!(Vm, stack) + self.stack_vec_offsets.len) as i32
    }

    fn codegen_ld_stack_len(&mut self, dest: Register) {
        let stack_offset = self.stack_len_offset();
        mov_reg_mem_disp(&mut self.buf, dest, self.prologue.vm_csreg, stack_offset);
    }

    fn codegen_u64_rip_offset(&mut self, value: u64, patch_site: u32) {
        if let Some((_, ips)) = self.constants.iter_mut().find(|&&mut (c, _)| c == value) {
            ips.push(patch_site);
        } else {
            self.constants.push((value, smallvec::smallvec![patch_site]));
        }
    }

    fn codegen_f64_rip_offset(&mut self, value: f64, patch_site: u32) {
        self.codegen_u64_rip_offset(value.to_bits(), patch_site);
    }

    /// Registers an f64 constant and generates a memory load from the constant pool
    fn codegen_ld_f64(&mut self, dest: Register, value: f64) {
        let patch_site = movsd_reg_mem_disp(&mut self.buf, dest, Register::Rip, 0);

        self.codegen_f64_rip_offset(value, patch_site);
    }

    fn apply_length_change(&mut self, len_change: LengthChange, stack_len_offset: i32) {
        match len_change {
            LengthChange::Dec => {
                decq_mem(&mut self.buf, self.prologue.vm_csreg, stack_len_offset);
            }
            LengthChange::Inc => {
                incq_mem(&mut self.buf, self.prologue.vm_csreg, stack_len_offset);
            }
            LengthChange::None => {}
        }
    }

    fn codegen_local_stack_index(&mut self, BackLocalId(local): BackLocalId, stack_index: Register) {
        let stack_index32 = stack_index.to_32();
        mov_reg_mem_disp(
            &mut self.buf,
            stack_index32,
            self.prologue.vm_csreg,
            self.sp_field_offset,
        );
        add_reg_imm(&mut self.buf, stack_index32, local.into());
    }

    fn codegen_block(&mut self, block: BasicBlockId) {
        // If the user block has already been registered, we don't need to codegen again.
        if !register_user_block(&mut self.buf, block) {
            return;
        }

        let mut cursor = self.dataflow.block_cursor(self.body, block);
        for stmt in self.body.blocks[block].stmts.iter_indices() {
            let before_stmt_state = cursor.seek_to_before(self.body, StmtOrTerminator::Stmt(stmt));
            self.max_stack_depth = self.max_stack_depth.max(before_stmt_state.stack.len());

            match self.body.blocks[block].stmts[stmt] {
                Stmt::Generic(ref generic_stmt, ip) => match *generic_stmt {
                    GenericStmt::StoreLocal { .. } => self.codegen_handler_stub(Instruction::StoreLocal, ip),
                    GenericStmt::LtNumLConstR32(_)
                    | GenericStmt::PostfixIncLocalNum(_)
                    | GenericStmt::PostfixDecLocalNum(_)
                    | GenericStmt::PrefixIncLocalNum(_)
                    | GenericStmt::PrefixDecLocalNum(_)
                    | GenericStmt::AddNumLR
                    | GenericStmt::SubNumLR
                    | GenericStmt::MulNumLR
                    | GenericStmt::DivNumLR
                    | GenericStmt::RemNumLR
                    | GenericStmt::PowNumLR
                    | GenericStmt::GtNumLR
                    | GenericStmt::GeNumLR
                    | GenericStmt::LtNumLR
                    | GenericStmt::LeNumLR
                    | GenericStmt::EqNumLR
                    | GenericStmt::NeNumLR
                    | GenericStmt::BitOrNumLR
                    | GenericStmt::BitXorNumLR
                    | GenericStmt::BitAndNumLR
                    | GenericStmt::BitShlNumLR
                    | GenericStmt::BitShrNumLR
                    | GenericStmt::BitUshrNumLR
                    | GenericStmt::GtNumLConstR(_)
                    | GenericStmt::GtNumLConstR32(_)
                    | GenericStmt::GeNumLConstR(_)
                    | GenericStmt::GeNumLConstR32(_)
                    | GenericStmt::LtNumLConstR(_)
                    | GenericStmt::LeNumLConstR(_)
                    | GenericStmt::LeNumLConstR32(_) => self.codegen_handler_stub(Instruction::IntrinsicOp, ip),
                    GenericStmt::Number(_) => self.codegen_handler_stub(Instruction::Number, ip),
                    GenericStmt::Lt => self.codegen_handler_stub(Instruction::Lt, ip),
                    GenericStmt::Add => self.codegen_handler_stub(Instruction::Add, ip),
                    GenericStmt::Mul => self.codegen_handler_stub(Instruction::Mul, ip),
                    GenericStmt::Sub => self.codegen_handler_stub(Instruction::Sub, ip),
                    GenericStmt::Div => self.codegen_handler_stub(Instruction::Div, ip),
                    GenericStmt::Rem => self.codegen_handler_stub(Instruction::Rem, ip),
                    GenericStmt::Pow => self.codegen_handler_stub(Instruction::Pow, ip),
                    GenericStmt::Gt => self.codegen_handler_stub(Instruction::Gt, ip),
                    GenericStmt::Ge => self.codegen_handler_stub(Instruction::Ge, ip),
                    GenericStmt::Le => self.codegen_handler_stub(Instruction::Le, ip),
                    GenericStmt::Eq => self.codegen_handler_stub(Instruction::Eq, ip),
                    GenericStmt::Ne => self.codegen_handler_stub(Instruction::Ne, ip),
                    GenericStmt::StrictEq => self.codegen_handler_stub(Instruction::StrictEq, ip),
                    GenericStmt::StrictNe => self.codegen_handler_stub(Instruction::StrictNe, ip),
                    GenericStmt::BitOr => self.codegen_handler_stub(Instruction::BitOr, ip),
                    GenericStmt::BitXor => self.codegen_handler_stub(Instruction::BitXor, ip),
                    GenericStmt::BitAnd => self.codegen_handler_stub(Instruction::BitAnd, ip),
                    GenericStmt::BitShl => self.codegen_handler_stub(Instruction::BitShl, ip),
                    GenericStmt::BitShr => self.codegen_handler_stub(Instruction::BitShr, ip),
                    GenericStmt::BitUshr => self.codegen_handler_stub(Instruction::BitUshr, ip),
                    GenericStmt::ObjIn => self.codegen_handler_stub(Instruction::ObjIn, ip),
                    GenericStmt::InstanceOf => self.codegen_handler_stub(Instruction::InstanceOf, ip),
                    GenericStmt::Pos => self.codegen_handler_stub(Instruction::Pos, ip),
                    GenericStmt::Neg => self.codegen_handler_stub(Instruction::Neg, ip),
                    GenericStmt::Not => self.codegen_handler_stub(Instruction::Not, ip),
                    GenericStmt::BitNot => self.codegen_handler_stub(Instruction::BitNot, ip),
                    GenericStmt::TypeOf => self.codegen_handler_stub(Instruction::TypeOf, ip),
                    GenericStmt::CallSymbolIterator => self.codegen_handler_stub(Instruction::CallSymbolIterator, ip),
                    GenericStmt::CallForInIterator => self.codegen_handler_stub(Instruction::CallForInIterator, ip),
                    GenericStmt::DeletePropertyStatic => {
                        self.codegen_handler_stub(Instruction::DeletePropertyStatic, ip)
                    }
                    GenericStmt::DeletePropertyDynamic => {
                        self.codegen_handler_stub(Instruction::DeletePropertyDynamic, ip)
                    }
                    GenericStmt::String => self.codegen_handler_stub(Instruction::String, ip),
                    GenericStmt::Boolean => self.codegen_handler_stub(Instruction::Boolean, ip),
                    GenericStmt::Regex => self.codegen_handler_stub(Instruction::Regex, ip),
                    GenericStmt::Function => self.codegen_handler_stub(Instruction::Function, ip),
                    GenericStmt::Null => self.codegen_handler_stub(Instruction::Null, ip),
                    GenericStmt::Undefined => self.codegen_handler_stub(Instruction::Undefined, ip),
                    GenericStmt::Undef => self.codegen_handler_stub(Instruction::Undef, ip),
                    GenericStmt::Nan => self.codegen_handler_stub(Instruction::Nan, ip),
                    GenericStmt::Infinity => self.codegen_handler_stub(Instruction::Infinity, ip),
                    GenericStmt::Global => self.codegen_handler_stub(Instruction::Global, ip),
                    GenericStmt::This => self.codegen_handler_stub(Instruction::This, ip),
                    GenericStmt::NewTarget => self.codegen_handler_stub(Instruction::NewTarget, ip),
                    GenericStmt::LdGlobal => self.codegen_handler_stub(Instruction::LdGlobal, ip),
                    GenericStmt::TypeOfGlobalIdent => self.codegen_handler_stub(Instruction::TypeOfGlobalIdent, ip),
                    GenericStmt::LdLocalExt => self.codegen_handler_stub(Instruction::LdLocalExt, ip),
                    GenericStmt::Nop => self.codegen_handler_stub(Instruction::Nop, ip),
                    GenericStmt::Debugger => self.codegen_handler_stub(Instruction::Debugger, ip),
                    GenericStmt::ExportNamed => self.codegen_handler_stub(Instruction::ExportNamed, ip),
                    GenericStmt::ImportStatic { .. } => self.codegen_handler_stub(Instruction::ImportStatic, ip),
                    GenericStmt::BindThis => self.codegen_handler_stub(Instruction::BindThis, ip),
                    GenericStmt::ExportDefault => self.codegen_handler_stub(Instruction::ExportDefault, ip),
                    GenericStmt::ImportDyn => self.codegen_handler_stub(Instruction::ImportDyn, ip),
                    GenericStmt::Throw => self.codegen_handler_stub(Instruction::Throw, ip),
                    GenericStmt::Call { .. } => self.codegen_handler_stub(Instruction::Call, ip),
                    GenericStmt::StoreGlobal { .. } => self.codegen_handler_stub(Instruction::StoreGlobal, ip),
                    GenericStmt::StoreLocalExt { .. } => self.codegen_handler_stub(Instruction::StoreLocalExt, ip),
                    GenericStmt::StaticPropAssign { .. } => {
                        self.codegen_handler_stub(Instruction::StaticPropAssign, ip)
                    }
                    GenericStmt::DynamicPropAssign { .. } => {
                        self.codegen_handler_stub(Instruction::DynamicPropAssign, ip)
                    }
                },
                Stmt::UncheckedNumberAdd { lhs, rhs } => {
                    let OpStackLayout {
                        lhs,
                        rhs,
                        result_offset,
                        len_change,
                    } = OpStackLayout::for_binop(lhs, rhs);

                    let len_reg = self.caller_gpr64();
                    let stack_ptr_reg = self.caller_gpr64();
                    let fpr = self.fpr();

                    self.codegen_ld_stack_base(stack_ptr_reg);
                    self.codegen_ld_stack_len(len_reg);

                    match lhs {
                        ResolvedValueSource::Stack(StackLenRelative(lhs)) => {
                            movsd_reg_mem_sib(
                                &mut self.buf,
                                fpr,
                                stack_ptr_reg,
                                len_reg,
                                Scale::target_pointer_with(),
                                lhs,
                            );
                        }
                        ResolvedValueSource::Float(lhs) => {
                            self.codegen_ld_f64(fpr, lhs);
                        }
                    }

                    match rhs {
                        ResolvedValueSource::Stack(StackLenRelative(rhs)) => {
                            addsd_reg_mem_sib(
                                &mut self.buf,
                                fpr,
                                stack_ptr_reg,
                                len_reg,
                                Scale::target_pointer_with(),
                                rhs,
                            );
                        }
                        ResolvedValueSource::Float(rhs) => {
                            let patch_site = addsd_reg_mem_disp(&mut self.buf, fpr, Register::Rip, 0);
                            self.codegen_f64_rip_offset(rhs, patch_site);
                        }
                    }

                    movsd_mem_sib_reg(
                        &mut self.buf,
                        fpr,
                        stack_ptr_reg,
                        len_reg,
                        Scale::target_pointer_with(),
                        result_offset,
                    );

                    let stack_len_offset = self.stack_len_offset();
                    self.apply_length_change(len_change, stack_len_offset);

                    self.add_fpr(fpr);
                    self.add_caller_reg(stack_ptr_reg);
                    self.add_caller_reg(len_reg);
                }
                Stmt::UncheckedNumberLt { lhs, rhs } => {
                    let OpStackLayout {
                        lhs,
                        rhs,
                        result_offset,
                        len_change,
                    } = OpStackLayout::for_binop(lhs, rhs);

                    let stack_ptr_reg = self.caller_gpr64();
                    let len_reg = self.caller_gpr64();
                    let status_reg = self.caller_gpr64();

                    let status32_reg = status_reg.to_32();
                    let status8_reg = status_reg.to_8();
                    let fpr = self.fpr();

                    self.codegen_ld_stack_base(stack_ptr_reg);
                    self.codegen_ld_stack_len(len_reg);

                    match rhs {
                        ResolvedValueSource::Stack(StackLenRelative(stack_off)) => {
                            movsd_reg_mem_sib(
                                &mut self.buf,
                                fpr,
                                stack_ptr_reg,
                                len_reg,
                                Scale::target_pointer_with(),
                                stack_off,
                            );
                        }
                        ResolvedValueSource::Float(rhs) => {
                            self.codegen_ld_f64(fpr, rhs);
                        }
                    }

                    xor_reg_reg(&mut self.buf, status32_reg, status32_reg);

                    match lhs {
                        ResolvedValueSource::Stack(StackLenRelative(stack_off)) => {
                            ucomisd_reg_mem_sib(
                                &mut self.buf,
                                fpr,
                                stack_ptr_reg,
                                len_reg,
                                Scale::target_pointer_with(),
                                stack_off,
                            );
                        }
                        ResolvedValueSource::Float(lhs) => {
                            let patch_site = ucomisd_reg_mem_disp(&mut self.buf, fpr, Register::Rip, 0);
                            self.codegen_f64_rip_offset(lhs, patch_site);
                        }
                    }

                    seta_reg(&mut self.buf, status8_reg);
                    let or_patch_site = orq_mem_reg(&mut self.buf, status_reg, Register::Rip, 0);
                    self.codegen_u64_rip_offset(Value::BOOLEAN_MASK, or_patch_site);

                    mov_mem_reg_scaled(
                        &mut self.buf,
                        status_reg,
                        stack_ptr_reg,
                        len_reg,
                        Scale::target_pointer_with(),
                        result_offset,
                    );

                    let stack_len_offset = self.stack_len_offset();
                    self.apply_length_change(len_change, stack_len_offset);

                    self.add_fpr(fpr);
                    self.add_caller_reg(len_reg);
                    self.add_caller_reg(stack_ptr_reg);
                }
                Stmt::Pop => {
                    let stack_len_offset = self.stack_len_offset();
                    self.apply_length_change(LengthChange::Dec, stack_len_offset);
                }
                Stmt::LdLocal(local) => {
                    let stack_base = self.caller_gpr64();
                    let stack_index = self.caller_gpr64();
                    let stack_len = self.caller_gpr64();

                    self.codegen_ld_stack_base(stack_base);
                    self.codegen_local_stack_index(local, stack_index);

                    mov_reg_mem_sib(
                        &mut self.buf,
                        stack_index,
                        stack_base,
                        stack_index,
                        Scale::target_pointer_with(),
                        0,
                    );

                    self.codegen_ld_stack_len(stack_len);

                    // Move result to stack top
                    mov_mem_reg_scaled(
                        &mut self.buf,
                        stack_index,
                        stack_base,
                        stack_len,
                        Scale::target_pointer_with(),
                        0,
                    );

                    // Increment stack length
                    let stack_len_offset = self.stack_len_offset();
                    self.apply_length_change(LengthChange::Inc, stack_len_offset);

                    self.add_caller_reg(stack_index);
                    self.add_caller_reg(stack_base);
                    self.add_caller_reg(stack_len);
                }
                Stmt::UncheckedStoreLocal { kind, local } => match kind {
                    UncheckedAssignmentKind::NumberPostfixIncrement => {
                        let stack_base = self.caller_gpr64();
                        let stack_index = self.caller_gpr64();
                        let stack_len = self.caller_gpr64();
                        let fpr = self.fpr();

                        // Load value from stack variable slot
                        self.codegen_ld_stack_base(stack_base);
                        self.codegen_local_stack_index(local, stack_index);

                        movsd_reg_mem_sib(
                            &mut self.buf,
                            fpr,
                            stack_base,
                            stack_index,
                            Scale::target_pointer_with(),
                            0,
                        );

                        // Increment
                        let patch_site = addsd_reg_mem_disp(&mut self.buf, fpr, Register::Rip, 0);
                        self.codegen_f64_rip_offset(1.0, patch_site);

                        // Write back result
                        self.codegen_ld_stack_len(stack_len);
                        movsd_mem_sib_reg(
                            &mut self.buf,
                            fpr,
                            stack_base,
                            stack_index,
                            Scale::target_pointer_with(),
                            0,
                        );
                        self.apply_length_change(LengthChange::Inc, self.stack_len_offset());

                        self.add_fpr(fpr);
                        self.add_caller_reg(stack_index);
                        self.add_caller_reg(stack_base);
                        self.add_caller_reg(stack_len);
                    }
                },
            }
        }

        let terminator = self.body.blocks[block].terminator();
        let before_terminator_state = cursor.seek_to_before(self.body, StmtOrTerminator::Terminator);
        self.max_stack_depth = self.max_stack_depth.max(before_terminator_state.stack.len());

        self.codegen_terminator(terminator);

        let after_terminator_state = cursor.seek_to_before(self.body, StmtOrTerminator::PostTerminator);
        self.max_stack_depth = self.max_stack_depth.max(after_terminator_state.stack.len());

        // NB: at this point, we've generated code for the successor blocks, so don't do anything block-specific here.
    }

    fn codegen_terminator(&mut self, terminator: &Terminator) {
        match *terminator {
            Terminator::Goto(target) => match target {
                BranchTarget::Block(target) => {
                    jmp_block(&mut self.buf, target);
                    self.codegen_block(target);
                }
                BranchTarget::Break => todo!(),
            },
            Terminator::ConditionalGoto {
                true_,
                false_,
                pop,
                condition,
            } => {
                match condition {
                    ConditionalGotoCondition::IsTrue | ConditionalGotoCondition::IsFalse => {
                        let stack_base = self.caller_gpr64();
                        let stack_len = self.caller_gpr64();
                        let testreg = self.caller_gpr64();

                        self.codegen_ld_stack_base(stack_base);

                        self.codegen_ld_stack_len(stack_len);

                        mov_reg_mem_sib(
                            &mut self.buf,
                            testreg,
                            stack_base,
                            stack_len,
                            Scale::target_pointer_with(),
                            -i32::try_from(size_of::<usize>()).unwrap(),
                        );

                        match pop {
                            ConditionalGotoPopPolicy::Pop => {
                                self.apply_length_change(LengthChange::Dec, self.stack_len_offset())
                            }
                            ConditionalGotoPopPolicy::Peek => {}
                        }

                        let true_mask = i32::try_from(Value::TRUE.raw() & !Value::BOOLEAN_MASK).unwrap();

                        and_reg_imm(&mut self.buf, testreg, true_mask);
                        match condition {
                            ConditionalGotoCondition::IsFalse => {
                                je_branch_target(&mut self.buf, true_);
                                jmp_branch_target(&mut self.buf, false_);
                            }
                            ConditionalGotoCondition::IsTrue => {
                                je_branch_target(&mut self.buf, false_);
                                jmp_branch_target(&mut self.buf, true_);
                            }
                            _ => unreachable!(),
                        }

                        self.add_caller_reg(stack_base);
                        self.add_caller_reg(stack_len);
                        self.add_caller_reg(testreg);

                        if let BranchTarget::Block(target) = true_ {
                            self.codegen_block(target);
                        }
                        if let BranchTarget::Block(target) = false_ {
                            self.codegen_block(target);
                        }
                    }
                    ConditionalGotoCondition::IsFalsy
                    | ConditionalGotoCondition::IsTruthy
                    | ConditionalGotoCondition::IsUndefined
                    | ConditionalGotoCondition::IsNullish => {
                        // Fallback

                        mov_reg_reg(&mut self.buf, REG_ARG1, self.prologue.vm_csreg);
                        mov_reg_imm32(&mut self.buf, REG_ARG2, pop as u32);
                        mov_reg_imm32(&mut self.buf, REG_ARG3, condition as u32);
                        call_reg_disp(
                            &mut self.buf,
                            self.prologue.vtable_csreg,
                            offset_of!(JitVtable, check_last_value).try_into().unwrap(),
                        );
                        cmp_reg_al_imm8(&mut self.buf, 1);
                        je_branch_target(&mut self.buf, true_);
                        jmp_branch_target(&mut self.buf, false_);

                        if let BranchTarget::Block(target) = true_ {
                            self.codegen_block(target);
                        }
                        if let BranchTarget::Block(target) = false_ {
                            self.codegen_block(target);
                        }
                    }
                }
            }
        }
    }

    fn codegen_body(&mut self) {
        self.codegen_block(self.body.root);
    }
}

/// Merges the machine code buffer with the constants, appending them at the end and patching the relative offsets to them.
fn patch_constant_loads(mut buf: CodeBuffer, mut constants: Constants) -> Vec<u8> {
    for (constant, patch_sites) in &mut constants {
        let offset = buf.offset();
        buf.extend_from_slice(&constant.to_ne_bytes());

        patch_sites_to(&mut buf, patch_sites, offset);
    }

    buf.buffer
}

pub struct CompileResult {
    pub func: MmapFn,
    pub max_stack_depth: usize,
}

pub fn compile(
    body: &Body,
    start_ip: Ip,
    end_ip: Ip,
    dataflow: &DataflowEngine,
    scope: &LocalScope<'_>,
) -> CompileResult {
    let mut buf = CodeBuffer::new(body);
    let mut callee_regs = usable_callee_saved_regs();
    let caller_regs = usable_caller_saved_gprs();
    let fprs = usable_fprs();

    let prologue = codegen_prologue(&mut buf, &mut callee_regs);

    let mut cx = CodegenCx {
        callee_regs,
        caller_regs,
        fprs,
        body,
        dataflow,
        buf,
        prologue,
        start: start_ip,
        stack_vec_offsets: get_vec_offsets(&scope.stack),
        sp_field_offset: get_sp_offset(scope).try_into().unwrap(),
        constants: Constants::new(),
        max_stack_depth: 0,
    };

    cx.codegen_body();

    // Exit block
    register_synthetic_block(&mut cx.buf, SyntheticLabel::Exit);

    // Set the ip to the end
    mov_reg_mem_disp(
        &mut cx.buf,
        REG_RETURN64,
        REG_STACK_RELATIVE,
        -cx.prologue.out_data_rbp_offset,
    );
    mov_mem_imm32(
        &mut cx.buf,
        REG_RETURN64,
        offset_of!(JitOutData, ip).try_into().unwrap(),
        end_ip.0.cast_signed(),
    );
    jmp_synthetic_label(&mut cx.buf, SyntheticLabel::Epilogue);

    // Stub error handler
    register_synthetic_block(&mut cx.buf, SyntheticLabel::StubStatusHandler);
    jmp_synthetic_label(&mut cx.buf, SyntheticLabel::Epilogue);

    // Epilogue
    register_synthetic_block(&mut cx.buf, SyntheticLabel::Epilogue);
    codegen_epilogue(&mut cx.buf, &cx.prologue);

    cx.buf.jumps.assert_resolved();

    // TODO: do this in the same fn as assert_resolved? move this stuff to the JumpResolver?
    let buffer = patch_constant_loads(cx.buf, cx.constants);

    if scope.params.experimental_objdump_jit {
        let mut bin_path = temp_dir();
        bin_path.push("dash_jit_out.bin");

        std::fs::write(&bin_path, &buffer).unwrap();
        eprintln!("------- Start of x86 blob -------");
        std::process::Command::new("objdump")
            .args(&["-D", "-b", "binary", "-m", "i386:x86-64", bin_path.to_str().unwrap()])
            .status()
            .unwrap();
        eprintln!("------- End of x86 blob -------");
    }

    CompileResult {
        func: MmapFn::alloc(&buffer),
        max_stack_depth: cx.max_stack_depth,
    }
}
