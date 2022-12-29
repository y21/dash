use dash_llvm_jit_backend::function::CompileQuery;
use dash_llvm_jit_backend::function::JITConstant;
use dash_llvm_jit_backend::passes::infer::InferQueryProvider;
use dash_llvm_jit_backend::passes::infer::Type;
use dash_llvm_jit_backend::Trace;
use dash_middle::compiler::constant::Constant;

use crate::value::Value;
use crate::Vm;

pub struct QueryProvider<'a> {
    vm: &'a Vm,
    trace: &'a Trace,
}

impl<'a> QueryProvider<'a> {
    pub fn new(vm: &'a Vm, trace: &'a Trace) -> Self {
        Self { vm, trace }
    }
}

impl<'a> InferQueryProvider for QueryProvider<'a> {
    fn type_of_constant(&self, index: u16) -> Option<Type> {
        let constant = &self.vm.frames.last().unwrap().function.constants[usize::from(index)];
        match constant {
            Constant::Boolean(..) => Some(Type::Boolean),
            Constant::Number(..) => Some(Type::F64),
            _ => None,
        }
    }
    fn type_of_local(&self, index: u16) -> Option<Type> {
        match self.vm.get_local(index.into()).unwrap() {
            Value::Boolean(..) => Some(Type::Boolean),
            Value::Number(..) => Some(Type::F64),
            _ => None,
        }
    }
    fn did_take_nth_branch(&self, nth: usize) -> bool {
        self.trace.get_conditional_jump(nth)
    }
}

impl<'a> CompileQuery for QueryProvider<'a> {
    fn get_constant(&self, id: u16) -> JITConstant {
        let constant = &self.vm.frames.last().unwrap().function.constants[usize::from(id)];
        match constant {
            Constant::Boolean(b) => JITConstant::Boolean(*b),
            Constant::Number(n) => JITConstant::F64(*n), // TODO: I64 may be ok
            _ => todo!(),
        }
    }
}
