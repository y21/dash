use std::{any::Any, cell::RefCell, rc::Rc};

use super::{
    instruction::Instruction,
    value::{
        function::{CallState, Constructor, FunctionType, UserFunction},
        Value,
    },
};

#[derive(Debug)]
pub struct NativeResume {
    pub func: Rc<RefCell<Value>>,
    pub args: Vec<Rc<RefCell<Value>>>,
    pub ctor: bool,
    pub receiver: Option<Rc<RefCell<Value>>>,
}

#[derive(Debug)]
pub struct Frame {
    pub func: Rc<RefCell<Value>>,
    pub buffer: Box<[Instruction]>,
    pub ip: usize,
    pub sp: usize,
    pub state: Option<CallState<Box<dyn Any>>>,
    pub resume: Option<NativeResume>,
}

impl Frame {
    pub fn from_buffer<B>(buffer: B, sp: usize) -> Self
    where
        B: Into<Box<[Instruction]>>,
    {
        let buffer = buffer.into();

        let func = UserFunction::new(
            buffer.clone(),
            0,
            FunctionType::Function,
            0,
            Constructor::NoCtor,
        );

        Self {
            func: Value::from(func).into(),
            buffer,
            ip: 0,
            sp,
            state: None,
            resume: None,
        }
    }
}

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

#[derive(Debug)]
pub struct Loop {
    /// Loop condition instruction pointer
    pub condition_ip: usize,
    /// End of loop instruction pointer
    pub end_ip: usize,
}
