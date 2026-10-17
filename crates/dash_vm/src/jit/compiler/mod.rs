use std::mem::offset_of;

use dash_middle::compiler::instruction::Instruction;

use crate::frame::Ip;
use crate::jit::compiler::backend::{
    PrologueData, REG_ARG1, REG_ARG2, REG_ARG3, REG_ARG4, REG_RETURN32, REG_RETURN64, REG_STACK_RELATIVE, Register,
    call_reg, cmp_reg_al_imm8, codegen_epilogue, codegen_prologue, je_block, je_branch_target, je_synthetic_label,
    jmp_block, jmp_branch_target, jmp_synthetic_label, jne_block, jne_synthetic_label, mov_mem_imm32, mov_reg_imm32,
    mov_reg_mem_u8, mov_reg_reg, register_synthetic_block, register_user_block, test_reg_reg, usable_callee_saved_regs,
    usable_caller_saved_regs,
};
use crate::jit::compiler::codebuffer::CodeBuffer;
use crate::jit::compiler::jumpresolver::{CpuIp, SyntheticLabel};
use crate::jit::dataflow::{BlockState, DataflowEngine};
use crate::jit::ir::{BasicBlockId, Body, BranchTarget, ConditionalGotoPopPolicy, GenericStmt, Stmt, Terminator};
use crate::jit::mmap::MmapFn;
use crate::jit::{JitOutData, JitVtable};

mod backend;
mod codebuffer;
mod jumpresolver;

struct CodegenCx<'a> {
    callee_regs: Vec<Register>,
    caller_regs: Vec<Register>,
    body: &'a Body,
    dataflow: &'a DataflowEngine,
    buf: CodeBuffer,
    prologue: PrologueData,
    /// Start IP of the codegen region *relative to the full function's bytecode*.
    start: Ip,
}

impl<'a> CodegenCx<'a> {
    fn caller_reg(&mut self) -> Register {
        self.caller_regs
            .pop()
            .expect("No more caller-saved registers available")
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

    /// Emits a call to the handler stub that forwards to the dispatch handler. Used as a fallback for unspecialized instructions.
    fn codegen_handler_stub(&mut self, instr: Instruction, instruction_ip: Ip) {
        // TODO: make all of these methods on CodeBuffer?
        mov_reg_reg(&mut self.buf, REG_ARG1, self.prologue.vm_csreg);
        mov_reg_mem_u8(
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
            match self.body.blocks[block].stmts[stmt] {
                Stmt::Generic(ref generic_stmt, ip) => match *generic_stmt {
                    GenericStmt::LdLocal(_) => self.codegen_handler_stub(Instruction::LdLocal, ip),
                    GenericStmt::StoreLocal { .. } => self.codegen_handler_stub(Instruction::StoreLocal, ip),
                    GenericStmt::LtNumLConstR32(_) | GenericStmt::PostfixIncLocalNum(_) => {
                        self.codegen_handler_stub(Instruction::IntrinsicOp, ip)
                    }
                    GenericStmt::Number(_) => self.codegen_handler_stub(Instruction::Number, ip),
                    GenericStmt::Pop => self.codegen_handler_stub(Instruction::Pop, ip),
                    GenericStmt::Add => self.codegen_handler_stub(Instruction::Add, ip),
                },
                Stmt::UncheckedNumberAdd => todo!(
                    "Implement the actual first specialized instruction operating directly on the stack top doing unchecked fadds"
                ),
            }
        }

        let terminator = self.body.blocks[block].terminator();
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
                let check_last_value_fn = self.caller_reg();

                mov_reg_mem_u8(
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

pub fn compile(body: &Body, start_ip: Ip, end_ip: Ip, dataflow: &DataflowEngine) -> MmapFn {
    let mut buf = CodeBuffer::new(body);
    let mut callee_regs = usable_callee_saved_regs();
    let caller_regs = usable_caller_saved_regs();

    let prologue = codegen_prologue(&mut buf, &mut callee_regs);

    let mut cx = CodegenCx {
        callee_regs,
        caller_regs,
        body,
        dataflow,
        buf,
        prologue,
        start: start_ip,
    };

    cx.codegen_body();

    // Exit block
    register_synthetic_block(&mut cx.buf, SyntheticLabel::Exit);

    // Set the ip to the end
    mov_reg_mem_u8(
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
