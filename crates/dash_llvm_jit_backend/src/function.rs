use std::cell::Cell;
use std::collections::HashMap;
use std::ffi::CStr;
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
use llvm_sys::core::LLVMBuildBitCast;
use llvm_sys::core::LLVMBuildBr;
use llvm_sys::core::LLVMBuildCondBr;
use llvm_sys::core::LLVMBuildFAdd;
use llvm_sys::core::LLVMBuildFCmp;
use llvm_sys::core::LLVMBuildFDiv;
use llvm_sys::core::LLVMBuildFMul;
use llvm_sys::core::LLVMBuildFRem;
use llvm_sys::core::LLVMBuildFSub;
use llvm_sys::core::LLVMBuildGEP2;
use llvm_sys::core::LLVMBuildLoad2;
use llvm_sys::core::LLVMBuildMul;
use llvm_sys::core::LLVMBuildRet;
use llvm_sys::core::LLVMBuildRetVoid;
use llvm_sys::core::LLVMBuildStore;
use llvm_sys::core::LLVMBuildStructGEP2;
use llvm_sys::core::LLVMBuildSub;
use llvm_sys::core::LLVMConstInt;
use llvm_sys::core::LLVMConstReal;
use llvm_sys::core::LLVMCreateBuilder;
use llvm_sys::core::LLVMDoubleType;
use llvm_sys::core::LLVMFloatType;
use llvm_sys::core::LLVMFunctionType;
use llvm_sys::core::LLVMGetParam;
use llvm_sys::core::LLVMGetValueName2;
use llvm_sys::core::LLVMInt16TypeInContext;
use llvm_sys::core::LLVMInt1Type;
use llvm_sys::core::LLVMInt32Type;
use llvm_sys::core::LLVMInt64Type;
use llvm_sys::core::LLVMInt8Type;
use llvm_sys::core::LLVMPointerType;
use llvm_sys::core::LLVMPositionBuilderAtEnd;
use llvm_sys::core::LLVMPrintValueToString;
use llvm_sys::core::LLVMSetInstructionCallConv;
use llvm_sys::core::LLVMSizeOf;
use llvm_sys::core::LLVMStructType;
use llvm_sys::core::LLVMVoidType;
use llvm_sys::execution_engine::LLVMExecutionEngineRef;
use llvm_sys::execution_engine::LLVMGetExecutionEngineTargetData;
use llvm_sys::execution_engine::LLVMGetFunctionAddress;
use llvm_sys::prelude::LLVMBasicBlockRef;
use llvm_sys::prelude::LLVMBuilderRef;
use llvm_sys::prelude::LLVMModuleRef;
use llvm_sys::prelude::LLVMTypeRef;
use llvm_sys::prelude::LLVMValueRef;
use llvm_sys::target::LLVMABISizeOfType;
use llvm_sys::target::LLVMSizeOfTypeInBits;
use llvm_sys::LLVMCallConv;
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
    setup_block: Option<LLVMBasicBlockRef>,
    setup_builder: Option<LLVMBuilderRef>,
    exit_block: Option<LLVMBasicBlockRef>,
    exit_builder: Option<LLVMBuilderRef>,
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
        unsafe { LLVMSetInstructionCallConv(function, LLVMCallConv::LLVMCCallConv as u32) }

        Self {
            function,
            locals: HashMap::new(),
            value_union,
            labels: HashMap::new(),
            setup_block: None,
            setup_builder: None,
            exit_block: None,
            exit_builder: None,
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

    fn i64_ty(&self) -> LLVMTypeRef {
        unsafe { LLVMInt64Type() }
    }

    fn f64_ty(&self) -> LLVMTypeRef {
        unsafe { LLVMDoubleType() }
    }

    fn const_i64(&self, value: i64) -> LLVMValueRef {
        unsafe { LLVMConstInt(LLVMInt64Type(), value as u64, 0) }
    }

    fn const_f64(&self, value: f64) -> LLVMValueRef {
        unsafe { LLVMConstReal(LLVMDoubleType(), value) }
    }

    fn const_f32(&self, value: f32) -> LLVMValueRef {
        unsafe { LLVMConstReal(LLVMDoubleType(), value as f64) }
    }

    fn build_add(&self, builder: LLVMBuilderRef, a: LLVMValueRef, b: LLVMValueRef) -> LLVMValueRef {
        unsafe { LLVMBuildAdd(builder, a, b, cstr!("add")) }
    }

    // TODO: merge this function with build_add, check for types in there
    fn build_fadd(&self, builder: LLVMBuilderRef, a: LLVMValueRef, b: LLVMValueRef) -> LLVMValueRef {
        unsafe { LLVMBuildFAdd(builder, a, b, cstr!("fadd")) }
    }

    fn build_sub(&self, builder: LLVMBuilderRef, a: LLVMValueRef, b: LLVMValueRef) -> LLVMValueRef {
        unsafe { LLVMBuildSub(builder, a, b, cstr!("sub")) }
    }

    // TODO: merge this function with build_sub, check for types in there
    fn build_fsub(&self, builder: LLVMBuilderRef, a: LLVMValueRef, b: LLVMValueRef) -> LLVMValueRef {
        unsafe { LLVMBuildFSub(builder, a, b, cstr!("fsub")) }
    }

    fn build_mul(&self, builder: LLVMBuilderRef, a: LLVMValueRef, b: LLVMValueRef) -> LLVMValueRef {
        unsafe { LLVMBuildMul(builder, a, b, cstr!("mul")) }
    }

    // TODO: merge this function with build_sub, check for types in there
    fn build_fmul(&self, builder: LLVMBuilderRef, a: LLVMValueRef, b: LLVMValueRef) -> LLVMValueRef {
        unsafe { LLVMBuildFMul(builder, a, b, cstr!("fmul")) }
    }

    fn build_fdiv(&self, builder: LLVMBuilderRef, a: LLVMValueRef, b: LLVMValueRef) -> LLVMValueRef {
        unsafe { LLVMBuildFDiv(builder, a, b, cstr!("fdiv")) }
    }

    fn build_frem(&self, builder: LLVMBuilderRef, a: LLVMValueRef, b: LLVMValueRef) -> LLVMValueRef {
        unsafe { LLVMBuildFRem(builder, a, b, cstr!("frem")) }
    }

    fn build_load(&self, builder: LLVMBuilderRef, ty: LLVMTypeRef, value: LLVMValueRef) -> LLVMValueRef {
        unsafe { LLVMBuildLoad2(builder, ty, value, cstr!("load")) }
    }

    fn build_store(&self, builder: LLVMBuilderRef, value: LLVMValueRef, ptr: LLVMValueRef) -> LLVMValueRef {
        unsafe { LLVMBuildStore(builder, value, ptr) }
    }

    fn build_br(&self, builder: LLVMBuilderRef, to: LLVMBasicBlockRef) -> LLVMValueRef {
        unsafe { LLVMBuildBr(builder, to) }
    }

    fn build_condbr(
        &self,
        builder: LLVMBuilderRef,
        condition: LLVMValueRef,
        dest_true: LLVMBasicBlockRef,
        dest_false: LLVMBasicBlockRef,
    ) -> LLVMValueRef {
        unsafe { LLVMBuildCondBr(builder, condition, dest_true, dest_false) }
    }

    fn build_fult(&self, builder: LLVMBuilderRef, a: LLVMValueRef, b: LLVMValueRef) -> LLVMValueRef {
        unsafe { LLVMBuildFCmp(builder, LLVMRealPredicate::LLVMRealULT, a, b, cstr!("lt")) }
    }

    fn build_fugt(&self, builder: LLVMBuilderRef, a: LLVMValueRef, b: LLVMValueRef) -> LLVMValueRef {
        unsafe { LLVMBuildFCmp(builder, LLVMRealPredicate::LLVMRealUGT, a, b, cstr!("gt")) }
    }

    fn build_local_load(&self, builder: LLVMBuilderRef, id: u16) -> LLVMValueRef {
        let (local, ty) = self.locals[&id];
        unsafe { LLVMBuildLoad2(builder, ty, local, cstr!("local_load")) }
    }

    fn build_retvoid(&self, builder: LLVMBuilderRef) -> LLVMValueRef {
        unsafe { LLVMBuildRetVoid(builder) }
    }

    fn build_bitcast(&self, builder: LLVMBuilderRef, value: LLVMValueRef, to: LLVMTypeRef) -> LLVMValueRef {
        unsafe { LLVMBuildBitCast(builder, value, to, cstr!("bitcast")) }
    }

    fn create_jumpable_block(&mut self, ip: u16) -> (LLVMBasicBlockRef, LLVMBuilderRef) {
        let (block, builder) = self.append_block();
        self.labels.insert(ip, (block, builder));
        (block, builder)
    }

    fn position_builder_at(&self, builder: LLVMBuilderRef, to: LLVMBasicBlockRef) {
        unsafe { LLVMPositionBuilderAtEnd(builder, to) }
    }

    fn setup_block(&self) -> LLVMBasicBlockRef {
        self.setup_block.unwrap()
    }

    fn setup_builder(&self) -> LLVMBuilderRef {
        self.setup_builder.unwrap()
    }

    fn exit_block(&self) -> LLVMBasicBlockRef {
        self.exit_block.unwrap()
    }

    fn exit_builder(&self) -> LLVMBuilderRef {
        self.exit_builder.unwrap()
    }

    /// Initializes locals. Must be the first function to be called
    pub fn init_locals(&mut self, locals: &HashMap<u16, Type>) {
        let (setup_block, setup_builder) = self.append_and_enter_block();
        self.setup_block = Some(setup_block);
        self.setup_builder = Some(setup_builder);

        for (&local_index, ty) in locals {
            let space = self.alloca_local(setup_builder, ty);

            let stack_ptr = self.get_param(0);
            let stack_offset = self.get_param(1);
            let index = self.const_i64(local_index as i64);
            let stack_offset = self.build_add(setup_builder, stack_offset, index);

            let mut indices = [stack_offset, self.const_i32(1)];
            let value_ptr = unsafe {
                LLVMBuildGEP2(
                    setup_builder,
                    self.value_union,
                    stack_ptr,
                    indices.as_mut_ptr(),
                    indices.len() as u32,
                    cstr!("gep"),
                )
            };

            let value = self.build_load(setup_builder, self.i64_ty(), value_ptr);
            let value = self.build_bitcast(setup_builder, value, ty.to_llvm_type());
            self.build_store(setup_builder, value, space);

            self.locals.insert(local_index, (space, ty.to_llvm_type()));
        }

        // Compile exit block
        // Write all the values back to the stack
        let (exit_block, exit_builder) = self.append_block();
        self.position_builder_at(exit_builder, exit_block);

        self.exit_block = Some(exit_block);
        self.exit_builder = Some(exit_builder);

        for (&local_index, ty) in locals {
            let (space, _) = self.locals[&local_index];
            let value = self.build_local_load(exit_builder, local_index);
            let value = self.build_bitcast(exit_builder, value, self.i64_ty());

            let stack_ptr = self.get_param(0);
            let stack_offset = self.get_param(1);
            let index = self.const_i64(local_index as i64);
            let stack_offset = self.build_add(exit_builder, stack_offset, index);

            let mut indices = [stack_offset, self.const_i32(1)];
            let value_ptr = unsafe {
                LLVMBuildGEP2(
                    exit_builder,
                    self.value_union,
                    stack_ptr,
                    indices.as_mut_ptr(),
                    indices.len() as u32,
                    cstr!("gep"),
                )
            };

            self.build_store(exit_builder, value, value_ptr);
        }
        self.build_retvoid(exit_builder);
    }

    pub fn compile_trace<Q: CompileQuery>(&mut self, bytecode: &[u8], q: &Q, infer: &InferResult, trace: &Trace) {
        let mut cx = CompilationContext::new(self, bytecode);

        let mut jumps = 0;

        while let Some((index, instr)) = cx.next_instruction() {
            let is_label = infer.labels.get(index).unwrap();
            if is_label {
                let (block, builder) = cx.create_jumpable_block(index as u16);
                cx.build_br(cx.current_builder, block);
                cx.position_builder_at(cx.current_builder, block);
            }

            match instr {
                Instruction::Add => cx.with2(|cx, a, b| cx.build_fadd(cx.current_builder, a, b)),
                Instruction::Sub => cx.with2(|cx, a, b| cx.build_fsub(cx.current_builder, a, b)),
                Instruction::Mul => cx.with2(|cx, a, b| cx.build_fmul(cx.current_builder, a, b)),
                Instruction::Div => cx.with2(|cx, a, b| cx.build_fdiv(cx.current_builder, a, b)),
                Instruction::Rem => cx.with2(|cx, a, b| cx.build_frem(cx.current_builder, a, b)),
                Instruction::LdLocal => {
                    let id = cx.next_byte();
                    let value = cx.build_local_load(cx.current_builder, id.into());
                    cx.push(value);
                }
                Instruction::StoreLocal => {
                    let id = cx.next_byte();
                    let value = cx.pop();
                    let (local, ty) = cx.locals[&id.into()];
                    cx.build_store(cx.current_builder, value, local);
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
                Instruction::Pop => drop(cx.pop()),
                Instruction::Lt => cx.with2(|cx, a, b| cx.build_fult(cx.current_builder, a, b)),
                Instruction::Gt => cx.with2(|cx, a, b| cx.build_fugt(cx.current_builder, a, b)),
                Instruction::Jmp => {
                    let count = cx.next_wide() as i16;
                    let target_ip = (index as isize + count as isize) + 3;
                    let (target_block, target_builder) = cx.labels[&(target_ip as u16)];
                    cx.build_br(cx.current_builder, target_block);
                }
                Instruction::JmpFalseP => {
                    let count = cx.next_wide() as i16;
                    let value = cx.pop();
                    let did_take = trace.conditional_jumps[jumps];
                    jumps += 1;
                    cx.emit_guard(value, !did_take);
                }
                _ => panic!("Unimplemented instruction: {:?}", instr),
            }
        }
    }

    pub fn verify(&self) {
        unsafe { LLVMVerifyFunction(self.function, LLVMVerifierFailureAction::LLVMAbortProcessAction) };
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
    exit_builder: LLVMBuilderRef,
    exit_block: LLVMBasicBlockRef,
    value_stack: Vec<LLVMValueRef>,
}

impl<'fun, 'bytecode> CompilationContext<'fun, 'bytecode> {
    pub fn new(function: &'fun mut Function, bytecode: &'bytecode [u8]) -> Self {
        let current_builder = function.setup_builder();
        let current_block = function.setup_block();
        let exit_builder = function.exit_builder();
        let exit_block = function.exit_block();

        Self {
            iter: bytecode.iter().enumerate(),
            function,
            current_builder,
            current_block,
            exit_block,
            exit_builder,
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

    pub fn emit_guard(&self, condition: LLVMValueRef, expected: bool) {
        let (next_block, next_builder) = self.append_block();
        let (dest_true, dest_false) = match expected {
            true => (next_block, self.exit_block),
            false => (self.exit_block, next_block),
        };
        self.build_condbr(self.current_builder, condition, dest_true, dest_false);
        self.position_builder_at(self.current_builder, next_block);
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
