use std::collections::HashMap;
use std::collections::HashSet;

use dash_middle::compiler::instruction::AssignKind;
use dash_middle::compiler::instruction::Instruction;
use dash_middle::compiler::instruction::IntrinsicOperation;
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
use llvm_sys::LLVMTypeKind;

use crate::error::Error;
use crate::llvm_wrapper as llvm;
use crate::llvm_wrapper::Value;
use crate::passes::bb_generation::BasicBlockKey;
use crate::passes::bb_generation::BasicBlockMap;
use crate::passes::bb_generation::BasicBlockSuccessor;
use crate::passes::bb_generation::ConditionalBranchAction;
use crate::passes::type_infer::Type;
use crate::passes::type_infer::TypeMap;
use crate::typed_cfg::TypedCfg;
use crate::util::DecodeCtxt;

use cstr::cstr;

pub type JitFunction = unsafe extern "C" fn(
    *mut (),  // stack pointer
    u64,      // stack offset for frame
    *mut u64, // out pointer for the IP after exiting
);

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

fn function_type(cx: &llvm::Context, ee: &llvm::ExecutionEngine) -> llvm::Ty {
    let mut args = [
        cx.pointer_ty(&value_ty_in_context(cx, ee)),
        cx.i64_ty(),
        cx.pointer_ty(&cx.i64_ty()),
    ];
    let ret = cx.void_ty();
    cx.function_ty(&ret, &mut args)
}

/// Recursively registers all reachable basic blocks
/// (i.e. actioned successor blocks)
fn register_llvm_bbs(
    llcx: &llvm::Context,
    func: &llvm::Function,
    bb_map: &BasicBlockMap,
    llvm_bbs: &mut HashMap<usize, llvm::BasicBlock>,
    bbk: BasicBlockKey,
    visited: &mut HashSet<usize>,
) {
    if visited.contains(&bbk) {
        return;
    }
    visited.insert(bbk);

    let bb = llcx.append_basic_block(func, cstr!("bb"));
    llvm_bbs.insert(bbk, bb);

    match &bb_map[&bbk].successor {
        Some(BasicBlockSuccessor::Conditional {
            true_ip,
            false_ip,
            action,
        }) => {
            if let Some(ConditionalBranchAction::Either | ConditionalBranchAction::Taken) = action {
                register_llvm_bbs(llcx, func, bb_map, llvm_bbs, *true_ip, visited);
            }
            if let Some(ConditionalBranchAction::Either | ConditionalBranchAction::NotTaken) = action {
                register_llvm_bbs(llcx, func, bb_map, llvm_bbs, *false_ip, visited);
            }
        }
        Some(BasicBlockSuccessor::Unconditional(target)) => {
            register_llvm_bbs(llcx, func, bb_map, llvm_bbs, *target, visited);
        }
        None => {}
    }
}

pub enum JitConstant {
    Boolean(bool),
    I64(i64),
    F64(f64),
}

impl JitConstant {
    pub fn to_llvm_value(&self, llcx: &llvm::Context) -> Value {
        match self {
            Self::Boolean(b) => llcx.const_i1(*b),
            JitConstant::I64(i) => llcx.const_i64(*i),
            JitConstant::F64(f) => llcx.const_f64(*f),
        }
    }
}

pub trait CodegenQuery {
    fn get_constant(&self, cid: u16) -> JitConstant;
}

pub struct CodegenCtxt<'a, 'q, Q> {
    pub ty_map: &'q TypeMap,
    pub bb_map: &'q BasicBlockMap,
    pub bytecode: &'a [u8],
    pub query: &'q mut Q,

    pub bbs_visited: HashSet<BasicBlockKey>,
    pub llcx: llvm::Context,
    pub module: llvm::Module,
    pub ee: llvm::ExecutionEngine,
    pub pm: llvm::PassManager,
    pub function: llvm::Function,
    pub value_ty: llvm::Ty,
    pub locals: HashMap<u16, (llvm::Value, llvm::Ty)>,
    pub llvm_bbs: HashMap<usize, llvm::BasicBlock>,
    pub builder: llvm::Builder,
    pub setup_block: llvm::BasicBlock,
    pub exit_block: llvm::BasicBlock,
    pub exit_guards: Vec<(usize, llvm::BasicBlock)>,
}

