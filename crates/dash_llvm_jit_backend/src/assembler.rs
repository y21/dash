use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::ffi::CStr;
use std::ffi::CString;
use std::fmt;
use std::fmt::format;
use std::hash::Hash;
use std::hash::Hasher;
use std::mem;
use std::ptr;

use cstr::cstr;
use dash_middle::compiler::constant::Function;
use dash_middle::compiler::instruction::Instruction;
use indexmap::IndexMap;
use llvm_sys::analysis::LLVMVerifierFailureAction;
use llvm_sys::analysis::LLVMVerifyFunction;
use llvm_sys::core::LLVMAddFunction;
use llvm_sys::core::LLVMAddIncoming;
use llvm_sys::core::LLVMAppendBasicBlock;
use llvm_sys::core::LLVMBuildAdd;
use llvm_sys::core::LLVMBuildAlloca;
use llvm_sys::core::LLVMBuildBitCast;
use llvm_sys::core::LLVMBuildBr;
use llvm_sys::core::LLVMBuildCondBr;
use llvm_sys::core::LLVMBuildFCmp;
use llvm_sys::core::LLVMBuildGEP2;
use llvm_sys::core::LLVMBuildICmp;
use llvm_sys::core::LLVMBuildLoad2;
use llvm_sys::core::LLVMBuildMul;
use llvm_sys::core::LLVMBuildNot;
use llvm_sys::core::LLVMBuildPhi;
use llvm_sys::core::LLVMBuildRet;
use llvm_sys::core::LLVMBuildRetVoid;
use llvm_sys::core::LLVMBuildStore;
use llvm_sys::core::LLVMBuildStructGEP2;
use llvm_sys::core::LLVMBuildSub;
use llvm_sys::core::LLVMBuildXor;
use llvm_sys::core::LLVMConstInt;
use llvm_sys::core::LLVMCreateBuilder;
use llvm_sys::core::LLVMCreatePassManager;
use llvm_sys::core::LLVMDisposeModule;
use llvm_sys::core::LLVMFloatType;
use llvm_sys::core::LLVMFunctionType;
use llvm_sys::core::LLVMGetParam;
use llvm_sys::core::LLVMGetTypeKind;
use llvm_sys::core::LLVMInt1Type;
use llvm_sys::core::LLVMInt32Type;
use llvm_sys::core::LLVMInt64Type;
use llvm_sys::core::LLVMIntType;
use llvm_sys::core::LLVMModuleCreateWithName;
use llvm_sys::core::LLVMPointerType;
use llvm_sys::core::LLVMPositionBuilderAtEnd;
use llvm_sys::core::LLVMPrintModuleToString;
use llvm_sys::core::LLVMRunFunctionPassManager;
use llvm_sys::core::LLVMRunPassManager;
use llvm_sys::core::LLVMStructType;
use llvm_sys::core::LLVMTypeOf;
use llvm_sys::core::LLVMVoidType;
use llvm_sys::execution_engine::LLVMCreateExecutionEngineForModule;
use llvm_sys::execution_engine::LLVMDisposeExecutionEngine;
use llvm_sys::execution_engine::LLVMExecutionEngineRef;
use llvm_sys::execution_engine::LLVMGetFunctionAddress;
use llvm_sys::prelude::*;
use llvm_sys::transforms::pass_manager_builder::LLVMPassManagerBuilderCreate;
use llvm_sys::transforms::pass_manager_builder::LLVMPassManagerBuilderPopulateFunctionPassManager;
use llvm_sys::transforms::pass_manager_builder::LLVMPassManagerBuilderPopulateModulePassManager;
use llvm_sys::transforms::pass_manager_builder::LLVMPassManagerBuilderSetOptLevel;
use llvm_sys::LLVMIntPredicate;
use llvm_sys::LLVMRealPredicate;
use llvm_sys::LLVMTypeKind;

type JitFunction = unsafe extern "C" fn(*mut Value) -> i64;

