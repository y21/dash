use std::{cell::RefCell, rc::Rc};

use super::{
    instruction::Instruction,
    value::{UserFunction, Value},
};

#[derive(Debug)]
pub struct Frame {
    pub func: Rc<RefCell<Value>>,
    pub buffer: Box<[Instruction]>,
    pub ip: usize,
    pub sp: usize,
}
