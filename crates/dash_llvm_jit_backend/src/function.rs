use std::cell::Cell;
use std::collections::HashMap;
use std::ffi::CStr;
use std::ffi::CString;
use std::iter::Enumerate;
use std::ops::Deref;
use std::ops::DerefMut;
use std::slice::Iter;

use dash_middle::compiler::instruction::Instruction;
use dash_middle::compiler::instruction::IntrinsicOperation;
use indexmap::Equivalent;
use llvm_sys::analysis::LLVMVerifierFailureAction;
use llvm_sys::analysis::LLVMVerifyFunction;
use llvm_sys::analysis::LLVMVerifyModule;
use llvm_sys::core::LLVMAddFunction;
use llvm_sys::core::LLVMAddIncoming;
use llvm_sys::core::LLVMAppendBasicBlock;
use llvm_sys::core::LLVMAppendBasicBlockInContext;
use llvm_sys::core::LLVMBuildAdd;
use llvm_sys::core::LLVMBuildAlloca;
use llvm_sys::core::LLVMBuildBitCast;
use llvm_sys::core::LLVMBuildBr;
use llvm_sys::core::LLVMBuildCondBr;
use llvm_sys::core::LLVMBuildFAdd;
use llvm_sys::core::LLVMBuildFCmp;
use llvm_sys::core::LLVMBuildFDiv;
use llvm_sys::core::LLVMBuildFMul;
use llvm_sys::core::LLVMBuildFPToSI;
use llvm_sys::core::LLVMBuildFRem;
use llvm_sys::core::LLVMBuildFSub;
use llvm_sys::core::LLVMBuildGEP2;
use llvm_sys::core::LLVMBuildICmp;
use llvm_sys::core::LLVMBuildLoad2;
use llvm_sys::core::LLVMBuildMul;
use llvm_sys::core::LLVMBuildNot;
use llvm_sys::core::LLVMBuildPhi;
use llvm_sys::core::LLVMBuildRet;
use llvm_sys::core::LLVMBuildRetVoid;
use llvm_sys::core::LLVMBuildSDiv;
use llvm_sys::core::LLVMBuildSExt;
use llvm_sys::core::LLVMBuildSIToFP;
use llvm_sys::core::LLVMBuildSRem;
use llvm_sys::core::LLVMBuildStore;
use llvm_sys::core::LLVMBuildStructGEP2;
use llvm_sys::core::LLVMBuildSub;
use llvm_sys::core::LLVMBuildTrunc;
use llvm_sys::core::LLVMBuildUDiv;
use llvm_sys::core::LLVMBuildURem;
use llvm_sys::core::LLVMConstInt;
use llvm_sys::core::LLVMConstReal;
use llvm_sys::core::LLVMContextCreate;
use llvm_sys::core::LLVMContextDispose;
use llvm_sys::core::LLVMCreateBuilder;
use llvm_sys::core::LLVMCreateBuilderInContext;
use llvm_sys::core::LLVMCreatePassManager;
use llvm_sys::core::LLVMDisposeBuilder;
use llvm_sys::core::LLVMDisposeMessage;
use llvm_sys::core::LLVMDisposePassManager;
use llvm_sys::core::LLVMDoubleType;
use llvm_sys::core::LLVMDoubleTypeInContext;
use llvm_sys::core::LLVMFloatType;
use llvm_sys::core::LLVMFunctionType;
use llvm_sys::core::LLVMGetLastBasicBlock;
use llvm_sys::core::LLVMGetParam;
use llvm_sys::core::LLVMGetTypeKind;
use llvm_sys::core::LLVMGetValueName2;
use llvm_sys::core::LLVMInt16TypeInContext;
use llvm_sys::core::LLVMInt1Type;
use llvm_sys::core::LLVMInt1TypeInContext;
use llvm_sys::core::LLVMInt32Type;
use llvm_sys::core::LLVMInt32TypeInContext;
use llvm_sys::core::LLVMInt64Type;
use llvm_sys::core::LLVMInt64TypeInContext;
use llvm_sys::core::LLVMInt8Type;
use llvm_sys::core::LLVMInt8TypeInContext;
use llvm_sys::core::LLVMModuleCreateWithNameInContext;
use llvm_sys::core::LLVMPointerType;
use llvm_sys::core::LLVMPositionBuilderAtEnd;
use llvm_sys::core::LLVMPrintModuleToString;
use llvm_sys::core::LLVMPrintValueToString;
use llvm_sys::core::LLVMRunPassManager;
use llvm_sys::core::LLVMSetInstructionCallConv;
use llvm_sys::core::LLVMSizeOf;
use llvm_sys::core::LLVMStructType;
use llvm_sys::core::LLVMStructTypeInContext;
use llvm_sys::core::LLVMTypeOf;
use llvm_sys::core::LLVMVoidType;
use llvm_sys::core::LLVMVoidTypeInContext;
use llvm_sys::error::LLVMDisposeErrorMessage;
use llvm_sys::execution_engine::LLVMCreateExecutionEngineForModule;
use llvm_sys::execution_engine::LLVMDisposeExecutionEngine;
use llvm_sys::execution_engine::LLVMExecutionEngineRef;
use llvm_sys::execution_engine::LLVMGetExecutionEngineTargetData;
use llvm_sys::execution_engine::LLVMGetFunctionAddress;
use llvm_sys::prelude::LLVMBasicBlockRef;
use llvm_sys::prelude::LLVMBuilderRef;
use llvm_sys::prelude::LLVMContextRef;
use llvm_sys::prelude::LLVMModuleRef;
use llvm_sys::prelude::LLVMPassManagerRef;
use llvm_sys::prelude::LLVMTypeRef;
use llvm_sys::prelude::LLVMValueRef;
use llvm_sys::target::LLVMABISizeOfType;
use llvm_sys::target::LLVMSizeOfTypeInBits;
use llvm_sys::target_machine::LLVMCodeGenOptLevel;
use llvm_sys::transforms::pass_builder::LLVMDisposePassBuilderOptions;
use llvm_sys::transforms::pass_manager_builder::LLVMPassManagerBuilderCreate;
use llvm_sys::transforms::pass_manager_builder::LLVMPassManagerBuilderDispose;
use llvm_sys::transforms::pass_manager_builder::LLVMPassManagerBuilderPopulateFunctionPassManager;
use llvm_sys::transforms::pass_manager_builder::LLVMPassManagerBuilderPopulateModulePassManager;
use llvm_sys::transforms::pass_manager_builder::LLVMPassManagerBuilderRef;
use llvm_sys::transforms::pass_manager_builder::LLVMPassManagerBuilderSetOptLevel;
use llvm_sys::LLVMCallConv;
use llvm_sys::LLVMIntPredicate;
use llvm_sys::LLVMRealPredicate;
use llvm_sys::LLVMTypeKind;
use thiserror::Error;

