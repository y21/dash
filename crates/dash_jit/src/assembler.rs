use std::collections::HashMap;
use std::ffi::CStr;
use std::fmt;
use std::mem;
use std::ptr;

use cstr::cstr;
use dash_middle::compiler::instruction::ADD;
use dash_middle::compiler::instruction::CONSTANT;
use dash_middle::compiler::instruction::JMP;
use dash_middle::compiler::instruction::JMPFALSENP;
use dash_middle::compiler::instruction::JMPFALSEP;
use dash_middle::compiler::instruction::JMPNULLISHNP;
use dash_middle::compiler::instruction::JMPNULLISHP;
use dash_middle::compiler::instruction::JMPTRUENP;
use dash_middle::compiler::instruction::JMPTRUEP;
use dash_middle::compiler::instruction::LDLOCAL;
use dash_middle::compiler::instruction::LDLOCALW;
use dash_middle::compiler::instruction::LT;
use dash_middle::compiler::instruction::MUL;
use dash_middle::compiler::instruction::POP;
use dash_middle::compiler::instruction::REVSTCK;
use dash_middle::compiler::instruction::STORELOCAL;
use llvm_sys::analysis::LLVMVerifierFailureAction;
use llvm_sys::analysis::LLVMVerifyFunction;
use llvm_sys::core::LLVMAddFunction;
use llvm_sys::core::LLVMAddIncoming;
use llvm_sys::core::LLVMAppendBasicBlock;
use llvm_sys::core::LLVMBuildAdd;
use llvm_sys::core::LLVMBuildAlloca;
use llvm_sys::core::LLVMBuildBr;
use llvm_sys::core::LLVMBuildCondBr;
use llvm_sys::core::LLVMBuildGEP2;
use llvm_sys::core::LLVMBuildICmp;
use llvm_sys::core::LLVMBuildLoad2;
use llvm_sys::core::LLVMBuildMul;
use llvm_sys::core::LLVMBuildPhi;
use llvm_sys::core::LLVMBuildRet;
use llvm_sys::core::LLVMBuildRetVoid;
use llvm_sys::core::LLVMBuildStore;
use llvm_sys::core::LLVMConstInt;
use llvm_sys::core::LLVMCreateBuilder;
use llvm_sys::core::LLVMCreatePassManager;
use llvm_sys::core::LLVMDisposeModule;
use llvm_sys::core::LLVMFunctionType;
use llvm_sys::core::LLVMGetParam;
use llvm_sys::core::LLVMInt64Type;
use llvm_sys::core::LLVMModuleCreateWithName;
use llvm_sys::core::LLVMPointerType;
use llvm_sys::core::LLVMPositionBuilderAtEnd;
use llvm_sys::core::LLVMPrintModuleToString;
use llvm_sys::core::LLVMRunFunctionPassManager;
use llvm_sys::core::LLVMRunPassManager;
use llvm_sys::core::LLVMVoidType;
use llvm_sys::execution_engine::LLVMCreateExecutionEngineForModule;
use llvm_sys::execution_engine::LLVMGetFunctionAddress;
use llvm_sys::prelude::*;
use llvm_sys::LLVMIntPredicate;
use llvm_sys::transforms::pass_manager_builder::LLVMPassManagerBuilderCreate;
use llvm_sys::transforms::pass_manager_builder::LLVMPassManagerBuilderPopulateFunctionPassManager;
use llvm_sys::transforms::pass_manager_builder::LLVMPassManagerBuilderPopulateModulePassManager;
use llvm_sys::transforms::pass_manager_builder::LLVMPassManagerBuilderSetOptLevel;

type JitFunction = unsafe extern "C" fn(*mut i64) -> i64;

const EMPTY: *const i8 = cstr!("").as_ptr();

use crate::trace::Trace;

pub struct Assembler {
    module: LLVMModuleRef,
}

/// Trait that users of the assembler must use.
///
/// Currently, only integers are supported.
pub trait AssemblerQuery {
    fn get_local(&self, id: u16) -> i64;
    fn get_constant(&self, id: u16) -> i64;
    fn update_ip(&mut self, ip: usize);
}

impl Assembler {
    pub fn new() -> Self {
        let module = unsafe { LLVMModuleCreateWithName(cstr!("dash_jit").as_ptr()) };
        Self { module }
    }

