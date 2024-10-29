use std::any::Any;
use std::cell::RefCell;

use dash_proc_macro::Trace;

use crate::frame::TryBlock;
use crate::gc::trace::{Trace, TraceCtxt};
use crate::gc::ObjectId;
use crate::localscope::LocalScope;
use crate::value::arguments::Arguments;
use crate::value::object::{NamedObject, Object};
use crate::value::{Typeof, Unrooted, Value};
use crate::{delegate, Vm};

use super::extend_stack_from_args;
use super::user::UserFunction;

#[derive(Debug, Trace)]
pub struct GeneratorFunction {
    function: UserFunction,
}

impl GeneratorFunction {
    pub fn new(function: UserFunction) -> Self {
        Self { function }
    }

    pub fn function(&self) -> &UserFunction {
        &self.function
    }

    pub(crate) fn handle_function_call(
        &self,
        scope: &mut LocalScope,
        callee: ObjectId,
        _this: Value,
        args: Vec<Value>,
        _is_constructor_call: bool,
    ) -> Result<Value, Unrooted> {
        let mut arguments = None;
        if self.function.inner().references_arguments {
            let args = Arguments::new(scope, args.iter().cloned());
            let args = scope.register(args);
            arguments = Some(args);
        }

        // Handle edge cases such as provided_args != expected_args
        // by delegating to the usual arg handling logic that occurs with normal user functions
        let args = {
            // TODO: if this turns out slow
            // we can avoid extending into the stack just to drain + collect again
            let inner = self.function.inner();
            let sp = scope.stack.len();
            extend_stack_from_args(args, inner.params, scope, inner.rest_local.is_some());
            scope.stack.drain(sp..).collect::<Vec<_>>()
        };

        let iter = GeneratorIterator::new(callee, scope, args, arguments, Vec::new());
        Ok(Value::object(scope.register(iter)))
    }
}

#[derive(Debug, Clone)]
pub enum GeneratorState {
    Finished,
    Running {
        ip: usize,
        stack: Vec<Value>,
        try_blocks: Vec<TryBlock>,
        arguments: Option<ObjectId>,
    },
}

impl GeneratorState {
    pub fn did_run(&self) -> bool {
        match self {
            Self::Finished => true,
            Self::Running { ip, .. } => *ip != 0,
        }
    }
}

unsafe impl Trace for GeneratorState {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        match self {
            GeneratorState::Finished => {}
            GeneratorState::Running {
                ip: _,
                stack,
                arguments,
                try_blocks,
            } => {
                stack.trace(cx);
                arguments.trace(cx);
                try_blocks.trace(cx);
            }
        }
    }
}

#[derive(Debug, Trace)]
pub struct GeneratorIterator {
    function: ObjectId,
    obj: NamedObject,
    state: RefCell<GeneratorState>,
}

impl GeneratorIterator {
    pub fn new(
        function: ObjectId,
        vm: &Vm,
        stack: Vec<Value>,
        arguments: Option<ObjectId>,
        try_blocks: Vec<TryBlock>,
    ) -> Self {
        Self {
            function,
            obj: NamedObject::with_prototype_and_constructor(vm.statics.generator_iterator_prototype, function),
            state: RefCell::new(GeneratorState::Running {
                ip: 0,
                stack,
                arguments,
                try_blocks,
            }),
        }
    }

    pub fn empty(function: ObjectId) -> Self {
        Self {
            function,
            obj: NamedObject::null(),
            state: RefCell::new(GeneratorState::Finished),
        }
    }

    pub fn state(&self) -> &RefCell<GeneratorState> {
        &self.state
    }

    pub fn function(&self) -> ObjectId {
        self.function
    }

    pub fn did_run(&self) -> bool {
        self.state.borrow().did_run()
    }
}

impl Object for GeneratorIterator {
    delegate!(
        obj,
        get_own_property_descriptor,
        get_property,
        get_property_descriptor,
        set_property,
        delete_property,
        set_prototype,
        get_prototype,
        own_keys
    );

    fn apply(
        &self,
        scope: &mut LocalScope,
        callee: ObjectId,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        self.obj.apply(scope, callee, this, args)
    }

    fn as_any(&self, _: &Vm) -> &dyn Any {
        self
    }

    fn type_of(&self, _: &Vm) -> Typeof {
        Typeof::Object
    }
}
