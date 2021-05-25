use std::{cell::RefCell, rc::Rc};

use super::{instruction::Instruction, value::Value};

#[derive(Debug)]
pub struct Frame {
    pub func: Rc<RefCell<Value>>,
    pub buffer: Box<[Instruction]>,
    pub ip: usize,
    pub sp: usize,
}

#[derive(Debug)]
pub struct UnwindHandler {
    pub catch_ip: usize,
    pub catch_value_sp: Option<usize>,
    pub finally_ip: Option<usize>,
}
