use std::any::Any;

use crate::gc::Handle;

use super::{
    instruction::Instruction,
    value::{
        function::{CallState, Constructor, FunctionType, UserFunction},
        Value,
    },
    VM,
};

/// Represents a function that needs to be resumed at a later point
#[derive(Debug)]
pub struct NativeResume {
    /// The function that needs to be called
    pub func: Handle<Value>,
    /// Arguments that were originally passed to the function when called initially
    pub args: Vec<Handle<Value>>,
    /// Whether this is a constructor call
    pub ctor: bool,
    /// The receiver of this function
    pub receiver: Option<Handle<Value>>,
}

/// An execution frame
#[derive(Debug)]
pub struct Frame {
    /// JavaScript function
    pub func: Handle<Value>,
    /// This frames bytecode
    pub buffer: Box<[Instruction]>,
    /// Instruction pointer
    pub ip: usize,
    /// Stack pointer
    pub sp: usize,
    /// State that is associated to a native function
    /// that called this user function
    pub state: Option<CallState<Box<dyn Any>>>,
    /// Native resume
    pub resume: Option<NativeResume>,
}

impl Frame {
    /// Creates a frame from bytecode and a stack pointer
    pub fn from_buffer<B>(buffer: B, vm: &VM) -> Self
    where
        B: Into<Box<[Instruction]>>,
    {
        let sp = vm.stack.get_stack_pointer();
        let buffer = buffer.into();

        let func = UserFunction::new(
            buffer.clone(),
            0,
            FunctionType::Function,
            0,
            Constructor::NoCtor,
        );

        Self {
            func: Value::from(func).into_handle(vm),
            buffer,
            ip: 0,
            sp,
            state: None,
            resume: None,
        }
    }

    pub(crate) fn mark_visited(&self) {
        Value::mark(&self.func);

        if let Some(resume) = &self.resume {
            Value::mark(&resume.func);

            for arg in &resume.args {
                Value::mark(arg);
            }

            if let Some(receiver) = &resume.receiver {
                Value::mark(receiver);
            }
        }
    }
}

/// An unwind handler, also known as a try catch block
#[derive(Debug)]
pub struct UnwindHandler {
    /// Catch block instruction pointer
    pub catch_ip: usize,
    /// Catch error value
    pub catch_value_sp: Option<usize>,
    /// Finally block instruction pointer
    pub finally_ip: Option<usize>,
    /// Pointer to frame where this try/catch block lives
    pub frame_pointer: usize,
}

/// A loop
#[derive(Debug)]
pub struct Loop {
    /// Loop condition instruction pointer
    pub condition_ip: usize,
    /// End of loop instruction pointer
    pub end_ip: usize,
}