use crate::backend::JitFunction;
use crate::cstrp;
use crate::passes_legacy::infer::InferResult;
use crate::passes_legacy::infer::Type;
use crate::Backend;
use crate::Trace;

enum Predicate {
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,
}

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("Unimplemented instruction")]
    UnimplementedInstr(Instruction),
}

pub struct Function {
    context: LLVMContextRef,
    module: LLVMModuleRef,
    engine: LLVMExecutionEngineRef,
    pass_manager: LLVMPassManagerRef,
    function: LLVMValueRef,
    value_union: LLVMTypeRef,
    locals: HashMap<u16, (LLVMValueRef, LLVMTypeRef)>,
    labels: HashMap<u16, LLVMBasicBlockRef>,
    setup_block: LLVMBasicBlockRef,
    builder: LLVMBuilderRef,
    exit_block: LLVMBasicBlockRef,
    exit_guards: Vec<(u64, LLVMBasicBlockRef)>,
}

impl Function {
    fn value_type(ctx: LLVMContextRef, engine: LLVMExecutionEngineRef) -> LLVMTypeRef {
        // Biggest type is (u8, usize, usize): trait object
        // TODO: don't hardcode this
        // TODO2: don't hardcode usize as i64
        unsafe {
            let mut elements = [
                LLVMInt8TypeInContext(ctx),
                LLVMInt64TypeInContext(ctx),
                LLVMInt64TypeInContext(ctx),
            ];
            let ty = LLVMStructTypeInContext(ctx, elements.as_mut_ptr(), elements.len() as u32, 0);

            debug_assert!({
                // TODO: do we need to free this TargetData?
                let size = LLVMSizeOfTypeInBits(LLVMGetExecutionEngineTargetData(engine), ty);
                size == 24 * 8
            });

            ty
        }
    }
    fn create_function_type(ctx: LLVMContextRef, engine: LLVMExecutionEngineRef) -> LLVMTypeRef {
        unsafe {
            let mut args = [
                LLVMPointerType(Self::value_type(ctx, engine), 0),
                LLVMInt64TypeInContext(ctx),
                LLVMPointerType(LLVMInt64TypeInContext(ctx), 0),
            ];
            let ret = LLVMVoidTypeInContext(ctx);
            LLVMFunctionType(ret, args.as_mut_ptr(), args.len() as u32, 0)
        }
    }

