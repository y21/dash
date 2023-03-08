use std::collections::HashMap;

use llvm_sys::execution_engine::LLVMExecutionEngineRef;
use llvm_sys::execution_engine::LLVMGetFunctionAddress;
use llvm_sys::prelude::LLVMBasicBlockRef;
use llvm_sys::prelude::LLVMBuilderRef;
use llvm_sys::prelude::LLVMContextRef;
use llvm_sys::prelude::LLVMModuleRef;
use llvm_sys::prelude::LLVMPassManagerRef;
use llvm_sys::prelude::LLVMTypeRef;
use llvm_sys::prelude::LLVMValueRef;
use llvm_sys::target_machine::LLVMCodeGenOptLevel;

use crate::llvm_wrapper as llvm;
use crate::llvm_wrapper::Value;
use crate::passes::bb_generation::BasicBlockMap;
use crate::passes::type_infer::Type;
use crate::passes::type_infer::TypeMap;

use cstr::cstr;

fn value_ty_in_context(cx: &llvm::Context, ee: &llvm::ExecutionEngine) -> llvm::Ty {
    let mut elements = [
        // Discriminant
        cx.i8_ty(),
        // Data ptr
        cx.i64_ty(),
        // Vtable ptr
        cx.i64_ty(),
    ];
    let value = cx.struct_ty_unpacked(&mut elements);
    debug_assert_eq!(ee.size_of_ty_bits(&value), usize::BITS as usize * 3);
    value
}

pub struct CodegenCtxt {
    pub ty_map: TypeMap,
    pub bb_map: BasicBlockMap,

    pub llcx: llvm::Context,
    pub module: llvm::Module,
    pub ee: llvm::ExecutionEngine,
    pub pm: llvm::PassManager,
    pub function: llvm::Function,
    pub value_ty: llvm::Ty,
    pub locals: HashMap<usize, (llvm::Value, llvm::Ty)>,
    pub llvm_bbs: HashMap<usize, llvm::BasicBlock>,
    pub builder: llvm::Builder,
    pub setup_block: llvm::BasicBlock,
    pub exit_block: llvm::BasicBlock,
    pub exit_guards: Vec<(usize, llvm::BasicBlock)>,
}

impl CodegenCtxt {
    pub fn new(ty_map: TypeMap, bb_map: BasicBlockMap) -> Self {
        let mut llcx = llvm::Context::new();
        let module = llcx.create_module();
        let ee = module.create_execution_engine();
        let pm = llvm::PassManager::new(LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive);
        let value_ty = value_ty_in_context(&llcx, &ee);
        let function = module.create_c_function(&value_ty);
        let locals = HashMap::new();
        let llvm_bbs = HashMap::new();
        let builder = llcx.create_builder();
        let setup_block = llcx.append_basic_block(&function, cstr!("setup"));
        let exit_block = llcx.append_basic_block(&function, cstr!("exit"));
        let exit_guards = Vec::new();

        Self {
            ty_map,
            bb_map,
            llcx,
            module,
            ee,
            pm,
            function,
            value_ty,
            locals,
            llvm_bbs,
            builder,
            setup_block,
            exit_block,
            exit_guards,
        }
    }

    fn alloca_local(&self, t: &Type) -> Value {
        self.builder.build_alloca(&self.llcx.mir_ty_to_llvm_ty(t))
    }

    fn cast_mir(&self, value: &Value, from: &Type, to: &Type) -> Value {
        match (from, to) {
            (Type::I64, Type::Boolean) => self.builder.build_trunc(&self.llcx.i1_ty(), value),
            (Type::F64, Type::Boolean) => {
                let to_int = self.cast_mir(value, from, &Type::I64);
                self.cast_mir(&to_int, &Type::I64, &Type::Boolean)
            }
            (Type::Boolean, Type::I64) => self.builder.build_sext(&self.llcx.i64_ty(), value),
            (Type::Boolean, Type::F64) => {
                let to_int = self.cast_mir(value, from, &Type::I64);
                self.cast_mir(&to_int, &Type::I64, &Type::F64)
            }
            (Type::I64, Type::F64) => self.builder.build_si2fp(&self.llcx.f64_ty(), value),
            (Type::F64, Type::I64) => self.builder.build_fp2si(&self.llcx.i64_ty(), value),
            _ => panic!("Invalid cast {:?} -> {:?}", from, to),
        }
    }

