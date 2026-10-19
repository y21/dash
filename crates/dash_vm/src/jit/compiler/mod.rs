use std::mem::offset_of;

use dash_middle::compiler::instruction::Instruction;

use crate::Vm;
use crate::frame::Ip;
use crate::jit::compiler::backend::{
    PrologueData, REG_ARG1, REG_ARG2, REG_ARG3, REG_ARG4, REG_RETURN32, REG_RETURN64, REG_STACK_RELATIVE, Register,
    Scale, addsd_reg_mem_sib, call_reg, cmp_reg_al_imm8, codegen_epilogue, codegen_prologue, decq_mem, je_block,
    je_branch_target, je_synthetic_label, jmp_block, jmp_branch_target, jmp_synthetic_label, jne_block,
    jne_synthetic_label, mov_mem_imm32, mov_reg_imm32, mov_reg_mem, mov_reg_reg, movsd_mem_sib_reg, movsd_reg_mem_sib,
    register_synthetic_block, register_user_block, test_reg_reg, usable_callee_saved_regs, usable_caller_saved_gprs,
    usable_fprs,
};
use crate::jit::compiler::codebuffer::CodeBuffer;
use crate::jit::compiler::jumpresolver::{CpuIp, SyntheticLabel};
use crate::jit::compiler::offsets::{VecOffsets, get_vec_offsets};
use crate::jit::dataflow::{BlockState, DataflowEngine};
use crate::jit::ir::{BasicBlockId, Body, BranchTarget, ConditionalGotoPopPolicy, GenericStmt, Stmt, Terminator};
use crate::jit::mmap::MmapFn;
use crate::jit::{JitOutData, JitVtable};
use crate::localscope::LocalScope;

mod backend;
mod codebuffer;
mod jumpresolver;
mod offsets;

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
}