impl<'a, 'q, Q: CodegenQuery> CodegenCtxt<'a, 'q, Q> {
    pub fn new(ty_map: &'q TypeMap, bb_map: &'q BasicBlockMap, bytecode: &'a [u8], query: &'q mut Q) -> Self {
        let mut llcx = llvm::Context::new();
        let module = llcx.create_module();
        let ee = module.create_execution_engine();
        let pm = llvm::PassManager::new(LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive);
        let value_ty = value_ty_in_context(&llcx, &ee);
        let function = module.create_c_function(&function_type(&llcx, &ee));
        let locals = HashMap::new();
        let mut llvm_bbs = HashMap::new();
        let builder = llcx.create_builder();
        let setup_block = llcx.append_basic_block(&function, cstr!("setup"));
        let exit_block = llcx.append_basic_block(&function, cstr!("exit"));
        let exit_guards = Vec::new();

        register_llvm_bbs(&llcx, &function, &bb_map, &mut llvm_bbs, 0, &mut HashSet::new());

        Self {
            ty_map,
            bb_map,
            bbs_visited: HashSet::new(),
            query,
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
            bytecode,
        }
    }

    fn alloca_local(&self, t: &Type) -> Value {
        self.builder.build_alloca(&self.llcx.mir_ty_to_llvm_ty(t))
    }

    fn load_local(&self, id: u16) -> Value {
        let (val, ty) = &self.locals[&id];
        self.builder.build_load(ty, val)
    }