const EMPTY: *const i8 = cstr!("").as_ptr();

use crate::trace::Trace;
use crate::value::Value;

pub struct JitResult {
    pub function: JitFunction,
    pub values: Vec<Value>,
    pub locals: Vec<u16>,
}

impl JitResult {
    pub fn exec(&mut self) -> i64 {
        unsafe { (self.function)(self.values.as_mut_ptr()) }
    }
}

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct JitCacheKey {
    pub function: *const Function,
    pub ip: usize,
}

impl JitCacheKey {
    pub fn to_c_hash(&self) -> CString {
        let mut hasher = DefaultHasher::new();
        Hash::hash(self, &mut hasher);
        CString::new(format!("jit{:x}", hasher.finish())).unwrap()
    }
}

#[derive(Debug)]
pub struct JitCacheValue {
    pub function: JitFunction,
    /// Captured locals in the same order as `arguments`
    pub locals: Vec<u16>,
}

impl JitCacheValue {
    pub fn exec(&self, args: &mut [Value]) {
        assert_eq!(self.locals.len(), args.len());
        unsafe { (self.function)(args.as_mut_ptr()) };
    }
}

impl From<&Trace> for JitCacheKey {
    fn from(trace: &Trace) -> Self {
        Self {
            function: trace.origin,
            ip: trace.start,
        }
    }
}

pub struct Assembler {
    module: LLVMModuleRef,
    execution_engine: LLVMExecutionEngineRef,
    value_union: LLVMTypeRef,
    cache: HashMap<JitCacheKey, JitCacheValue>,
}

impl Assembler {
    pub fn new() -> Self {
        let module = unsafe { LLVMModuleCreateWithName(cstr!("dash_jit").as_ptr()) };

        let mut engine = ptr::null_mut();
        let mut error = ptr::null_mut();
        unsafe { LLVMCreateExecutionEngineForModule(&mut engine, module, &mut error) };
        assert!(!engine.is_null());

        let largest = unsafe { LLVMIntType(Value::SIZE_OF_LARGEST as u32) };
        let mut types = unsafe {
            [
                LLVMInt32Type(), // discriminant
                largest,
            ]
        };
        let value = unsafe { LLVMStructType(types.as_mut_ptr(), types.len() as u32, 0) };

        Self {
            module,
            value_union: value,
            execution_engine: engine,
            cache: HashMap::new(),
        }
    }

    pub fn get_function(&self, key: JitCacheKey) -> Option<&JitCacheValue> {
        self.cache.get(&key)
    }