    pub fn new() -> Self {
        let context = unsafe { LLVMContextCreate() };
        let module = unsafe { LLVMModuleCreateWithNameInContext(cstrp!("jit"), context) };
        let mut engine = std::ptr::null_mut();
        let mut error = std::ptr::null_mut();
        assert!(unsafe { LLVMCreateExecutionEngineForModule(&mut engine, module, &mut error) } == 0);
        assert!(error.is_null());

        let value_union = Self::value_type(context, engine);
        let ty = Self::create_function_type(context, engine);
        let function = unsafe { LLVMAddFunction(module, cstrp!("jit"), ty) };
        unsafe { LLVMSetInstructionCallConv(function, LLVMCallConv::LLVMCCallConv as u32) }

        let pass_manager = unsafe {
            let pm = LLVMCreatePassManager();
            let pmb = LLVMPassManagerBuilderCreate();
            LLVMPassManagerBuilderSetOptLevel(pmb, LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive as u32);
            LLVMPassManagerBuilderPopulateFunctionPassManager(pmb, pm);
            LLVMPassManagerBuilderPopulateModulePassManager(pmb, pm);

            LLVMPassManagerBuilderDispose(pmb);
            pm
        };

        let builder = unsafe { LLVMCreateBuilderInContext(context) };
        let setup_block = unsafe { LLVMAppendBasicBlockInContext(context, function, cstrp!("setup")) };
        let exit_block = unsafe { LLVMAppendBasicBlockInContext(context, function, cstrp!("exit")) };

        Self {
            context,
            engine,
            module,
            pass_manager,
            function,
            locals: HashMap::new(),
            value_union,
            labels: HashMap::new(),
            setup_block,
            exit_block,
            builder,
            exit_guards: Vec::new(),
        }
    }

    fn append_block(&self) -> LLVMBasicBlockRef {
        unsafe { LLVMAppendBasicBlockInContext(self.context, self.function, cstrp!("block")) }
    }

    fn append_and_enter_block(&self) -> LLVMBasicBlockRef {
        unsafe {
            let block = self.append_block();
            LLVMPositionBuilderAtEnd(self.builder, block);
            block
        }
    }

    fn alloca_local(&self, ty: &Type) -> LLVMValueRef {
        unsafe { LLVMBuildAlloca(self.builder, ty.to_llvm_type(self.context), cstrp!("local")) }
    }

    fn get_param(&self, param: u32) -> LLVMValueRef {
        unsafe { LLVMGetParam(self.function, param) }
    }

    fn const_i1(&self, value: bool) -> LLVMValueRef {
        unsafe { LLVMConstInt(self.i1_ty(), value as u64, 0) }
    }

    fn const_i32(&self, value: i32) -> LLVMValueRef {
        unsafe { LLVMConstInt(self.i32_ty(), value as u64, 0) }
    }

    fn i64_ty(&self) -> LLVMTypeRef {
        unsafe { LLVMInt64TypeInContext(self.context) }
    }

    fn i32_ty(&self) -> LLVMTypeRef {
        unsafe { LLVMInt32TypeInContext(self.context) }
    }

    fn i1_ty(&self) -> LLVMTypeRef {
        unsafe { LLVMInt1TypeInContext(self.context) }
    }

    fn f64_ty(&self) -> LLVMTypeRef {
        unsafe { LLVMDoubleTypeInContext(self.context) }
    }

    fn const_i64(&self, value: i64) -> LLVMValueRef {
        unsafe { LLVMConstInt(self.i64_ty(), value as u64, 0) }
    }

    fn const_f64(&self, value: f64) -> LLVMValueRef {
        unsafe { LLVMConstReal(self.f64_ty(), value) }
    }

    fn build_add(&self, a: LLVMValueRef, b: LLVMValueRef) -> LLVMValueRef {
        let ty = self.type_of_value(a);
        let builder = self.builder;
        unsafe {
            match ty {
                LLVMTypeKind::LLVMIntegerTypeKind => LLVMBuildAdd(self.builder, a, b, cstrp!("iadd")),
                LLVMTypeKind::LLVMDoubleTypeKind => LLVMBuildFAdd(self.builder, a, b, cstrp!("fadd")),
                _ => panic!("Unsupported type for addition"),
            }
        }
    }

    fn build_sub(&self, a: LLVMValueRef, b: LLVMValueRef) -> LLVMValueRef {
        let ty = self.type_of_value(a);
        let builder = self.builder;
        unsafe {
            match ty {
                LLVMTypeKind::LLVMIntegerTypeKind => LLVMBuildSub(self.builder, a, b, cstrp!("isub")),
                LLVMTypeKind::LLVMDoubleTypeKind => LLVMBuildFSub(self.builder, a, b, cstrp!("fsub")),
                _ => panic!("Unsupported type for subtraction"),
            }
        }
    }

    fn build_mul(&self, a: LLVMValueRef, b: LLVMValueRef) -> LLVMValueRef {
        let ty = self.type_of_value(a);
        let builder = self.builder;
        unsafe {
            match ty {
                LLVMTypeKind::LLVMIntegerTypeKind => LLVMBuildMul(self.builder, a, b, cstrp!("imul")),
                LLVMTypeKind::LLVMDoubleTypeKind => LLVMBuildFMul(self.builder, a, b, cstrp!("fmul")),
                _ => panic!("Unsupported type for multiplication"),
            }
        }
    }