    /// Compiles the setup block. This block allocates stack space
    /// for the referenced local variables and has various checks.
    ///
    /// This function should be called before compiling other parts.
    pub fn compile_setup_block(&mut self) {
        self.builder.position_at_end(&self.setup_block);

        for (&id, ty) in &self.ty_map {
            // Allocate space for local
            let space = self.alloca_local(ty);

            // Copy value from the VM stack to the JIT stack
            let stack_ptr = self.function.get_param(0);
            let stack_offset = self.function.get_param(1);
            let index = self.llcx.const_i64(id as i64);

            let stack_offset = self.builder.build_add(&stack_offset, &index);
            let ptr = self
                .builder
                .build_gep(&self.value_ty, &stack_ptr, &mut [stack_offset, self.llcx.const_i32(1)]);

            let value = self.builder.build_load(&self.value_ty, &ptr);

            // Cast to appropriate type, since `value` is currently an i64
            // which is wrong in any case.
            let value = match ty {
                Type::Boolean => self.cast_mir(&value, &Type::I64, ty),
                Type::I64 => {
                    // even though value is of type i64, it only contains the raw bits
                    // so we need to do a i64 -> f64 -> i64 roundtrip
                    let as_f64 = self.builder.build_bitcast(&self.llcx.f64_ty(), &value);
                    self.cast_mir(&as_f64, &Type::F64, &Type::I64)
                }
                Type::F64 => self.builder.build_bitcast(&self.llcx.f64_ty(), &value),
            };

            // Finally, copy the cast value to the allocated space
            self.builder.build_store(&value, &space);

            self.locals.insert(id, (space, self.llcx.mir_ty_to_llvm_ty(ty)));
        }
    }

    /// Compiles the exit block.
    ///
    /// This function should be called after compiling other parts.
    pub fn compile_exit_block(&mut self) {
        // Jump to exit block
        self.builder.position_at_end(&self.exit_block);

        let mut ret_phi = self.builder.build_phi(&self.llcx.i64_ty());
        for (ip, block) in &self.exit_guards {
            ret_phi.add_incoming(&self.llcx.const_i64(*ip as i64), block);
        }
        let out_ip = self.function.get_param(2);
        self.builder.build_store(ret_phi.as_value(), &out_ip);

        for (&local_index, ty) in &self.ty_map {
            let (space, llty) = &self.locals[&local_index];
            let value = self.builder.build_load(llty, space);

            // Cast the type we have on the JIT stack back to an i64
            // so it matches the out pointer in the fn signature.
            let value = match ty {
                Type::Boolean => self.cast_mir(&value, ty, &Type::I64),
                Type::I64 => {
                    let as_f64 = self.cast_mir(&value, ty, &Type::F64);
                    self.builder.build_bitcast(&self.llcx.i64_ty(), &as_f64)
                }
                Type::F64 => self.builder.build_bitcast(&self.llcx.i64_ty(), &value),
            };

            let stack_ptr = self.function.get_param(0);
            let stack_offset = self.function.get_param(1);
            let index = self.llcx.const_i64(local_index as i64);
            let stack_offset = self.builder.build_add(&stack_offset, &index);
            let dest = self
                .builder
                .build_gep(&self.value_ty, &stack_ptr, &mut [stack_offset, self.llcx.const_i32(1)]);

            self.builder.build_store(&value, &dest);
        }

        self.builder.build_retvoid();
    }

    /// Compiles the bytecode to LLVM IR.
    pub fn compile_bbs(&mut self) {}
}
