use std::collections::BTreeMap;
use std::rc::Rc;

use dash_middle::compiler::constant::Function;
use dash_middle::compiler::CompileResult;
use dash_middle::parser::statement::FunctionKind;

use crate::gc::handle::Handle;
use crate::gc::trace::Trace;

use super::value::function::user::UserFunction;
use super::value::object::Object;
use super::value::Value;

#[derive(Debug, Clone)]
pub struct TryBlock {
    pub catch_ip: usize,
    pub frame_ip: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Exports {
    pub default: Option<Value>,
    pub named: Vec<(Rc<str>, Value)>,
}

#[derive(Debug, Clone)]
pub enum FrameState {
    /// Regular function
    Function {
        /// Whether the currently executing function is a constructor call
        is_constructor_call: bool,
    },
    /// Top level frame of a module
    Module(Exports),
}

#[derive(Debug, Clone, Default)]
pub struct LoopCounter(u32);

impl LoopCounter {
    pub fn inc(&mut self) {
        self.0 += 1;
    }

    pub fn is_hot(&self) -> bool {
        self.0 > 5
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub function: Rc<Function>,
    pub ip: usize,
    pub reserved_stack_size: usize,
    pub externals: Rc<[Handle<dyn Object>]>,
    pub this: Option<Value>,
    pub sp: usize,
    pub state: FrameState,

    /// Counts the number of backjumps to a particular loop header, to find hot loops
    pub loop_counter: BTreeMap<usize, LoopCounter>,
}

unsafe impl Trace for Frame {
    fn trace(&self) {
        self.externals.trace();
    }
}

impl Frame {
    pub fn from_function(this: Option<Value>, uf: &UserFunction, is_constructor_call: bool) -> Self {
        let inner = uf.inner();
        Self {
            this,
            function: inner.clone(),
            externals: uf.externals().clone(),
            ip: 0,
            sp: 0,
            reserved_stack_size: inner.locals, // TODO: do we need to add the param count to this number?
            state: FrameState::Function { is_constructor_call },
            loop_counter: BTreeMap::new(),
        }
    }

    pub fn from_module(this: Option<Value>, uf: &UserFunction) -> Self {
        let inner = uf.inner();
        Self {
            this,
            function: inner.clone(),
            externals: uf.externals().clone(),
            ip: 0,
            sp: 0,
            reserved_stack_size: inner.locals,
            state: FrameState::Module(Exports::default()),
            loop_counter: BTreeMap::new(),
        }
    }

    pub fn is_module(&self) -> bool {
        matches!(self.state, FrameState::Module(_))
    }

    pub fn from_compile_result(cr: CompileResult) -> Self {
        // it's [logically] impossible to create a Frame if the compile result references external values
        // there's likely a bug somewhere if this assertion fails and will be *really* confusing if this invariant doesn't get caught
        debug_assert!(cr.externals.is_empty());

        let fun = Function {
            buffer: cr.instructions.into(),
            constants: cr.cp.into_vec().into(),
            externals: Vec::new().into(),
            locals: cr.locals,
            name: None,
            params: 0,
            ty: FunctionKind::Function,
        };

        Self {
            this: None,
            function: Rc::new(fun),
            externals: Vec::new().into(),
            ip: 0,
            sp: 0,
            reserved_stack_size: cr.locals,
            state: FrameState::Function {
                is_constructor_call: false,
            },
            loop_counter: BTreeMap::new(),
        }
    }

    pub fn set_reserved_stack_size(&mut self, size: usize) {
        self.reserved_stack_size = size;
    }

    pub fn set_ip(&mut self, ip: usize) {
        self.ip = ip;
    }

    pub fn set_sp(&mut self, sp: usize) {
        self.sp = sp;
    }
}
