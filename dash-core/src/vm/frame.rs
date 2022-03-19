use std::rc::Rc;

use crate::compiler::constant::Constant;
use crate::compiler::CompileResult;
use crate::gc::handle::Handle;

use super::value::function::user::UserFunction;
use super::value::object::Object;
use super::value::Value;
use super::Vm;

#[derive(Debug, Clone)]
pub struct Frame {
    pub ip: usize,
    pub local_count: usize,
    pub constants: Rc<[Constant]>,
    pub externals: Rc<[Handle<dyn Object>]>,
    pub buffer: Rc<[u8]>,
    pub sp: usize,
}

impl Frame {
    pub fn from_function(uf: &UserFunction, vm: &mut Vm) -> Self {
        let mut externals = Vec::new();

        for external in uf.externals() {
            let val = vm
                .get_local(*external as usize)
                .expect("Referenced local not found");

            let obj = match val {
                Value::Object(o) => o,
                // primitive types need to be put on the heap and GCd
                // TODO: we need to update the locals in this current frame too
                Value::Number(n) => vm.gc.register(n),
                Value::Boolean(b) => vm.gc.register(b),
                Value::String(s) => vm.gc.register(s),
                _ => panic!("Expected object"),
            };

            externals.push(obj);
        }

        Self {
            buffer: uf.buffer().clone(),
            constants: uf.constants().clone(),
            // TODO: Rc allocation not needed in the common case when there is no external variables referenced in the frame
            externals: externals.into(),
            ip: 0,
            sp: 0,
            local_count: uf.locals(),
        }
    }

    pub fn from_compile_result(cr: CompileResult) -> Self {
        // it's impossible to create a Frame if the compile result references external values
        assert!(cr.externals.is_empty());

        Self {
            buffer: cr.instructions.into(),
            constants: cr.cp.into_vec().into(),
            externals: Vec::new().into(),
            ip: 0,
            sp: 0,
            local_count: cr.locals,
        }
    }
}