impl<'a> CodegenCx<'a> {
    fn caller_gpr(&mut self) -> Register {
        self.caller_regs
            .pop()
            .expect("No more caller-saved registers available")
    }
    pub fn fpr(&mut self) -> Register {
        self.fprs.pop().expect("No more floating-point registers available")
    }
    fn callee_reg(&mut self) -> Register {
        self.callee_regs
            .pop()
            .expect("No more callee-saved registers available")
    }
    fn add_caller_reg(&mut self, reg: Register) {
        self.caller_regs.push(reg);
    }
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
        mov_reg_mem(
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

    fn codegen_block(&mut self, block: BasicBlockId) {
        // If the user block has already been registered, we don't need to codegen again.
        if !register_user_block(&mut self.buf, block) {
            return;
        }

        for stmt in self.body.blocks[block].stmts.iter_indices() {
            println!(
                "Emitting {:?} at {:x?}",
                self.body.blocks[block].stmts[stmt],
                self.buf.offset()
            );
            match self.body.blocks[block].stmts[stmt] {
                Stmt::Generic(ref generic_stmt, ip) => match *generic_stmt {
                    GenericStmt::LdLocal(_) => self.codegen_handler_stub(Instruction::LdLocal, ip),
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
                    GenericStmt::Pop => self.codegen_handler_stub(Instruction::Pop, ip),
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
                Stmt::UncheckedNumberAdd => {
                    /*
                    vm->stack[vm->stack.len - 2] = vm->stack[vm->stack.len - 2] + vm->stack[vm->stack.len - 1];
                    vm->stack.len -= 1;

                    movsxd  rax, dword ptr [rdi + 8]                ;; mov len into rax, could just be mov eax, dword ptr [rdi + 8] - dont need sign extension
                    mov     rcx, qword ptr [rdi]                    ;; mov stack base into rcx

                    movsd   xmm0, qword ptr [rcx + 8*rax - 16]      ;; store pointer to the 2nd-to-last element (equiv to stack[stack.len() - 2])
                    addsd   xmm0, qword ptr [rcx + 8*rax - 8]       ;; add the top element (equiv to stack[stack.len() - 1]) to the 2nd-to-last element
                    movsd   qword ptr [rcx + 8*rax - 16], xmm0      ;; mov the sum (xmm0) back into the 2nd-to-last element (equiv to stack[stack.len() - 2])
                    dec     eax                                     ;; decrement the local stack length copy
                    mov     dword ptr [rdi + 8], eax                ;; write the new length back
                    ret

                                 */
                    // stack offset on the VM: offset_of!(Vm, stack)
                    let len_reg = self.caller_gpr();
                    let stack_ptr_reg = self.caller_gpr();
                    let fpr = self.fpr();

                    let stack_ptr_offset = offset_of!(Vm, stack) + self.stack_vec_offsets.ptr;
                    let stack_len_offset = offset_of!(Vm, stack) + self.stack_vec_offsets.len;

                    // Store stack base
                    mov_reg_mem(
                        &mut self.buf,
                        stack_ptr_reg,
                        self.prologue.vm_csreg,
                        stack_ptr_offset.try_into().unwrap(),
                    );

                    // Store len field
                    mov_reg_mem(
                        &mut self.buf,
                        len_reg,
                        self.prologue.vm_csreg,
                        stack_len_offset.try_into().unwrap(),
                    );

                    // Load second-to-last stack value as an f64
                    movsd_reg_mem_sib(
                        &mut self.buf,
                        fpr,
                        stack_ptr_reg,
                        len_reg,
                        Scale::target_pointer_with(),
                        -i32::try_from(size_of::<usize>() * 2).unwrap(),
                    );

                    // Add the top stack value (as an f64) to the second-to-last value
                    addsd_reg_mem_sib(
                        &mut self.buf,
                        fpr,
                        stack_ptr_reg,
                        len_reg,
                        Scale::target_pointer_with(),
                        -i32::try_from(size_of::<usize>() * 1).unwrap(),
                    );

                    // Write the result f64 back to the second-to-last stack value
                    movsd_mem_sib_reg(
                        &mut self.buf,
                        fpr,
                        stack_ptr_reg,
                        len_reg,
                        Scale::target_pointer_with(),
                        -i32::try_from(size_of::<usize>() * 2).unwrap(),
                    );

                    // Decrement length
                    decq_mem(
                        &mut self.buf,
                        self.prologue.vm_csreg,
                        stack_len_offset.try_into().unwrap(),
                    );

                    self.add_fpr(fpr);
                    self.add_caller_reg(stack_ptr_reg);
                    self.add_caller_reg(len_reg);
                }
            }
        }

        let terminator = self.body.blocks[block].terminator();
        println!("Emitting {:?} at {:x?}", terminator, self.buf.offset());
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
                let check_last_value_fn = self.caller_gpr();

                mov_reg_mem(
                    &mut self.buf,
                    check_last_value_fn,
                    self.prologue.vtable_csreg,
                    offset_of!(JitVtable, check_last_value).try_into().unwrap(),
                );
                mov_reg_reg(&mut self.buf, REG_ARG1, self.prologue.vm_csreg);
                mov_reg_imm32(&mut self.buf, REG_ARG2, pop as u32);
                mov_reg_imm32(&mut self.buf, REG_ARG3, condition as u32);
                call_reg(&mut self.buf, check_last_value_fn);
                cmp_reg_al_imm8(&mut self.buf, 1);

                self.add_caller_reg(check_last_value_fn);

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

        // NB: at this point, we've generated code for the successor blocks, so don't do anything block-specific here.
    }

    fn codegen_body(&mut self) {
        self.codegen_block(self.body.root);
    }
}

pub fn compile(body: &Body, start_ip: Ip, end_ip: Ip, dataflow: &DataflowEngine, scope: &LocalScope<'_>) -> MmapFn {
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
    };

    cx.codegen_body();

    // Exit block
    register_synthetic_block(&mut cx.buf, SyntheticLabel::Exit);

    // Set the ip to the end
    mov_reg_mem(
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

    std::fs::write("/tmp/dash_jit_out.bin", cx.buf.buffer()).unwrap();
    println!("------- Start of x86 blob -------");
    std::process::Command::new("objdump")
        .args(&["-D", "-b", "binary", "-m", "i386:x86-64", "/tmp/dash_jit_out.bin"])
        .status()
        .unwrap();
    println!("------- End of x86 blob -------");

    MmapFn::alloc(cx.buf.buffer())
}