    fn build_div(&self, a: LLVMValueRef, b: LLVMValueRef) -> LLVMValueRef {
        let ty = self.type_of_value(a);
        let builder = self.builder;
        unsafe {
            match ty {
                LLVMTypeKind::LLVMIntegerTypeKind => LLVMBuildSDiv(self.builder, a, b, cstrp!("idiv")),
                LLVMTypeKind::LLVMDoubleTypeKind => LLVMBuildFMul(self.builder, a, b, cstrp!("fdiv")),
                _ => panic!("Unsupported type for division"),
            }
        }
    }

    fn build_rem(&self, a: LLVMValueRef, b: LLVMValueRef) -> LLVMValueRef {
        let ty = self.type_of_value(a);
        let builder = self.builder;
        unsafe {
            match ty {
                LLVMTypeKind::LLVMIntegerTypeKind => LLVMBuildSRem(self.builder, a, b, cstrp!("irem")),
                LLVMTypeKind::LLVMDoubleTypeKind => LLVMBuildFRem(self.builder, a, b, cstrp!("frem")),
                _ => panic!("Unsupported type for remainder"),
            }
        }
    }

    fn build_load(&self, ty: LLVMTypeRef, value: LLVMValueRef) -> LLVMValueRef {
        unsafe { LLVMBuildLoad2(self.builder, ty, value, cstrp!("load")) }
    }

    fn build_store(&self, value: LLVMValueRef, ptr: LLVMValueRef) -> LLVMValueRef {
        unsafe { LLVMBuildStore(self.builder, value, ptr) }
    }

    fn build_br(&self, to: LLVMBasicBlockRef) -> LLVMValueRef {
        unsafe { LLVMBuildBr(self.builder, to) }
    }

    fn build_condbr(
        &self,
        condition: LLVMValueRef,
        dest_true: LLVMBasicBlockRef,
        dest_false: LLVMBasicBlockRef,
    ) -> LLVMValueRef {
        unsafe { LLVMBuildCondBr(self.builder, condition, dest_true, dest_false) }
    }

    fn build_cmp(&self, a: LLVMValueRef, b: LLVMValueRef, pred: Predicate) -> LLVMValueRef {
        let ty = self.type_of_value(a);
        unsafe {
            match (ty, pred) {
                (LLVMTypeKind::LLVMIntegerTypeKind, Predicate::Le) => {
                    LLVMBuildICmp(self.builder, LLVMIntPredicate::LLVMIntSLE, a, b, cstrp!("ile"))
                }
                (LLVMTypeKind::LLVMIntegerTypeKind, Predicate::Lt) => {
                    LLVMBuildICmp(self.builder, LLVMIntPredicate::LLVMIntSLT, a, b, cstrp!("ilt"))
                }
                (LLVMTypeKind::LLVMIntegerTypeKind, Predicate::Ge) => {
                    LLVMBuildICmp(self.builder, LLVMIntPredicate::LLVMIntSGE, a, b, cstrp!("ige"))
                }
                (LLVMTypeKind::LLVMIntegerTypeKind, Predicate::Gt) => {
                    LLVMBuildICmp(self.builder, LLVMIntPredicate::LLVMIntSGT, a, b, cstrp!("igt"))
                }
                (LLVMTypeKind::LLVMIntegerTypeKind, Predicate::Eq) => {
                    LLVMBuildICmp(self.builder, LLVMIntPredicate::LLVMIntEQ, a, b, cstrp!("ieq"))
                }
                (LLVMTypeKind::LLVMIntegerTypeKind, Predicate::Ne) => {
                    LLVMBuildICmp(self.builder, LLVMIntPredicate::LLVMIntNE, a, b, cstrp!("ine"))
                }
                (LLVMTypeKind::LLVMDoubleTypeKind, Predicate::Le) => {
                    LLVMBuildFCmp(self.builder, LLVMRealPredicate::LLVMRealULE, a, b, cstrp!("fle"))
                }
                (LLVMTypeKind::LLVMDoubleTypeKind, Predicate::Lt) => {
                    LLVMBuildFCmp(self.builder, LLVMRealPredicate::LLVMRealULT, a, b, cstrp!("flt"))
                }
                (LLVMTypeKind::LLVMDoubleTypeKind, Predicate::Ge) => {
                    LLVMBuildFCmp(self.builder, LLVMRealPredicate::LLVMRealUGE, a, b, cstrp!("fge"))
                }
                (LLVMTypeKind::LLVMDoubleTypeKind, Predicate::Gt) => {
                    LLVMBuildFCmp(self.builder, LLVMRealPredicate::LLVMRealUGT, a, b, cstrp!("fgt"))
                }
                (LLVMTypeKind::LLVMDoubleTypeKind, Predicate::Eq) => {
                    LLVMBuildFCmp(self.builder, LLVMRealPredicate::LLVMRealUEQ, a, b, cstrp!("feq"))
                }
                (LLVMTypeKind::LLVMDoubleTypeKind, Predicate::Ne) => {
                    LLVMBuildFCmp(self.builder, LLVMRealPredicate::LLVMRealUNE, a, b, cstrp!("fne"))
                }
                _ => panic!("Unsupported type for comparison"),
            }
        }
    }

