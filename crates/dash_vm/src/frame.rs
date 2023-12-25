use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashSet};
use std::rc::Rc;

use dash_middle::compiler::constant::{Buffer, Function};
use dash_middle::compiler::CompileResult;
use dash_middle::parser::statement::FunctionKind;
use dash_proc_macro::Trace;

use crate::gc::handle::Handle;
use crate::gc::trace::{Trace, TraceCtxt};
use crate::value::{ExternalValue, Unrooted};

use super::value::function::user::UserFunction;
use super::value::Value;

#[derive(Debug, Clone)]
pub struct TryBlock {
    pub catch_ip: usize,
    pub frame_ip: usize,
}

// TODO: these should be unrooted
#[derive(Debug, Clone, Default)]
pub struct Exports {
    pub default: Option<Unrooted>,
    pub named: Vec<(Rc<str>, Unrooted)>,
}

unsafe impl Trace for Exports {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        self.default.trace(cx);
        for (_, v) in &self.named {
            v.trace(cx);
        }
    }
}

#[derive(Debug, Clone)]
pub enum FrameState {
    /// Regular function
    Function {
        /// Whether the currently executing function is a constructor call
        is_constructor_call: bool,
        /// Whether this frame is a flat function call
        is_flat_call: bool,
    },
    /// Top level frame of a module
    Module(Exports),
}

unsafe impl Trace for FrameState {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        match self {
            Self::Module(exports) => exports.trace(cx),
            Self::Function {
                is_constructor_call: _,
                is_flat_call: _,
            } => {}
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LoopCounter(u32);

impl LoopCounter {
    pub fn inc(&mut self) {
        self.0 += 1;
    }

    pub fn is_hot(&self) -> bool {
        self.0 > 100
    }
}

#[derive(Debug, Clone, Default)]
pub struct LoopCounterMap(BTreeMap<usize, LoopCounter>);

impl LoopCounterMap {
    pub fn get_or_insert(&mut self, id: usize) -> &mut LoopCounter {
        self.0.entry(id).or_default()
    }
}

unsafe impl Trace for LoopCounterMap {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {}
}

#[derive(Debug, Clone, Trace)]
pub struct Frame {
    pub function: Rc<Function>,
    pub ip: usize,
    /// Extra stack space allocated at the start of frame execution, currently only used for local variables
    /// (excluding function parameters, as they are pushed onto the stack in Function::apply)
    pub extra_stack_space: usize,
    /// Contains local variable values from the outer scope
    pub externals: Rc<[Handle<ExternalValue>]>,
    pub this: Option<Value>,
    pub sp: usize,
    pub state: FrameState,

    /// Counts the number of backjumps to a particular loop header, to find hot loops
    pub loop_counter: LoopCounterMap,
}

impl Frame {
    pub fn from_function(
        this: Option<Value>,
        uf: &UserFunction,
        is_constructor_call: bool,
        is_flat_call: bool,
    ) -> Self {
        let inner = uf.inner();
        Self {
            this,
            function: inner.clone(),
            externals: uf.externals().clone(),
            ip: 0,
            sp: 0,
            extra_stack_space: inner.locals - uf.inner().params,
            state: FrameState::Function {
                is_constructor_call,
                is_flat_call,
            },
            loop_counter: LoopCounterMap::default(),
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
            extra_stack_space: inner.locals - uf.inner().params,
            state: FrameState::Module(Exports::default()),
            loop_counter: LoopCounterMap::default(),
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
            buffer: Buffer(Cell::new(cr.instructions.into())),
            constants: cr.cp.into_vec().into(),
            externals: Vec::new().into(),
            locals: cr.locals,
            name: None,
            params: 0,
            ty: FunctionKind::Function,
            r#async: false,
            rest_local: None,
            poison_ips: RefCell::new(HashSet::new()),
            source: cr.source,
            debug_symbols: cr.debug_symbols,
        };

        Self {
            this: None,
            function: Rc::new(fun),
            externals: Vec::new().into(),
            ip: 0,
            sp: 0,
            extra_stack_space: cr.locals, /* - 0 params */
            state: FrameState::Function {
                is_constructor_call: false,
                is_flat_call: false,
            },
            loop_counter: LoopCounterMap::default(),
        }
    }

    pub fn set_extra_stack_space(&mut self, size: usize) {
        self.extra_stack_space = size;
    }

    pub fn set_ip(&mut self, ip: usize) {
        self.ip = ip;
    }

    pub fn set_sp(&mut self, sp: usize) {
        self.sp = sp;
    }
}
