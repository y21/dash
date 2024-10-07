use dash_llvm_jit_backend::codegen::CodegenQuery;
use dash_llvm_jit_backend::Trace;
use dash_middle::compiler::constant::{BooleanConstant, NumberConstant};
use dash_middle::util::is_integer;
use dash_typed_cfg::passes::bb_generation::{BBGenerationQuery, ConditionalBranchAction};
use dash_typed_cfg::passes::type_infer::{Type, TypeInferQuery};
use dash_typed_cfg::TypedCfgQuery;

use crate::value::primitive::Number;
use crate::value::Value;
use crate::Vm;

pub struct QueryProvider<'a> {
    pub vm: &'a Vm,
    pub trace: &'a Trace,
}

impl<'a> TypedCfgQuery for QueryProvider<'a> {}

impl<'a> BBGenerationQuery for QueryProvider<'a> {
    fn conditional_branch_at(&self, ip: usize) -> Option<ConditionalBranchAction> {
        self.trace.get_conditional_jump(self.trace.start() + ip + 1)
    }
}

impl<'a> TypeInferQuery for QueryProvider<'a> {
    fn number_constant(&self, id: NumberConstant) -> f64 {
        self.vm.active_frame().function.constants.numbers[id]
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
    fn boolean_constant(&self, id: BooleanConstant) -> bool {
        self.vm.active_frame().function.constants.booleans[id]
    }

    fn number_constant(&self, id: NumberConstant) -> f64 {
        self.vm.active_frame().function.constants.numbers[id]
    }
}