    fn build_lt(&self, a: LLVMValueRef, b: LLVMValueRef) -> LLVMValueRef {
        self.build_cmp(a, b, Predicate::Lt)
    }

    fn build_gt(&self, a: LLVMValueRef, b: LLVMValueRef) -> LLVMValueRef {
        self.build_cmp(a, b, Predicate::Gt)
    }

    fn build_le(&self, a: LLVMValueRef, b: LLVMValueRef) -> LLVMValueRef {
        self.build_cmp(a, b, Predicate::Le)
    }

    fn build_ge(&self, a: LLVMValueRef, b: LLVMValueRef) -> LLVMValueRef {
        self.build_cmp(a, b, Predicate::Ge)
    }

    fn build_eq(&self, a: LLVMValueRef, b: LLVMValueRef) -> LLVMValueRef {
        self.build_cmp(a, b, Predicate::Eq)
    }

    fn build_ne(&self, a: LLVMValueRef, b: LLVMValueRef) -> LLVMValueRef {
        self.build_cmp(a, b, Predicate::Ne)
    }

    fn build_not(&self, a: LLVMValueRef) -> LLVMValueRef {
        let ty = self.type_of_value(a);
        unsafe {
            match ty {
                LLVMTypeKind::LLVMIntegerTypeKind => LLVMBuildNot(self.builder, a, cstrp!("not")),
                _ => panic!("Unsupported type for not"),
            }
        }
    }

    fn build_local_load(&self, id: u16) -> LLVMValueRef {
        let (local, ty) = self.locals[&id];
        unsafe { LLVMBuildLoad2(self.builder, ty, local, cstrp!("local_load")) }
    }

    fn build_local_store(&self, id: u16, value: LLVMValueRef) -> LLVMValueRef {
        let (local, _) = self.locals[&id];
        unsafe { LLVMBuildStore(self.builder, value, local) }
    }

    fn build_retvoid(&self) -> LLVMValueRef {
        unsafe { LLVMBuildRetVoid(self.builder) }
    }

    fn build_i64_to_f64_transmute(&self, value: LLVMValueRef) -> LLVMValueRef {
        unsafe { LLVMBuildBitCast(self.builder, value, self.f64_ty(), cstrp!("i64_to_f64")) }
    }

    fn build_f64_to_i64_transmute(&self, value: LLVMValueRef) -> LLVMValueRef {
        unsafe { LLVMBuildBitCast(self.builder, value, self.i64_ty(), cstrp!("f64_to_i64")) }
    }

    fn build_cast(&self, value: LLVMValueRef, from: &Type, to: &Type) -> LLVMValueRef {
        unsafe {
            match (from, to) {
                (Type::I64, Type::Boolean) => LLVMBuildTrunc(self.builder, value, self.i1_ty(), cstrp!("trunc")),
                (Type::F64, Type::Boolean) => {
                    let to_int = self.build_cast(value, from, &Type::I64);
                    self.build_cast(to_int, &Type::I64, &Type::Boolean)
                }
                (Type::Boolean, Type::I64) => LLVMBuildSExt(self.builder, value, self.i64_ty(), cstrp!("sext")),
                (Type::Boolean, Type::F64) => {
                    let to_int = self.build_cast(value, from, &Type::I64);
                    self.build_cast(to_int, &Type::I64, &Type::F64)
                }
                (Type::I64, Type::F64) => LLVMBuildSIToFP(self.builder, value, self.f64_ty(), cstrp!("sitofp")),
                (Type::F64, Type::I64) => LLVMBuildFPToSI(self.builder, value, self.i64_ty(), cstrp!("fptosi")),
                _ => panic!("Invalid cast {:?} -> {:?}", from, to),
            }
        }
    }

    fn create_jumpable_block(&mut self, ip: u16) -> LLVMBasicBlockRef {
        let block = self.append_block();
        self.labels.insert(ip, block);
        block
    }

    fn position_builder_at(&self, to: LLVMBasicBlockRef) {
        unsafe { LLVMPositionBuilderAtEnd(self.builder, to) }
    }

    fn setup_block(&self) -> LLVMBasicBlockRef {
        self.setup_block
    }

    fn builder(&self) -> LLVMBuilderRef {
        self.builder
    }

    fn exit_block(&self) -> LLVMBasicBlockRef {
        self.exit_block
    }

