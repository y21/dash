use std::cell::Cell;
use std::collections::HashMap;
use std::ffi::CString;
use std::iter::Enumerate;
use std::ops::Deref;
use std::ops::DerefMut;
use std::slice::Iter;

use dash_middle::compiler::instruction::Instruction;
use indexmap::Equivalent;
use llvm_sys::analysis::LLVMVerifierFailureAction;
use llvm_sys::analysis::LLVMVerifyFunction;
use llvm_sys::core::LLVMAddFunction;
use llvm_sys::core::LLVMAppendBasicBlock;
use llvm_sys::core::LLVMBuildAdd;
use llvm_sys::core::LLVMBuildAlloca;
use llvm_sys::core::LLVMBuildBr;
use llvm_sys::core::LLVMBuildFCmp;
use llvm_sys::core::LLVMBuildFDiv;
use llvm_sys::core::LLVMBuildFRem;
use llvm_sys::core::LLVMBuildGEP2;
use llvm_sys::core::LLVMBuildLoad2;
use llvm_sys::core::LLVMBuildMul;
use llvm_sys::core::LLVMBuildRet;
use llvm_sys::core::LLVMBuildRetVoid;
use llvm_sys::core::LLVMBuildStore;
use llvm_sys::core::LLVMBuildSub;
use llvm_sys::core::LLVMConstInt;
use llvm_sys::core::LLVMCreateBuilder;
use llvm_sys::core::LLVMDoubleType;
use llvm_sys::core::LLVMFloatType;
use llvm_sys::core::LLVMFunctionType;
use llvm_sys::core::LLVMGetParam;
use llvm_sys::core::LLVMInt16TypeInContext;
use llvm_sys::core::LLVMInt1Type;
use llvm_sys::core::LLVMInt32Type;
use llvm_sys::core::LLVMInt64Type;
use llvm_sys::core::LLVMInt8Type;
use llvm_sys::core::LLVMPointerType;
use llvm_sys::core::LLVMPositionBuilderAtEnd;
use llvm_sys::core::LLVMSizeOf;
use llvm_sys::core::LLVMStructType;
use llvm_sys::core::LLVMVoidType;
use llvm_sys::execution_engine::LLVMExecutionEngineRef;
use llvm_sys::execution_engine::LLVMGetExecutionEngineTargetData;
use llvm_sys::prelude::LLVMBasicBlockRef;
use llvm_sys::prelude::LLVMBuilderRef;
use llvm_sys::prelude::LLVMModuleRef;
use llvm_sys::prelude::LLVMTypeRef;
use llvm_sys::prelude::LLVMValueRef;
use llvm_sys::target::LLVMABISizeOfType;
use llvm_sys::target::LLVMSizeOfTypeInBits;
use llvm_sys::LLVMIntPredicate;
use llvm_sys::LLVMRealPredicate;

use crate::cstr;
use crate::passes::infer::InferResult;
use crate::passes::infer::Type;
use crate::Backend;
use crate::Trace;

pub struct Function {
    function: LLVMValueRef,
    value_union: LLVMTypeRef,
    locals: HashMap<u16, (LLVMValueRef, LLVMTypeRef)>,
    labels: HashMap<u16, (LLVMBasicBlockRef, LLVMBuilderRef)>,
    current_basic_block: Option<LLVMBasicBlockRef>,
    current_builder: Option<LLVMBuilderRef>,
}

impl Function {
    fn value_type(engine: LLVMExecutionEngineRef) -> LLVMTypeRef {
        // Biggest type is (u8, usize, usize): trait object
        // TODO: don't hardcode this
        // TODO2: don't hardcode usize as i64
        unsafe {
            let mut elements = [LLVMInt8Type(), LLVMInt64Type(), LLVMInt64Type()];
            let ty = LLVMStructType(elements.as_mut_ptr(), elements.len() as u32, 0);

            debug_assert!({
                let size = LLVMSizeOfTypeInBits(LLVMGetExecutionEngineTargetData(engine), ty);
                size == 24 * 8
            });

            ty
        }
    }
    fn create_function_type(engine: LLVMExecutionEngineRef) -> LLVMTypeRef {
        unsafe {
            let mut args = [LLVMPointerType(Self::value_type(engine), 0), LLVMInt64Type()];
            let ret = LLVMVoidType();
            LLVMFunctionType(ret, args.as_mut_ptr(), args.len() as u32, 0)
        }
    }

