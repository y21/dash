use std::cell::Cell;
use std::collections::BTreeMap;
use std::rc::Rc;

use dash_middle::compiler::CompileResult;
use dash_middle::compiler::constant::{Buffer, Function};
use dash_middle::parser::statement::{Asyncness, FunctionKind};
use dash_proc_macro::Trace;

use crate::gc::ObjectId;
use crate::gc::trace::{Trace, TraceCtxt};
use crate::localscope::LocalScope;
use crate::throw;
use crate::value::string::JsString;
use crate::value::{ExternalValue, Unrooted};

use super::value::Value;
use super::value::function::user::UserFunction;

#[derive(Debug, Clone, Copy, Trace)]
pub struct TryBlock {
    pub catch_ip: Option<usize>,
    pub finally_ip: Option<usize>,
    pub frame_ip: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Exports {
    pub default: Option<Unrooted>,
    pub named: Vec<(JsString, Unrooted)>,
}

unsafe impl Trace for Exports {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        self.default.trace(cx);
        for (k, v) in &self.named {
            k.trace(cx);
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
    fn trace(&self, _: &mut TraceCtxt<'_>) {}
}

#[derive(Debug, Clone, Copy)]
pub enum This {
    /// No `this` binding. Evaluates to the global object in non-strict mode, or undefined in strict mode
    Default,
    /// Initial state of `this` in subclass constructors. Throws an error if attempted to evaluate.
    /// Gets changed to `Bound` by the call to super().
    BeforeSuper,
    /// Bound as a value.
    Bound(Value),
}

impl This {
    pub fn to_value(self, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
        match self {
            // TODO: once we have strict mode, eval to undefined
            This::Default => Ok(Value::object(scope.global)),
            This::Bound(value) => Ok(value),
            This::BeforeSuper => throw!(scope, Error, "`super()` must be called before accessing `this`"),
        }
    }
}

unsafe impl Trace for This {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        match *self {
            This::Default => {}
            This::BeforeSuper => {}
            This::Bound(value) => value.trace(cx),
        }
    }
}

#[derive(Debug, Clone, Trace)]
pub struct Frame {
    pub function: Rc<Function>,
    pub ip: usize,
    /// Extra stack space allocated at the start of frame execution, currently only used for local variables
    /// (excluding function parameters, as they are pushed onto the stack in Function::apply)
    pub extra_stack_space: usize,
    /// Contains local variable values from the outer scope
    pub externals: Rc<[ExternalValue]>,
    pub this: This,
    pub sp: usize,
    pub state: FrameState,
    /// When evaluating a `return` op in a try/catch with a finally block,
    /// this will be set to Some(Ok(that_value)).
    /// Exceptions thrown will set it to Err(exception).
    pub delayed_ret: Option<Result<Unrooted, Unrooted>>,

    /// The `arguments` object.
    /// For optimization purposes, this is `None` in frames whose function never references `arguments`,
    /// because there's no reason to construct it in those cases.
    pub arguments: Option<ObjectId>,

    /// Counts the number of backjumps to a particular loop header, to find hot loops
    pub loop_counter: LoopCounterMap,
}

impl Frame {
    pub fn from_function(
        this: This,
        uf: &UserFunction,
        is_constructor_call: bool,
        is_flat_call: bool,
        arguments: Option<ObjectId>,
    ) -> Self {
        let inner = uf.inner();
        Self {
            this,
            function: inner.clone(),
            externals: uf.externals().clone(),
            ip: 0,
            sp: 0,
            delayed_ret: None,
            extra_stack_space: inner.locals - uf.inner().params,
            state: FrameState::Function {
                is_constructor_call,
                is_flat_call,
            },
            loop_counter: LoopCounterMap::default(),
            arguments,
        }
    }

    pub fn from_module(this: This, uf: &UserFunction, arguments: Option<ObjectId>) -> Self {
        let inner = uf.inner();
        Self {
            this,
            function: inner.clone(),
            externals: uf.externals().clone(),
            ip: 0,
            sp: 0,
            delayed_ret: None,
            extra_stack_space: inner.locals - uf.inner().params,
            state: FrameState::Module(Exports::default()),
            loop_counter: LoopCounterMap::default(),
            arguments,
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
            constants: cr.cp,
            externals: Vec::new().into(),
            locals: cr.locals,
            name: None,
            params: 0,
            ty: FunctionKind::Function(Asyncness::No),
            rest_local: None,
            source: cr.source,
            debug_symbols: cr.debug_symbols,
            references_arguments: false,
            has_extends_clause: false,
        };

        Self {
            this: This::Default,
            function: Rc::new(fun),
            externals: Vec::new().into(),
            ip: 0,
            sp: 0,
            delayed_ret: None,
            extra_stack_space: cr.locals, /* - 0 params */
            state: FrameState::Function {
                is_constructor_call: false,
                is_flat_call: false,
            },
            loop_counter: LoopCounterMap::default(),
            // Root function never has arguments
            arguments: None,
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