    /// Compiles the setup block by initializing locals (copying them out of the stack). Must be the first function to be called
    pub fn compile_setup(&mut self, locals: &HashMap<u16, Type>) {
        let setup_block = self.setup_block();

        self.position_builder_at(setup_block);

        for (&local_index, ty) in locals {
            let space = self.alloca_local(ty);

            let stack_ptr = self.get_param(0);
            let stack_offset = self.get_param(1);
            let index = self.const_i64(local_index as i64);
            let stack_offset = self.build_add(stack_offset, index);

            let mut indices = [stack_offset, self.const_i32(1)];
            let value_ptr = self.build_gep(self.value_union, stack_ptr, &mut indices);

            let value = self.build_load(self.i64_ty(), value_ptr);
            let value = match ty {
                Type::Boolean => self.build_cast(value, &Type::I64, ty),
                Type::I64 => {
                    // even though value is of type i64, it only contains the raw bits
                    // so we need to do a i64 -> f64 -> i64 roundtrip
                    let value = self.build_i64_to_f64_transmute(value);
                    self.build_cast(value, &Type::F64, &Type::I64)
                }
                Type::F64 => self.build_i64_to_f64_transmute(value),
            };
            self.build_store(value, space);

            self.locals.insert(local_index, (space, ty.to_llvm_type(self.context)));
        }
    }

    /// Compiles the exit block
    pub fn compile_exit_block(&mut self, locals: &HashMap<u16, Type>) {
        // Compile exit block
        // Write all the values back to the stack
        let exit_block = self.exit_block();
        self.position_builder_at(exit_block);

        let mut ret_phi = self.build_phi(self.i64_ty());
        for (ip, block) in &self.exit_guards {
            self.build_phi_node(ret_phi, self.const_i64(*ip as i64), *block);
        }
        let out_ip = self.get_param(2);
        self.build_store(ret_phi, out_ip);

        for (&local_index, ty) in locals {
            let (space, _) = self.locals[&local_index];
            let value = self.build_local_load(local_index);
            let value = match ty {
                Type::Boolean => self.build_cast(value, ty, &Type::I64),
                Type::I64 => {
                    let value = self.build_cast(value, ty, &Type::F64);
                    self.build_f64_to_i64_transmute(value)
                }
                Type::F64 => self.build_f64_to_i64_transmute(value),
            };

            let stack_ptr = self.get_param(0);
            let stack_offset = self.get_param(1);
            let index = self.const_i64(local_index as i64);
            let stack_offset = self.build_add(stack_offset, index);

            let mut indices = [stack_offset, self.const_i32(1)];
            let value_ptr = self.build_gep(self.value_union, stack_ptr, &mut indices);

            self.build_store(value, value_ptr);
        }

        self.build_retvoid();
    }

    fn build_gep(&self, ty: LLVMTypeRef, ptr: LLVMValueRef, indices: &mut [LLVMValueRef]) -> LLVMValueRef {
        unsafe {
            LLVMBuildGEP2(
                self.builder,
                ty,
                ptr,
                indices.as_mut_ptr(),
                indices.len() as u32,
                cstrp!("gep"),
            )
        }
    }

    fn type_of_value(&self, value: LLVMValueRef) -> LLVMTypeKind {
        unsafe { LLVMGetTypeKind(LLVMTypeOf(value)) }
    }

    fn build_phi(&self, ty: LLVMTypeRef) -> LLVMValueRef {
        unsafe { LLVMBuildPhi(self.builder, ty, cstrp!("phi")) }
    }

    fn build_phi_node(&self, phi: LLVMValueRef, value: LLVMValueRef, block: LLVMBasicBlockRef) {
        let mut values = [value];
        let mut blocks = [block];
        unsafe { LLVMAddIncoming(phi, values.as_mut_ptr(), blocks.as_mut_ptr(), 1) }
    }

