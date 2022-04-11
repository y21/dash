use std::rc::Rc;

use crate::compiler::constant::Constant;
use crate::compiler::CompileResult;
use crate::gc::handle::Handle;
use crate::gc::trace::Trace;

use super::value::function::user::UserFunction;
use super::value::object::Object;
use super::Vm;

#[derive(Debug, Clone)]
pub struct TryBlock {
    pub catch_ip: usize,
    pub frame_ip: usize,
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub ip: usize,
    pub local_count: usize,
    pub constants: Rc<[Constant]>,
    pub externals: Rc<[Handle<dyn Object>]>,
    pub buffer: Rc<[u8]>,
    pub sp: usize,
}

unsafe impl Trace for Frame {
    fn trace(&self) {
        self.externals.trace();
    }
}

impl Frame {
    pub fn from_function(uf: &UserFunction, vm: &mut Vm) -> Self {
        Self {
            buffer: uf.buffer().clone(),
            constants: uf.constants().clone(),
            externals: uf.externals().clone(),
            ip: 0,
            sp: 0,
            local_count: uf.locals(),
        }
    }

    pub fn from_compile_result(cr: CompileResult) -> Self {
        // it's [logically] impossible to create a Frame if the compile result references external values
        // there's likely a bug somewhere if this assertion fails and will be *really* confusing if this invariant doesn't get caught
        debug_assert!(cr.externals.is_empty());

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