    pub fn new(backend: &Backend) -> Self {
        let value_union = Self::value_type(backend.engine());
        let ty = Self::create_function_type(backend.engine());
        let function = unsafe { LLVMAddFunction(backend.module(), cstr!("jit"), ty) };

        Self {
            function,
            locals: HashMap::new(),
            value_union,
            labels: HashMap::new(),
            current_basic_block: None,
            current_builder: None,
        }
    }

    fn append_block(&self) -> (LLVMBasicBlockRef, LLVMBuilderRef) {
        unsafe {
            let block = LLVMAppendBasicBlock(self.function, cstr!("block"));
            let builder = LLVMCreateBuilder();
            (block, builder)
        }
    }

    fn append_and_enter_block(&self) -> (LLVMBasicBlockRef, LLVMBuilderRef) {
        unsafe {
            let (block, builder) = self.append_block();
            LLVMPositionBuilderAtEnd(builder, block);
            (block, builder)
        }
    }

    fn alloca_local(&self, b: LLVMBuilderRef, ty: &Type) -> LLVMValueRef {
        unsafe { LLVMBuildAlloca(b, ty.to_llvm_type(), cstr!("local")) }
    }

    fn get_param(&self, param: u32) -> LLVMValueRef {
        unsafe { LLVMGetParam(self.function, param) }
    }

    fn const_i1(&self, value: bool) -> LLVMValueRef {
        unsafe { LLVMConstInt(LLVMInt1Type(), value as u64, 0) }
    }

    fn const_i32(&self, value: i32) -> LLVMValueRef {
        unsafe { LLVMConstInt(LLVMInt32Type(), value as u64, 0) }
    }

    fn const_i64(&self, value: i64) -> LLVMValueRef {
        unsafe { LLVMConstInt(LLVMInt64Type(), value as u64, 0) }
    }

    fn const_f64(&self, value: f64) -> LLVMValueRef {
        unsafe { LLVMConstInt(LLVMDoubleType(), value.to_bits(), 0) }
    }

    fn const_f32(&self, value: f32) -> LLVMValueRef {
        unsafe { LLVMConstInt(LLVMFloatType(), value.to_bits() as u64, 0) }
    }

    fn build_add(&self, builder: LLVMBuilderRef, a: LLVMValueRef, b: LLVMValueRef) -> LLVMValueRef {
        unsafe { LLVMBuildAdd(builder, a, b, cstr!("add")) }
    }

    fn build_sub(&self, builder: LLVMBuilderRef, a: LLVMValueRef, b: LLVMValueRef) -> LLVMValueRef {
        unsafe { LLVMBuildSub(builder, a, b, cstr!("sub")) }
    }

    fn build_mul(&self, builder: LLVMBuilderRef, a: LLVMValueRef, b: LLVMValueRef) -> LLVMValueRef {
        unsafe { LLVMBuildMul(builder, a, b, cstr!("mul")) }
    }

    fn build_fdiv(&self, builder: LLVMBuilderRef, a: LLVMValueRef, b: LLVMValueRef) -> LLVMValueRef {
        unsafe { LLVMBuildFDiv(builder, a, b, cstr!("fdiv")) }
    }

    fn build_frem(&self, builder: LLVMBuilderRef, a: LLVMValueRef, b: LLVMValueRef) -> LLVMValueRef {
        unsafe { LLVMBuildFRem(builder, a, b, cstr!("frem")) }
    }

    fn build_load(&self, builder: LLVMBuilderRef, ty: &Type, value: LLVMValueRef) -> LLVMValueRef {
        unsafe { LLVMBuildLoad2(builder, ty.to_llvm_type(), value, cstr!("load")) }
    }

    fn build_store(&self, builder: LLVMBuilderRef, value: LLVMValueRef, ptr: LLVMValueRef) -> LLVMValueRef {
        unsafe { LLVMBuildStore(builder, value, ptr) }
    }

    fn build_br(&self, builder: LLVMBuilderRef, to: LLVMBasicBlockRef) -> LLVMValueRef {
        unsafe { LLVMBuildBr(builder, to) }
    }

    fn build_fult(&self, builder: LLVMBuilderRef, a: LLVMValueRef, b: LLVMValueRef) -> LLVMValueRef {
        unsafe { LLVMBuildFCmp(builder, LLVMRealPredicate::LLVMRealULT, a, b, cstr!("lt")) }
    }

    fn build_fugt(&self, builder: LLVMBuilderRef, a: LLVMValueRef, b: LLVMValueRef) -> LLVMValueRef {
        unsafe { LLVMBuildFCmp(builder, LLVMRealPredicate::LLVMRealUGT, a, b, cstr!("gt")) }
    }

    fn create_jumpable_block(&mut self, ip: u16) -> (LLVMBasicBlockRef, LLVMBuilderRef) {
        let (block, builder) = self.append_block();
        self.labels.insert(ip, (block, builder));
        (block, builder)
    }

