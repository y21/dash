use std::any::Any;
use std::cell::RefCell;

use dash_proc_macro::Trace;

use crate::delegate;
use crate::gc::handle::Handle;
use crate::gc::trace::Trace;
use crate::local::LocalScope;
use crate::throw;
use crate::value::object::NamedObject;
use crate::value::object::Object;
use crate::value::Typeof;
use crate::value::Value;
use crate::Vm;

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
        callee: Handle<dyn Object>,
        _this: Value,
        args: Vec<Value>,
        _is_constructor_call: bool,
    ) -> Result<Value, Value> {
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

        let iter = GeneratorIterator::new(callee, scope, args);
        Ok(scope.register(iter).into())
    }
}

#[derive(Debug, Clone)]
pub enum GeneratorState {
    Finished,
    Running { ip: usize, stack: Vec<Value> },
}

impl GeneratorState {
    pub fn did_run(&self) -> bool {
        match self {
            Self::Finished => true,
            Self::Running { ip, .. } => *ip != 0,
        }
    }
}

impl Default for GeneratorState {
    fn default() -> Self {
        Self::Running {
            ip: 0,
            stack: Vec::new(),
        }
    }
}

unsafe impl Trace for GeneratorState {
    fn trace(&self) {
        if let GeneratorState::Running { ref stack, .. } = self {
            stack.trace();
        }
    }
}

#[derive(Debug, Trace)]
pub struct GeneratorIterator {
    function: Handle<dyn Object>,
    obj: NamedObject,
    state: RefCell<GeneratorState>,
}

impl GeneratorIterator {
    pub fn new(function: Handle<dyn Object>, vm: &Vm, stack: Vec<Value>) -> Self {
        let proto = vm.statics.generator_iterator_prototype.clone();
        let ctor = function.clone();

        Self {
            function,
            obj: NamedObject::with_prototype_and_constructor(proto, ctor),
            state: RefCell::new(GeneratorState::Running { ip: 0, stack }),
        }
    }

    pub fn empty(function: Handle<dyn Object>) -> Self {
        Self {
            function,
            obj: NamedObject::null(),
            state: RefCell::new(GeneratorState::default()),
        }
    }

    pub fn state(&self) -> &RefCell<GeneratorState> {
        &self.state
    }

    pub fn function(&self) -> &Handle<dyn Object> {
        &self.function
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
        callee: Handle<dyn Object>,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        self.obj.apply(scope, callee, this, args)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn type_of(&self) -> Typeof {
        Typeof::Object
    }
}

pub fn as_generator<'a>(scope: &mut LocalScope, value: &'a Value) -> Result<&'a GeneratorIterator, Value> {
    let generator = match value.downcast_ref::<GeneratorIterator>() {
        Some(it) => it,
        None => throw!(scope, TypeError, "Incompatible receiver"),
    };

    Ok(generator)
}