    fn store_local(&self, id: u16, value: &Value) -> Value {
        let (dest, _) = &self.locals[&id];
        self.builder.build_store(value, dest)
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

        for (&id, ty) in self.ty_map.iter() {
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

            let value = self.builder.build_load(&self.llcx.i64_ty(), &ptr);

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

        let first_bb = &self.llvm_bbs[&0];
        self.builder.build_br(first_bb);
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

        for (&local_index, ty) in self.ty_map.iter() {
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

    pub fn compile_bb(&mut self, mut stack: ValueStack, bbk: BasicBlockKey) -> Result<(), Error> {
        if self.bbs_visited.contains(&bbk) {
            return Ok(());
        }
        self.bbs_visited.insert(bbk);

        let (mut dcx, succ, block_offset) = {
            let bb = &self.bb_map[&bbk];
            let bytecode = &self.bytecode[bb.index..bb.end];

            (DecodeCtxt::new(bytecode), bb.successor, bb.index)
        };

        let bb = &self.llvm_bbs[&bbk];
        self.builder.position_at_end(bb);

        while let Some((index, instr)) = dcx.next_instruction() {
            match instr {
                Instruction::Add => stack.binop(|a, b| self.builder.build_add(&a, &b)),
                Instruction::Sub => stack.binop(|a, b| self.builder.build_sub(&a, &b)),
                Instruction::Mul => stack.binop(|a, b| self.builder.build_mul(&a, &b)),
                Instruction::Div => stack.binop(|a, b| self.builder.build_div(&a, &b)),
                Instruction::Rem => stack.binop(|a, b| self.builder.build_rem(&a, &b)),
                Instruction::Lt => stack.binop(|a, b| self.builder.build_lt(&a, &b)),
                Instruction::Gt => stack.binop(|a, b| self.builder.build_gt(&a, &b)),
                Instruction::Le => stack.binop(|a, b| self.builder.build_le(&a, &b)),
                Instruction::Ge => stack.binop(|a, b| self.builder.build_ge(&a, &b)),
                Instruction::Eq => stack.binop(|a, b| self.builder.build_eq(&a, &b)),
                Instruction::Ne => stack.binop(|a, b| self.builder.build_ne(&a, &b)),
                Instruction::LdLocal => {
                    let id = dcx.next_byte();
                    let val = self.load_local(id.into());
                    stack.push(val);
                }
                Instruction::StoreLocal => {
                    let id = dcx.next_byte();
                    let kind = AssignKind::from_repr(dcx.next_byte()).unwrap();
                    assert_eq!(kind, AssignKind::Assignment);
                    let value = stack.pop();
                    self.store_local(id.into(), &value);
                    let value = self.load_local(id.into());
                    stack.push(value);
                }
                Instruction::Constant => {
                    let cid = dcx.next_byte();
                    let constant = self.query.get_constant(cid.into());
                    stack.push(constant.to_llvm_value(&self.llcx));
                }
                Instruction::Pop => drop(stack.pop()),
                Instruction::Jmp => {
                    let bb = &self.bb_map[&bbk];
                    let Some(BasicBlockSuccessor::Unconditional(target)) = &bb.successor else {
                        panic!("unmatched basic block successor");
                    };
                    let llbb = &self.llvm_bbs[target];
                    self.builder.build_br(llbb);
                    self.compile_bb(stack.clone(), *target);

                    return Ok(());
                }
                Instruction::JmpFalseP
                | Instruction::JmpFalseNP
                | Instruction::JmpTrueP
                | Instruction::JmpTrueNP
                | Instruction::JmpNullishP
                | Instruction::JmpNullishNP
                | Instruction::JmpUndefinedNP
                | Instruction::JmpUndefinedP => {
                    let cond = match instr {
                        Instruction::JmpFalseP
                        | Instruction::JmpNullishP
                        | Instruction::JmpTrueP
                        | Instruction::JmpUndefinedP => stack.pop(),
                        _ => stack.last(),
                    };

                    let count = dcx.next_wide_signed();
                    let _target_ip = usize::try_from(index as i16 + count + 3).unwrap();
                    let bb = &self.bb_map[&bbk];
                    let Some(BasicBlockSuccessor::Conditional { true_ip, false_ip, action: Some(action) }) = bb.successor else {
                        panic!("unmatched basic block successor");
                    };

                    let (true_ip, false_ip) = match instr {
                        Instruction::JmpTrueNP | Instruction::JmpTrueP => (true_ip, false_ip),
                        Instruction::JmpFalseNP | Instruction::JmpFalseP => (false_ip, true_ip),
                        _ => todo!(),
                    };
                    let llbb = self.llvm_bbs[&bbk].clone();

                    match action {
                        ConditionalBranchAction::Either => {
                            let true_bb = &self.llvm_bbs[&true_ip];
                            let false_bb = &self.llvm_bbs[&false_ip];
                            self.builder.build_condbr(&cond, true_bb, false_bb);
                            self.compile_bb(stack.clone(), true_ip);
                            self.compile_bb(stack.clone(), false_ip);
                        }
                        ConditionalBranchAction::NotTaken => {
                            let bb = &self.llvm_bbs[&true_ip];
                            self.exit_guards.push((false_ip, llbb));

                            self.builder.build_condbr(&cond, bb, &self.exit_block);
                            self.compile_bb(stack.clone(), true_ip);
                        }
                        ConditionalBranchAction::Taken => {
                            let bb = &self.llvm_bbs[&false_ip];
                            self.exit_guards.push((true_ip, llbb));

                            self.builder.build_condbr(&cond, &self.exit_block, bb);
                            self.compile_bb(stack.clone(), false_ip);
                        }
                    }

                    return Ok(());
                }
                Instruction::IntrinsicOp => {
                    let op = IntrinsicOperation::from_repr(dcx.next_byte()).unwrap();

                    match op {
                        IntrinsicOperation::AddNumLR => stack.binop(|a, b| self.builder.build_add(&a, &b)),
                        IntrinsicOperation::SubNumLR => stack.binop(|a, b| self.builder.build_sub(&a, &b)),
                        IntrinsicOperation::MulNumLR => stack.binop(|a, b| self.builder.build_mul(&a, &b)),
                        IntrinsicOperation::DivNumLR => stack.binop(|a, b| self.builder.build_div(&a, &b)),
                        IntrinsicOperation::RemNumLR => stack.binop(|a, b| self.builder.build_rem(&a, &b)),
                        IntrinsicOperation::GtNumLR => stack.binop(|a, b| self.builder.build_gt(&a, &b)),
                        IntrinsicOperation::GeNumLR => stack.binop(|a, b| self.builder.build_ge(&a, &b)),
                        IntrinsicOperation::LtNumLR => stack.binop(|a, b| self.builder.build_lt(&a, &b)),
                        IntrinsicOperation::LeNumLR => stack.binop(|a, b| self.builder.build_le(&a, &b)),
                        IntrinsicOperation::EqNumLR => stack.binop(|a, b| self.builder.build_eq(&a, &b)),
                        IntrinsicOperation::NeNumLR => stack.binop(|a, b| self.builder.build_ne(&a, &b)),
                        IntrinsicOperation::BitOrNumLR => todo!(),
                        IntrinsicOperation::BitXorNumLR => todo!(),
                        IntrinsicOperation::BitAndNumLR => todo!(),
                        IntrinsicOperation::BitShlNumLR => todo!(),
                        IntrinsicOperation::BitShrNumLR => todo!(),
                        IntrinsicOperation::BitUshrNumLR => todo!(),
                        IntrinsicOperation::LtNumLConstR
                        | IntrinsicOperation::LeNumLConstR
                        | IntrinsicOperation::GtNumLConstR
                        | IntrinsicOperation::GeNumLConstR => {
                            let value = stack.pop();
                            let num = dcx.next_byte() as f64;
                            let rhs = match value.ty_kind() {
                                LLVMTypeKind::LLVMIntegerTypeKind => self.llcx.const_i64(num as i64),
                                LLVMTypeKind::LLVMDoubleTypeKind => self.llcx.const_f64(num),
                                _ => unreachable!(),
                            };
                            let res = match op {
                                IntrinsicOperation::LtNumLConstR => self.builder.build_lt(&value, &rhs),
                                IntrinsicOperation::LeNumLConstR => self.builder.build_le(&value, &rhs),
                                IntrinsicOperation::GtNumLConstR => self.builder.build_gt(&value, &rhs),
                                IntrinsicOperation::GeNumLConstR => self.builder.build_ge(&value, &rhs),
                                _ => unreachable!(),
                            };
                            stack.push(res);
                        }
                        IntrinsicOperation::GtNumLConstR32
                        | IntrinsicOperation::GeNumLConstR32
                        | IntrinsicOperation::LtNumLConstR32
                        | IntrinsicOperation::LeNumLConstR32 => {
                            let value = stack.pop();
                            let num = dcx.next_u32() as f64;
                            let rhs = match value.ty_kind() {
                                LLVMTypeKind::LLVMIntegerTypeKind => self.llcx.const_i64(num as i64),
                                LLVMTypeKind::LLVMDoubleTypeKind => self.llcx.const_f64(num),
                                _ => unreachable!(),
                            };
                            let res = match op {
                                IntrinsicOperation::LtNumLConstR32 => self.builder.build_lt(&value, &rhs),
                                IntrinsicOperation::LeNumLConstR32 => self.builder.build_le(&value, &rhs),
                                IntrinsicOperation::GtNumLConstR32 => self.builder.build_gt(&value, &rhs),
                                IntrinsicOperation::GeNumLConstR32 => self.builder.build_ge(&value, &rhs),
                                _ => unreachable!(),
                            };
                            stack.push(res);
                        }
                        IntrinsicOperation::PostfixIncLocalNum => {
                            let id = dcx.next_byte();
                            let old = self.load_local(id.into());
                            let rhs = match old.ty_kind() {
                                LLVMTypeKind::LLVMIntegerTypeKind => self.llcx.const_i64(1),
                                LLVMTypeKind::LLVMDoubleTypeKind => self.llcx.const_f64(1.0),
                                _ => unreachable!(),
                            };
                            let value = self.builder.build_add(&old, &rhs);
                            self.store_local(id.into(), &value);
                            stack.push(old);
                        }
                        IntrinsicOperation::PostfixDecLocalNum => {
                            let id = dcx.next_byte();
                            let old = self.load_local(id.into());
                            let rhs = match old.ty_kind() {
                                LLVMTypeKind::LLVMIntegerTypeKind => self.llcx.const_i64(1),
                                LLVMTypeKind::LLVMDoubleTypeKind => self.llcx.const_f64(1.0),
                                _ => unreachable!(),
                            };
                            let value = self.builder.build_sub(&old, &rhs);
                            self.store_local(id.into(), &value);
                            stack.push(old);
                        }
                        IntrinsicOperation::PrefixIncLocalNum => {
                            let id = dcx.next_byte();
                            let old = self.load_local(id.into());
                            let rhs = match old.ty_kind() {
                                LLVMTypeKind::LLVMIntegerTypeKind => self.llcx.const_i64(1),
                                LLVMTypeKind::LLVMDoubleTypeKind => self.llcx.const_f64(1.0),
                                _ => unreachable!(),
                            };
                            let value = self.builder.build_add(&old, &rhs);
                            self.store_local(id.into(), &value);
                            stack.push(value);
                        }
                        IntrinsicOperation::PrefixDecLocalNum => {
                            let id = dcx.next_byte();
                            let old = self.load_local(id.into());
                            let rhs = match old.ty_kind() {
                                LLVMTypeKind::LLVMIntegerTypeKind => self.llcx.const_i64(1),
                                LLVMTypeKind::LLVMDoubleTypeKind => self.llcx.const_f64(1.0),
                                _ => unreachable!(),
                            };
                            let value = self.builder.build_sub(&old, &rhs);
                            self.store_local(id.into(), &value);
                            stack.push(value);
                        }
                        _ => return Err(Error::UnsupportedInstruction { instr }),
                    }
                }
                Instruction::Ret => {
                    let _value = stack.pop();
                    let _c = dcx.next_wide();
                }
                _ => return Err(Error::UnsupportedInstruction { instr }),
            }
        }

        // End of basic block was not reached in the block,
        // which means that this basic block was terminated
        // early not by a conditional jump but by another label
        if let Some(succ) = succ {
            let BasicBlockSuccessor::Unconditional(target) = succ else {
                panic!("mismatching basic block successor {:?}", succ);
            };
            let next_bb = &self.llvm_bbs[&target];
            self.builder.build_br(next_bb);
            self.compile_bb(stack, target);
        }

        Ok(())
    }
}

#[derive(Default, Clone)]
pub struct ValueStack(Vec<Value>);

impl ValueStack {
    pub fn binop<F>(&mut self, fun: F)
    where
        F: Fn(Value, Value) -> Value,
    {
        let (a, b) = self.pop2();
        let res = fun(a, b);
        self.0.push(res);
    }

    pub fn push(&mut self, value: Value) {
        self.0.push(value);
    }

    pub fn pop(&mut self) -> Value {
        self.0.pop().unwrap()
    }

    pub fn last(&mut self) -> Value {
        self.0.last().unwrap().clone()
    }

    pub fn pop2(&mut self) -> (Value, Value) {
        let b = self.pop();
        let a = self.pop();
        (b, a)
    }
}

pub fn compile_typed_cfg<Q: CodegenQuery>(
    bytecode: &[u8],
    tcfg: &TypedCfg,
    query: &mut Q,
) -> Result<JitFunction, Error> {
    let mut codegenctxt = CodegenCtxt::new(&tcfg.ty_map, &tcfg.bb_map, bytecode, query);
    codegenctxt.compile_setup_block();
    codegenctxt.compile_bb(ValueStack::default(), 0)?;
    codegenctxt.compile_exit_block();
    codegenctxt.module.verify();
    codegenctxt.module.run_pass_manager(&codegenctxt.pm);
    let func = codegenctxt.ee.compile_fn(codegenctxt.function.name());
    Ok(func)
}
