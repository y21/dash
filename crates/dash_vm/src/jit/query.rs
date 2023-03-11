use dash_llvm_jit_backend::codegen::CodegenQuery;
use dash_llvm_jit_backend::codegen::JitConstant;
use dash_llvm_jit_backend::passes::bb_generation::BBGenerationQuery;
use dash_llvm_jit_backend::passes::bb_generation::ConditionalBranchAction;
use dash_llvm_jit_backend::passes::type_infer::Type;
use dash_llvm_jit_backend::passes::type_infer::TypeInferQuery;
use dash_llvm_jit_backend::typed_cfg::TypedCfgQuery;
use dash_llvm_jit_backend::Trace;
use dash_middle::compiler::constant::Constant;
use dash_middle::util::is_integer;

use crate::value::primitive::Number;
use crate::value::Value;
use crate::Vm;

pub struct QueryProvider<'a> {
    pub vm: &'a Vm,
    pub trace: &'a Trace,
}

impl<'a> TypedCfgQuery for QueryProvider<'a> {}

impl<'a> BBGenerationQuery for QueryProvider<'a> {
    fn conditional_branch_at(&self, ip: usize) -> ConditionalBranchAction {
        self.trace.get_conditional_jump(self.trace.start() + ip + 1)
    }
}

impl<'a> TypeInferQuery for QueryProvider<'a> {
    fn type_of_constant(&self, index: u16) -> Type {
        let constant = &self.vm.frames.last().unwrap().function.constants[usize::from(index)];
        match constant {
            Constant::Boolean(..) => Type::Boolean,
            Constant::Number(n) => {
                if is_integer(*n) {
                    Type::I64
                } else {
                    Type::F64
                }
            }
            _ => panic!("invalid jit type"),
        }
    }
    fn type_of_local(&self, index: u16) -> Type {
        match self.vm.get_local(index.into()).unwrap() {
            Value::Boolean(..) => Type::Boolean,
            Value::Number(Number(n)) => {
                if is_integer(n) {
                    Type::I64
                } else {
                    Type::F64
                }
            }
            _ => panic!("invalid jit type"),
        }
    }
}

impl<'a> CodegenQuery for QueryProvider<'a> {
    fn get_constant(&self, id: u16) -> JitConstant {
        let constant = &self.vm.frames.last().unwrap().function.constants[usize::from(id)];
        match constant {
            Constant::Boolean(b) => JitConstant::Boolean(*b),
            Constant::Number(n) => {
                if is_integer(*n) {
                    JitConstant::I64(*n as i64)
                } else {
                    JitConstant::F64(*n)
                }
            }
            _ => todo!(),
        }
    }
}
