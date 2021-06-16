use std::{any::Any, cell::RefCell, rc::Rc};

use super::{
    instruction::Instruction,
    value::{function::CallState, Value},
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

#[derive(Debug)]
pub struct UnwindHandler {
    /// Catch block instruction pointer
    pub catch_ip: usize,
    /// Catch error value
    pub catch_value_sp: Option<usize>,
    /// Finally block instruction pointer
    pub finally_ip: Option<usize>,
}

#[derive(Debug)]
pub struct Loop {
    /// Loop condition instruction pointer
    pub condition_ip: usize,
    /// End of loop instruction pointer
    pub end_ip: usize,
}