    pub fn compile_trace<Q: CompileQuery>(
        &mut self,
        bytecode: &[u8],
        q: &Q,
        infer: &InferResult,
        trace: &Trace,
    ) -> Result<(), CompileError> {
        let mut cx = CompilationContext::new(self, bytecode);

        let mut jumps = 0;

        while let Some((index, instr)) = cx.next_instruction() {
            let is_label = infer.labels[index];
            if is_label {
                let block = cx.create_jumpable_block(index as u16);
                cx.build_br(block);
                cx.position_builder_at(block);
            }

            match instr {
                Instruction::Add => cx.with2(|cx, a, b| cx.build_add(a, b)),
                Instruction::Sub => cx.with2(|cx, a, b| cx.build_sub(a, b)),
                Instruction::Mul => cx.with2(|cx, a, b| cx.build_mul(a, b)),
                Instruction::Div => cx.with2(|cx, a, b| cx.build_div(a, b)),
                Instruction::Rem => cx.with2(|cx, a, b| cx.build_rem(a, b)),
                Instruction::LdLocal => {
                    let id = cx.next_byte();
                    let value = cx.build_local_load(id.into());
                    cx.push(value);
                }
                Instruction::StoreLocal => {
                    let id = cx.next_byte();
                    let value = cx.pop();
                    cx.build_local_store(id.into(), value);
                    let value = cx.build_local_load(id.into());
                    cx.push(value);
                }
                Instruction::Constant => {
                    let cid = cx.next_byte();
                    let constant = q.get_constant(cid.into());
                    let value = match constant {
                        JITConstant::F64(value) => cx.const_f64(value),
                        JITConstant::I64(value) => cx.const_i64(value),
                        JITConstant::Boolean(value) => cx.const_i1(value),
                    };
                    cx.push(value);
                }
                Instruction::Pop => drop(cx.pop()),
                Instruction::Lt => cx.with2(|cx, a, b| cx.build_lt(a, b)),
                Instruction::Gt => cx.with2(|cx, a, b| cx.build_gt(a, b)),
                Instruction::Jmp => {
                    let count = cx.next_wide() as i16;
                    let target_ip = (index as isize + count as isize) + 3;
                    let target_block = cx.labels[&(target_ip as u16)];
                    cx.build_br(target_block);
                }
                Instruction::JmpFalseP => {
                    let count = cx.next_wide() as i16;
                    let next_ip = index as isize + 3;
                    let jump_target_ip = (index as isize + count as isize) + 3;
                    let value = cx.pop();
                    let did_take = trace.conditional_jumps[jumps];
                    if did_take {
                        for i in 0..count {
                            cx.next_byte();
                        }
                    }
                    jumps += 1;
                    cx.emit_guard(
                        value,
                        !did_take,
                        next_ip.try_into().unwrap(),
                        jump_target_ip.try_into().unwrap(),
                    );
                }
                Instruction::Ne | Instruction::StrictNe => cx.with2(|cx, a, b| cx.build_ne(a, b)),
                Instruction::Eq | Instruction::StrictEq => cx.with2(|cx, a, b| cx.build_eq(a, b)),
                Instruction::Not => {
                    let value = cx.pop();
                    let value = cx.build_not(value);
                    cx.push(value);
                }
                Instruction::IntrinsicOp => {
                    let op = IntrinsicOperation::from_repr(cx.next_byte()).unwrap();

                    match op {
                        IntrinsicOperation::AddNumLR => {
                            cx.with2(|cx, a, b| cx.build_add(a, b));
                        }
                        IntrinsicOperation::SubNumLR => {
                            cx.with2(|cx, a, b| cx.build_sub(a, b));
                        }
                        IntrinsicOperation::MulNumLR => {
                            cx.with2(|cx, a, b| cx.build_mul(a, b));
                        }
                        IntrinsicOperation::DivNumLR => {
                            cx.with2(|cx, a, b| cx.build_div(a, b));
                        }

                        IntrinsicOperation::PostfixIncLocalNum => {
                            let id = cx.next_byte();
                            let old_value = cx.build_local_load(id.into());
                            let rhs = match cx.type_of_value(old_value) {
                                LLVMTypeKind::LLVMIntegerTypeKind => cx.const_i64(1),
                                LLVMTypeKind::LLVMDoubleTypeKind => cx.const_f64(1.0),
                                _ => unreachable!(),
                            };
                            let value = cx.build_add(old_value, rhs);
                            cx.build_local_store(id.into(), value);
                            cx.push(old_value);
                        }
                        IntrinsicOperation::PostfixDecLocalNum => {
                            let id = cx.next_byte();
                            let old_value = cx.build_local_load(id.into());
                            let rhs = match cx.type_of_value(old_value) {
                                LLVMTypeKind::LLVMIntegerTypeKind => cx.const_i64(1),
                                LLVMTypeKind::LLVMDoubleTypeKind => cx.const_f64(1.0),
                                _ => unreachable!(),
                            };
                            let value = cx.build_sub(old_value, rhs);
                            cx.build_local_store(id.into(), value);
                            cx.push(old_value);
                        }

                        IntrinsicOperation::LtNumLConstR => {
                            let value = cx.pop();
                            let num = cx.next_byte() as f64;
                            let rhs = match cx.type_of_value(value) {
                                LLVMTypeKind::LLVMIntegerTypeKind => cx.const_i64(num as i64),
                                LLVMTypeKind::LLVMDoubleTypeKind => cx.const_f64(num),
                                _ => unreachable!(),
                            };
                            let res = cx.build_lt(value, rhs);
                            cx.push(res);
                        }

                        IntrinsicOperation::GtNumLConstR32
                        | IntrinsicOperation::GeNumLConstR32
                        | IntrinsicOperation::LtNumLConstR32
                        | IntrinsicOperation::LeNumLConstR32 => {
                            let value = cx.pop();
                            let num = cx.next_u32() as f64;
                            let rhs = match cx.type_of_value(value) {
                                LLVMTypeKind::LLVMIntegerTypeKind => cx.const_i64(num as i64),
                                LLVMTypeKind::LLVMDoubleTypeKind => cx.const_f64(num),
                                _ => unreachable!(),
                            };
                            let res = match op {
                                IntrinsicOperation::GtNumLConstR32 => cx.build_gt(value, rhs),
                                IntrinsicOperation::GeNumLConstR32 => cx.build_ge(value, rhs),
                                IntrinsicOperation::LtNumLConstR32 => cx.build_lt(value, rhs),
                                IntrinsicOperation::LeNumLConstR32 => cx.build_le(value, rhs),
                                _ => unreachable!(),
                            };
                            cx.push(res);
                        }
                        _ => return Err(CompileError::UnimplementedInstr(instr)),
                    }
                }
                _ => return Err(CompileError::UnimplementedInstr(instr)),
            }
        }

        Ok(())
    }