    pub fn compile_trace(&mut self, trace: Trace, bytecode: Vec<u8>) -> JitResult {
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

        let cache_key = JitCacheKey::from(&trace);
        let cache_key_hash = cache_key.to_c_hash();

        let (int64, int64ptr, valueptr) = unsafe {
            (
                LLVMInt64Type(),
                LLVMPointerType(LLVMInt64Type(), 0),
                LLVMPointerType(self.value_union, 0),
            )
        };

        let fun = unsafe {
            let mut params = [valueptr];
            let funty = LLVMFunctionType(int64, params.as_mut_ptr(), 1, 0);

            LLVMAddFunction(self.module, cache_key_hash.as_ptr(), funty)
        };

        let mut bytecode = bytecode.into_iter().enumerate();

        let mut paths = trace.conditional_jumps.into_iter();

        // A stack of LLVM values, for building instructions that depend on temporaries
        let mut stack = Vec::new();

        let trace_locals = trace.locals;
        let trace_constants = trace.constants;

        // This maps the LDLOCAL index to a LLVM value, which can be used for building instructions
        // The LLVM value refers to alloca'd stack space in the entry block.
        let mut locals = HashMap::new();

        // This vector stores all referenced locals, in the exact order as they appeared.
        let local_values = trace_locals.values().copied().collect::<Vec<_>>();
        let local_keys = trace_locals.keys().copied().collect::<Vec<_>>();

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
            entry_block = LLVMAppendBasicBlock(fun, cstr!("setup").as_ptr());
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

        while let Some((bytecode_index, inst)) = bytecode.next() {
            let inst = Instruction::from_repr(inst).unwrap();
            match inst {
                Instruction::Jmp => {
                    let operands = [bytecode.next().unwrap().1, bytecode.next().unwrap().1];
                    let count = i16::from_ne_bytes(operands);

                    // Signed addition here because JMP can do negative jumps.
                    // In bytecode, the count always takes into account the operator and its operands,
                    // so here we need to add 3 to it (1 byte for operator, 2 bytes for operands).
                    let target = (bytecode_index as isize + count as isize + 3) as usize;

                    if count >= 0 {
                        // If this is a forwards jump, simply "skip" that many instructions in the iterator
                        (0..count).for_each(|_| drop(bytecode.next()));
                    } else {
                        // Otherwise this is a backjump, means we need to jump to an existing label
                        let (target_block, target_builder) = labels
                            .get(&target)
                            .copied()
                            .expect("Arbitrary backjumps are currently unsupported");

                        unsafe { LLVMBuildBr(current_builder, target_block) };

                        current_block = target_block;
                        current_builder = target_builder;
                    }
                }
                Instruction::JmpFalseNP | Instruction::JmpFalseP | Instruction::JmpTrueNP | Instruction::JmpTrueP => {
                    let condition = stack.pop().unwrap();
                    let operands = [bytecode.next().unwrap().1, bytecode.next().unwrap().1];
                    let count = i16::from_ne_bytes(operands);

                    let did_jump = paths.next().unwrap();

                    if did_jump {
                        (0..count).for_each(|_| drop(bytecode.next()));
                    }

                    let target = (bytecode_index as isize + count as isize + 3) as usize;
                    let (target_block, target_builder) = *labels.entry(target).or_insert_with(|| unsafe {
                        let block = LLVMAppendBasicBlock(fun, EMPTY);
                        let builder = LLVMCreateBuilder();
                        LLVMPositionBuilderAtEnd(builder, block);
                        (block, builder)
                    });

                    unsafe {
                        let (then, or) = match (inst, did_jump) {
                            (Instruction::JmpFalseNP | Instruction::JmpFalseP, true) => (exit_block, target_block),
                            (Instruction::JmpFalseNP | Instruction::JmpFalseP, false) => (target_block, exit_block),
                            (Instruction::JmpTrueNP | Instruction::JmpTrueP, true) => (target_block, exit_block),
                            (Instruction::JmpTrueNP | Instruction::JmpTrueP, false) => (exit_block, target_block),
                            _ => unreachable!(),
                        };

                        LLVMBuildCondBr(current_builder, condition, then, or);

                        let ip = target + trace.start;
                        exit_ips.push(LLVMConstInt(int64, ip as u64, 0));
                        exit_pred_blocks.push(current_block);
                    }

                    current_block = target_block;
                    current_builder = target_builder;
                }
                Instruction::LdLocal => {
                    let (_, operand) = bytecode.next().unwrap();
                    let idx = operand as u16;

                    // This stores the "offset" of the loaded local variable in the parameter pointer.
                    // This will be used by LLVM's GEP instruction.
                    let (gep_idx, _, value) = trace_locals.get_full(&idx).unwrap();

                    let ty = value.type_of();

                    let value = self.find_local_or_insert(&trace_locals, &mut locals, idx, fun, entry_builder);

                    let load = unsafe { LLVMBuildLoad2(current_builder, ty, value, EMPTY) };

                    stack.push(load);
                }
                Instruction::StoreLocal => {
                    let (_, operand) = bytecode.next().unwrap();
                    let idx = operand as u16;

                    // This stores the "offset" of the loaded local variable in the parameter pointer.
                    // This will be used by LLVM's GEP instruction.
                    let (gep_idx, _, value) = trace_locals.get_full(&idx).unwrap();

                    let ty = value.type_of();

                    let place = self.find_local_or_insert(&trace_locals, &mut locals, idx, fun, entry_builder);

                    let value = stack.pop().unwrap();
                    unsafe { LLVMBuildStore(current_builder, value, place) };
                    stack.push(value);
                }
                Instruction::LdLocalW => todo!(),
                Instruction::Constant => {
                    let (_, operand) = bytecode.next().unwrap();
                    let idx = operand as u16;

                    let num = trace_constants[&idx];
                    stack.push(num.to_const_value());
                }
                Instruction::Lt => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();

                    let ty = unsafe {
                        let lhs = LLVMGetTypeKind(LLVMTypeOf(lhs));
                        let rhs = LLVMGetTypeKind(LLVMTypeOf(rhs));
                        assert_eq!(lhs, rhs);
                        lhs
                    };

                    let result = unsafe {
                        match ty {
                            LLVMTypeKind::LLVMFloatTypeKind => {
                                LLVMBuildFCmp(current_builder, LLVMRealPredicate::LLVMRealOLT, lhs, rhs, EMPTY)
                            }
                            LLVMTypeKind::LLVMIntegerTypeKind => {
                                LLVMBuildICmp(current_builder, LLVMIntPredicate::LLVMIntSLT, lhs, rhs, EMPTY)
                            }
                            _ => panic!("Unhandled LLVM type {:?}", ty),
                        }
                    };

                    stack.push(result);
                }
                Instruction::Gt => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();

                    let ty = unsafe {
                        let lhs = LLVMGetTypeKind(LLVMTypeOf(lhs));
                        let rhs = LLVMGetTypeKind(LLVMTypeOf(rhs));
                        assert_eq!(lhs, rhs);
                        lhs
                    };

                    let result = unsafe {
                        match ty {
                            LLVMTypeKind::LLVMFloatTypeKind => {
                                LLVMBuildFCmp(current_builder, LLVMRealPredicate::LLVMRealOGT, lhs, rhs, EMPTY)
                            }
                            LLVMTypeKind::LLVMIntegerTypeKind => {
                                LLVMBuildICmp(current_builder, LLVMIntPredicate::LLVMIntSGT, lhs, rhs, EMPTY)
                            }
                            _ => panic!("Unhandled LLVM type {:?}", ty),
                        }
                    };

                    stack.push(result);
                }
                Instruction::Add => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();

                    // TODO: Does BuildAdd work on floats, or do we need BuildFAdd
                    let result = unsafe { LLVMBuildAdd(current_builder, lhs, rhs, EMPTY) };
                    stack.push(result);
                }
                Instruction::Mul => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();