    fn position_builder_at(&self, builder: LLVMBuilderRef, to: LLVMBasicBlockRef) {
        unsafe { LLVMPositionBuilderAtEnd(builder, to) }
    }

    fn current_basic_block(&self) -> LLVMBasicBlockRef {
        self.current_basic_block.unwrap()
    }

    fn current_builder(&self) -> LLVMBuilderRef {
        self.current_builder.unwrap()
    }

    fn build_local_load(&self, builder: LLVMBuilderRef, id: u16) -> LLVMValueRef {
        let (local, ty) = self.locals[&id];
        unsafe { LLVMBuildLoad2(builder, ty, local, cstr!("local_load")) }
    }

    /// Initializes locals. Must be the first function to be called
    pub fn init_locals(&mut self, locals: &HashMap<u16, Type>) {
        let (block, builder) = self.append_and_enter_block();
        self.current_basic_block = Some(block);
        self.current_builder = Some(builder);

        for (&local_index, ty) in locals {
            let space = self.alloca_local(builder, ty);

            let stack_ptr = self.get_param(0);
            let stack_offset = self.get_param(1);
            let index = self.const_i64(local_index as i64);
            let stack_offset = self.build_add(builder, stack_offset, index);

            let mut indices = [stack_offset];
            let gep = unsafe {
                LLVMBuildGEP2(
                    builder,
                    self.value_union,
                    stack_ptr,
                    indices.as_mut_ptr(),
                    indices.len() as u32,
                    cstr!("gep"),
                )
            };

            let value = self.build_load(builder, &ty, gep);
            self.build_store(builder, value, space);

            self.locals.insert(local_index, (space, ty.to_llvm_type()));
        }
    }

    pub fn compile_trace<Q: CompileQuery>(&mut self, bytecode: &[u8], q: Q, infer: &InferResult, trace: &Trace) {
        let mut cx = CompilationContext::new(self, bytecode);

        while let Some((index, instr)) = cx.next_instruction() {
            let is_label = infer.labels.get(index).unwrap();
            if is_label {
                let (block, builder) = cx.create_jumpable_block(index as u16);
                cx.build_br(cx.current_builder, block);
                cx.position_builder_at(builder, block);
            }

            match instr {
                Instruction::Add => cx.with2(|cx, a, b| cx.build_add(cx.current_builder, a, b)),
                Instruction::Sub => cx.with2(|cx, a, b| cx.build_sub(cx.current_builder, a, b)),
                Instruction::Mul => cx.with2(|cx, a, b| cx.build_mul(cx.current_builder, a, b)),
                Instruction::Div => cx.with2(|cx, a, b| cx.build_fdiv(cx.current_builder, a, b)),
                Instruction::Rem => cx.with2(|cx, a, b| cx.build_frem(cx.current_builder, a, b)),
                Instruction::LdLocal => {
                    let id = cx.next_byte();
                    let value = cx.build_local_load(cx.current_builder, id.into());
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
                Instruction::Lt => cx.with2(|cx, a, b| cx.build_fult(cx.current_builder, a, b)),
                Instruction::Gt => cx.with2(|cx, a, b| cx.build_fugt(cx.current_builder, a, b)),
                Instruction::JmpFalseP => {
                    let value = cx.pop();
                    let did_take = trace.conditional_jumps[index];
                    // TODO: Emit guard here to assert value is false or true, depending on value of `did_take`
                }
                _ => panic!("Unimplemented instruction: {:?}", instr),
            }
        }
    }

    pub fn verify(&self) {
        unsafe { LLVMVerifyFunction(self.function, LLVMVerifierFailureAction::LLVMAbortProcessAction) };
    }
}

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
    current_builder: LLVMBuilderRef,
    current_block: LLVMBasicBlockRef,
    value_stack: Vec<LLVMValueRef>,
}

impl<'fun, 'bytecode> CompilationContext<'fun, 'bytecode> {
    pub fn new(function: &'fun mut Function, bytecode: &'bytecode [u8]) -> Self {
        let current_builder = function.current_builder();
        let current_block = function.current_basic_block();

        Self {
            iter: bytecode.iter().enumerate(),
            function,
            current_builder,
            current_block,
            value_stack: Vec::new(),
        }
    }

    pub fn with2<F>(&mut self, fun: F)
    where
        F: Fn(&mut CompilationContext<'fun, 'bytecode>, LLVMValueRef, LLVMValueRef) -> LLVMValueRef,
    {
        let (a, b) = self.pop2();
        let result = fun(self, a, b);
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