    pub fn compile_trace<A: AssemblerQuery>(&self, trace: Trace, bytecode: Vec<u8>, mut query: A) {
        // The idea for jitted function is simple:
        //
        // Functions will always have the signature: void jit(i64*);
        // The array of i64s is all of the input variables, i.e. referenced (and possibly mutated) locals
        // within this trace.
        // (Note: currently, with this idea, we can only support traces that only reference integers)
        //
        // The function will then allocate space for all input variables through llvm's alloca instruction
        // and copy the parameter into each of the variables, at the beginning, in an `entry` block.
        //
        // The machine code can then access and mutate this specific stack variable throughout its execution.
        //
        // Later, when reaching an exit point, it "synchronizes" the state by writing the alloca'd variables
        // back to the parameter pointer.
        // In Rust we can then see the changes in the passed array and simply update the VM stack with all of the new values.

        let (int64, int64ptr) = unsafe { (LLVMInt64Type(), LLVMPointerType(LLVMInt64Type(), 0)) };

        let fun = unsafe {
            let mut params = [int64ptr];
            let funty = LLVMFunctionType(int64, params.as_mut_ptr(), 1, 0);

            LLVMAddFunction(self.module, cstr!("jit").as_ptr(), funty)
        };

        let mut bytecode = bytecode.into_iter().enumerate();

        let mut paths = trace.conditional_jumps.into_iter();

        // A stack of LLVM values, for building instructions that depend on temporaries
        let mut stack = Vec::new();

        // This HashMap maps the LDLOCAL index to a LLVM value, which can be used for building instructions
        // The LLVM value refers to alloca'd stack space in the entry block.
        let mut locals: HashMap<usize, LLVMValueRef> = HashMap::new();

        // This vector stores all referenced locals, in the exact order as they appeared.
        let mut local_values: Vec<(u16, i64)> = Vec::new();

        let mut labels = HashMap::new();

        // A vector of (target_ip_of_jump, pred_basic_block) that have an exit guard in them.
        // This is later used in the exit block to build a phi to tell the interpreter where it needs to resume
        // executing bytecode
        let (mut exit_ips, mut exit_pred_blocks) = (Vec::new(), Vec::new());

        let mut current_block;
        let mut current_builder;

        let entry_block;
        let entry_builder;

        let trace_start_block;
        let trace_start_builder;

        let exit_block;
        let exit_builder;

        unsafe {
            // The entry/setup block is the block in which all of the local variables are allocated.
            entry_block = LLVMAppendBasicBlock(fun, cstr!("empty").as_ptr());
            entry_builder = LLVMCreateBuilder();
            LLVMPositionBuilderAtEnd(entry_builder, entry_block);
            
            trace_start_block = LLVMAppendBasicBlock(fun, cstr!("trace_start").as_ptr());
            trace_start_builder = LLVMCreateBuilder();
            LLVMPositionBuilderAtEnd(trace_start_builder, trace_start_block);
            labels.insert(0, (trace_start_block, trace_start_builder));

            current_block = trace_start_block;
            current_builder = trace_start_builder;

            exit_block = LLVMAppendBasicBlock(fun, cstr!("exit").as_ptr());
            exit_builder = LLVMCreateBuilder();
            LLVMPositionBuilderAtEnd(exit_builder, exit_block);
        }

        let mut ssa_bytecode = Vec::with_capacity(bytecode.len());

        while let Some((bytecode_index, inst)) = bytecode.next() {
            // Every instruction is always copied into the SSA-form bytecode
            ssa_bytecode.push(inst);

            match inst {
                JMP => {
                    let operands = [bytecode.next().unwrap().1, bytecode.next().unwrap().1];
                    let count = i16::from_ne_bytes(operands);

                    // Signed addition here because JMP can do negative jumps.
                    // In bytecode, the count always takes into account the operator and its operands,
                    // so here we need to add 3 to it (1 byte for operator, 2 bytes for operands).
                    let target = (bytecode_index as isize + count as isize + 3) as usize;

                    if count > 0 {
                        (0..count).for_each(|_| drop(bytecode.next()));
                    }

                    let (target_block, target_builder) = labels.get(&target)
                        .copied()
                        .expect("Arbitrary backjumps are currently unsupported.");

                    unsafe { LLVMBuildBr(current_builder, target_block) };

                    current_block = target_block;
                    current_builder = target_builder;                    
                }
                JMPFALSENP | JMPFALSEP | /* JMPNULLISHNP | JMPNULLISHP |*/ JMPTRUENP | JMPTRUEP => {
                    let condition = stack.pop().unwrap();
                    let operands = [bytecode.next().unwrap().1, bytecode.next().unwrap().1];
                    let count = i16::from_ne_bytes(operands);

                    let did_jump = paths.next().unwrap();

                    if did_jump {
                        (0..count).for_each(|_| drop(bytecode.next()));
                    }

                    let target = (bytecode_index as isize + count as isize + 3) as usize;
                    let (target_block, target_builder) = *labels.entry(target)
                        .or_insert_with(|| unsafe {
                            let block = LLVMAppendBasicBlock(fun, EMPTY);
                            let builder = LLVMCreateBuilder();
                            LLVMPositionBuilderAtEnd(builder, block);
                            (block, builder)
                        });

                    unsafe {
                        let (then, or) = match (inst, did_jump) {
                            (JMPFALSENP | JMPFALSEP, true) => (exit_block, target_block),
                            (JMPFALSENP | JMPFALSEP, false) => (target_block, exit_block),
                            (JMPTRUENP | JMPTRUEP, true) => (target_block, exit_block),
                            (JMPTRUENP | JMPTRUEP, false) => (exit_block, target_block),
                            _ => unreachable!()
                        };
                        
                        LLVMBuildCondBr(current_builder, condition, then, or);
                        
                        let ip = target + trace.start;
                        exit_ips.push(LLVMConstInt(int64, ip as u64, 0));
                        exit_pred_blocks.push(current_block);
                    }

                    current_block = target_block;
                    current_builder = target_builder;
                }
                LDLOCAL => {
                    let (_, operand) = bytecode.next().unwrap();
                    let idx = operand as u16;

                    let value = *locals.entry(idx as usize).or_insert_with(|| unsafe {
                        // Assert: this is the first time we are referencing this local,
                        // and need to do the necessary things to make space for it.

                        // Alloca stack space for local variable in entry block
                        let space = LLVMBuildAlloca(entry_builder, int64, EMPTY);

                        // Copy from parameter pointer into this allocated stack space
                        let param = LLVMGetParam(fun, 0);

                        // This stores the "offset" of the loaded local variable in the parameter pointer.
                        // This will be used by LLVM's GEP instruction.
                        // We're about to insert this local variable into local_values, so the index will be correct
                        // at the point of building the GEP instruction.
                        let gep_idx = local_values.len() as u64;

                        // If this is the first time we've encountered a reference to this local variable,
                        // we need to store it in the local_values vector.
                        let val = query.get_local(idx);
                        local_values.push((idx, val));

                        let mut indices = [LLVMConstInt(int64, gep_idx, 0)];
                        let gep = LLVMBuildGEP2(
                            entry_builder,
                            int64,
                            param,
                            indices.as_mut_ptr(),
                            1,
                            EMPTY,
                        );

                        let gep_value = LLVMBuildLoad2(entry_builder, int64, gep, EMPTY);

                        // Finally, with this GEP result we can do the actual copy from parameter into the stack space
                        LLVMBuildStore(entry_builder, gep_value, space);

                        space
                    });

                    let load = unsafe {
                        LLVMBuildLoad2(current_builder, int64, value, EMPTY)
                    };

                    stack.push(load);
                }
                STORELOCAL => {
                    let (_, operand) = bytecode.next().unwrap();
                    let idx = operand as u16;

                    let place = *locals.entry(idx as usize).or_insert_with(|| unsafe {
                        // Assert: this is the first time we are referencing this local,
                        // and need to do the necessary things to make space for it.

                        // Alloca stack space for local variable in entry block
                        let space = LLVMBuildAlloca(entry_builder, int64, EMPTY);

                        // Copy from parameter pointer into this allocated stack space
                        let param = LLVMGetParam(fun, 0);

                        // This stores the "offset" of the loaded local variable in the parameter pointer.
                        // This will be used by LLVM's GEP instruction.
                        // We're about to insert this local variable into local_values, so the index will be correct
                        // at the point of building the GEP instruction.
                        let gep_idx = local_values.len() as u64;

                        // If this is the first time we've encountered a reference to this local variable,
                        // we need to store it in the local_values vector.
                        let val = query.get_local(idx);
                        local_values.push((idx, val));

                        let mut indices = [LLVMConstInt(int64, gep_idx, 0)];
                        let gep = LLVMBuildGEP2(
                            entry_builder,
                            int64,
                            param,
                            indices.as_mut_ptr(),
                            1,
                            EMPTY,
                        );

                        // Finally, with this GEP result we can do the actual copy from parameter into the stack space
                        LLVMBuildStore(entry_builder, gep, space);

                        space
                    });
                    let value = stack.pop().unwrap();
                    unsafe { LLVMBuildStore(current_builder, value, place) };
                    stack.push(value);
                }
                LDLOCALW => todo!(),
                CONSTANT => {
                    let (_, operand) = bytecode.next().unwrap();
                    let idx = operand as u16;

                    let num = query.get_constant(idx);
                    let value = unsafe { LLVMConstInt(int64, num as u64, 0) };
                    stack.push(value);
                }
                LT => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();

                    stack.push(unsafe {
                        LLVMBuildICmp(
                            current_builder,
                            LLVMIntPredicate::LLVMIntSLT,
                            lhs,
                            rhs,
                            EMPTY,
                        )
                    });
                }
                REVSTCK => {
                    let amount = bytecode.next().unwrap().1 as usize;
                    let len = stack.len();
                    let target = &mut stack[len - amount..];
                    target.reverse();
                }
                ADD => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();

                    let result = unsafe { LLVMBuildAdd(current_builder, lhs, rhs, EMPTY) };
                    stack.push(result);
                }
                MUL => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();

                    let result = unsafe { LLVMBuildMul(current_builder, lhs, rhs, EMPTY) };
                    stack.push(result);
                }
                POP => {
                    stack.pop().expect("Pop instruction has no target");
                }
                other => {
                    todo!("{other}")
                },
            }
        }

        unsafe {
            // We can only add the final direct jump to trace_start at the very end because throughout IR generation,
            // we keep adding new alloca/load instructions to the entry block.
            LLVMBuildBr(entry_builder, trace_start_block);

            // Exit code here.
            let pred_phi = LLVMBuildPhi(exit_builder, int64, EMPTY);

            debug_assert!(exit_ips.len() == exit_pred_blocks.len());
            LLVMAddIncoming(pred_phi, exit_ips.as_mut_ptr(), exit_pred_blocks.as_mut_ptr(), exit_ips.len() as u32);

            // Copy all of the locals into the function's parameter pointer
            for (index, _) in local_values.iter() {
                let index = (*index) as usize;
                let value = locals[&index];

                let loaded = LLVMBuildLoad2(exit_builder, int64, value, EMPTY);

                let mut indices = [LLVMConstInt(int64, index as u64, 0)];

                let param = LLVMGetParam(fun, 0);
                let gep = LLVMBuildGEP2(exit_builder, int64, param, indices.as_mut_ptr(), 1, EMPTY);
                LLVMBuildStore(exit_builder, loaded, gep);
            }
            
            LLVMBuildRet(exit_builder, pred_phi);

            let pm = LLVMCreatePassManager();
            let pmb = LLVMPassManagerBuilderCreate();
            LLVMPassManagerBuilderSetOptLevel(pmb, 3);
            LLVMPassManagerBuilderPopulateFunctionPassManager(pmb, pm);
            LLVMPassManagerBuilderPopulateModulePassManager(pmb, pm);
            LLVMRunPassManager(pm, self.module);
        }

        let func = unsafe {
            #[cfg(debug_assertions)]
            LLVMVerifyFunction(fun, LLVMVerifierFailureAction::LLVMAbortProcessAction);

            let mut engine = ptr::null_mut();
            let mut error = ptr::null_mut();
            LLVMCreateExecutionEngineForModule(&mut engine, self.module, &mut error);
            assert!(!engine.is_null());

            let addr = LLVMGetFunctionAddress(engine, cstr!("jit").as_ptr());

            mem::transmute::<u64, JitFunction>(addr)
        };
        
        let mut values = local_values.iter().map(|(_, v)| *v).collect::<Vec<_>>();
        println!("=================");
        println!("JIT State");
        let target_ip = unsafe { func(values.as_mut_ptr()) };
        println!("<x = {}>\n", values[0]);

        query.update_ip(target_ip as usize);

        std::process::abort();

        // TODO: synchronize `values` with interpreter here
    }
}

impl Drop for Assembler {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeModule(self.module);
        }
    }
}

impl fmt::Debug for Assembler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = unsafe { CStr::from_ptr(LLVMPrintModuleToString(self.module)) };
        let string = String::from_utf8_lossy(string.to_bytes());
        f.write_str(&string)
    }
}