    pub fn verify(&self) {
        let mut error = std::ptr::null_mut();
        unsafe {
            LLVMVerifyModule(
                self.module,
                LLVMVerifierFailureAction::LLVMAbortProcessAction,
                &mut error,
            );
            LLVMDisposeMessage(error);
        };
    }

    pub fn run_pass_manager(&self) {
        unsafe {
            LLVMRunPassManager(self.pass_manager, self.module);
        }
    }

    pub fn print_module(&self) {
        let string = unsafe { CStr::from_ptr(LLVMPrintModuleToString(self.module)) };
        let rust_string = String::from_utf8_lossy(string.to_bytes());
        println!("{}", rust_string);

        unsafe { LLVMDisposeMessage(string.as_ptr() as *mut i8) }
    }

    pub fn compile(&self) -> JitFunction {
        unsafe {
            let addr = LLVMGetFunctionAddress(self.engine, self.function_name().as_ptr());
            assert!(addr != 0);
            let fun = std::mem::transmute::<u64, JitFunction>(addr);
            fun
        }
    }

    pub fn function_name(&self) -> &CStr {
        unsafe {
            // TODO: is this correct? what is length even used for?
            let mut length = 0;
            let name = LLVMGetValueName2(self.function, &mut length);
            let name = CStr::from_ptr(name);
            name
        }
    }

    pub fn function_value(&self) -> LLVMValueRef {
        self.function
    }
}

impl Drop for Function {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.builder);
            LLVMDisposePassManager(self.pass_manager);
            LLVMDisposeExecutionEngine(self.engine);
            LLVMContextDispose(self.context);
        }
    }
}

#[derive(Debug)]
pub enum JITConstant {
    F64(f64),
    I64(i64),
    Boolean(bool),
}

pub trait CompileQuery {
    fn get_constant(&self, id: u16) -> JITConstant;
}

struct CompilationContext<'fun, 'bytecode> {
    iter: Enumerate<Iter<'bytecode, u8>>,
    function: &'fun mut Function,
    value_stack: Vec<LLVMValueRef>,
}

impl<'fun, 'bytecode> CompilationContext<'fun, 'bytecode> {
    pub fn new(function: &'fun mut Function, bytecode: &'bytecode [u8]) -> Self {
        Self {
            iter: bytecode.iter().enumerate(),
            function,
            value_stack: Vec::new(),
        }
    }

    pub fn with2<F>(&mut self, fun: F)
    where
        F: Fn(&mut CompilationContext<'fun, 'bytecode>, LLVMValueRef, LLVMValueRef) -> LLVMValueRef,
    {
        let (a, b) = self.pop2();
        let result = fun(self, b, a);
        self.push(result);
    }

    pub fn push(&mut self, value: LLVMValueRef) {
        self.value_stack.push(value);
    }

    pub fn pop(&mut self) -> LLVMValueRef {
        self.value_stack.pop().unwrap()
    }

    pub fn pop2(&mut self) -> (LLVMValueRef, LLVMValueRef) {
        let b = self.pop();
        let a = self.pop();
        (b, a)
    }

    pub fn next_instruction(&mut self) -> Option<(usize, Instruction)> {
        let (index, &instr) = self.iter.next()?;
        let instr = Instruction::from_repr(instr).unwrap();
        Some((index, instr))
    }

    pub fn next_byte(&mut self) -> u8 {
        let (_, &byte) = self.iter.next().unwrap();
        byte
    }

    pub fn next_wide(&mut self) -> u16 {
        let high = self.next_byte();
        let low = self.next_byte();
        u16::from_ne_bytes([high, low])
    }

    pub fn next_u32(&mut self) -> u32 {
        let a = self.next_byte();
        let b = self.next_byte();
        let c = self.next_byte();
        let d = self.next_byte();
        u32::from_ne_bytes([a, b, c, d])
    }

    pub fn emit_guard(&mut self, condition: LLVMValueRef, expected: bool, next_ip: u64, jump_target_ip: u64) {
        let block = unsafe { LLVMGetLastBasicBlock(self.function.function) };

        let next_block = self.append_block();
        let (dest_true, dest_false, target_ip) = match expected {
            true => (next_block, self.exit_block, jump_target_ip),
            false => (self.exit_block, next_block, next_ip),
        };
        self.exit_guards.push((target_ip, block));
        self.build_condbr(condition, dest_true, dest_false);
        self.position_builder_at(next_block);
    }
}

impl<'fun, 'bytecode> Deref for CompilationContext<'fun, 'bytecode> {
    type Target = Function;

    fn deref(&self) -> &Self::Target {
        self.function
    }
}

impl<'fun, 'bytecode> DerefMut for CompilationContext<'fun, 'bytecode> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.function
    }
}