                    let result = unsafe { LLVMBuildMul(current_builder, lhs, rhs, EMPTY) };
                    stack.push(result);
                }
                Instruction::Sub => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();

                    let result = unsafe { LLVMBuildSub(current_builder, lhs, rhs, EMPTY) };
                    stack.push(result);
                }
                Instruction::BitXor => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();

                    let result = unsafe { LLVMBuildXor(current_builder, lhs, rhs, EMPTY) };
                    stack.push(result);
                }
                Instruction::Ne => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();

                    stack.push(unsafe { LLVMBuildICmp(current_builder, LLVMIntPredicate::LLVMIntNE, lhs, rhs, EMPTY) });
                }
                Instruction::Not | Instruction::BitNot => {
                    let value = stack.pop().unwrap();

                    stack.push(unsafe { LLVMBuildNot(current_builder, value, EMPTY) });
                }
                Instruction::Pop => {
                    stack.pop().expect("Pop instruction has no target");
                }
                other => {
                    todo!("{other:?}")
                }
            }
        }

        unsafe {
            // We can only add the final direct jump to trace_start at the very end because throughout IR generation,
            // we keep adding new alloca/load instructions to the entry block.
            LLVMBuildBr(entry_builder, trace_start_block);

            // JIT exit code
            let pred_phi = LLVMBuildPhi(exit_builder, int64, EMPTY);

            debug_assert!(exit_ips.len() == exit_pred_blocks.len());
            LLVMAddIncoming(
                pred_phi,
                exit_ips.as_mut_ptr(),
                exit_pred_blocks.as_mut_ptr(),
                exit_ips.len() as u32,
            );

            // Copy all of the locals into the function's parameter pointer
            // Interpreter can synchronize the jitted values
            for (llvm_index, (local_index, local_value)) in trace_locals.iter().enumerate() {
                let value = locals[local_index];

                let ty = local_value.type_of();

                // Load alloca'd space
                let loaded = LLVMBuildLoad2(exit_builder, int64, value, EMPTY);

                // Create GEP to get a poiner to the nth value in the param
                let mut indices = [LLVMConstInt(int64, llvm_index as u64, 0)];
                let base_gep = LLVMBuildGEP2(
                    exit_builder,
                    self.value_union,
                    LLVMGetParam(fun, 0),
                    indices.as_mut_ptr(),
                    1,
                    EMPTY,
                );

                // Create GEP to get a pointer to the value
                let struct_gep = LLVMBuildStructGEP2(exit_builder, self.value_union, base_gep, 1, EMPTY);

                LLVMBuildStore(exit_builder, loaded, struct_gep);
            }

            LLVMBuildRet(exit_builder, pred_phi);

            let pm = LLVMCreatePassManager();
            let pmb = LLVMPassManagerBuilderCreate();
            LLVMPassManagerBuilderSetOptLevel(pmb, 3);
            LLVMPassManagerBuilderPopulateFunctionPassManager(pmb, pm);
            LLVMPassManagerBuilderPopulateModulePassManager(pmb, pm);
            LLVMRunPassManager(pm, self.module);
        }

        let function = unsafe {
            #[cfg(debug_assertions)]
            LLVMVerifyFunction(fun, LLVMVerifierFailureAction::LLVMAbortProcessAction);

            let addr = LLVMGetFunctionAddress(self.execution_engine, cache_key_hash.as_ptr());

            mem::transmute::<u64, JitFunction>(addr)
        };

        self.cache.insert(
            cache_key,
            JitCacheValue {
                function,
                locals: local_keys.clone(),
            },
        );

        JitResult {
            function,
            values: local_values,
            locals: local_keys,
        }
    }

    fn find_local_or_insert(
        &self,
        trace_locals: &IndexMap<u16, Value>,
        locals: &mut HashMap<u16, LLVMValueRef>,
        idx: u16,
        fun: LLVMValueRef,
        entry_builder: LLVMBuilderRef,
    ) -> LLVMValueRef {
        // This stores the "offset" of the loaded local variable in the parameter pointer.
        // This will be used by LLVM's GEP instruction.
        let (gep_idx, _, value) = trace_locals.get_full(&idx).unwrap();

        let ty = value.type_of();

        *locals.entry(idx).or_insert_with(|| unsafe {
            // Assert: this is the first time we are referencing this local,
            // and need to do the necessary things to make space for it.

            // Alloca stack space for local variable in entry block
            let space = LLVMBuildAlloca(entry_builder, ty, EMPTY);

            // Copy from parameter pointer into this allocated stack space
            let param = LLVMGetParam(fun, 0);

            let value = {
                let mut indices = [LLVMConstInt(LLVMInt64Type(), gep_idx as u64, 0)];
                let base_gep = LLVMBuildGEP2(entry_builder, self.value_union, param, indices.as_mut_ptr(), 1, EMPTY);

                let struct_gep = LLVMBuildStructGEP2(entry_builder, self.value_union, base_gep, 1, EMPTY);

                LLVMBuildLoad2(entry_builder, ty, struct_gep, EMPTY)
            };

            // Finally, with this GEP result we can do the actual copy from parameter into the stack space
            LLVMBuildStore(entry_builder, value, space);

            space
        })
    }
}

impl Drop for Assembler {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeExecutionEngine(self.execution_engine);
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
