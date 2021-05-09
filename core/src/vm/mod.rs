pub mod environment;
pub mod frame;
pub mod instruction;
pub mod stack;
pub mod value;

use std::{cell::RefCell, rc::Rc};

use instruction::{Instruction, Opcode};
use value::{JsValue, Value};

use crate::vm::value::{Compare, FunctionType, NativeFunction, Object};

use self::{
    environment::Environment,
    frame::Frame,
    stack::Stack,
    value::{FunctionKind, UserFunction},
};

#[derive(Debug)]
pub enum VMError {}

macro_rules! binary_op {
    ($self:ident, $op:tt) => {
        let (b, a) = (
            $self.read_number(),
            $self.read_number()
        );

        $self.stack.push(Rc::new(RefCell::new(Value::Number(a $op b))));
    }
}

pub struct VM {
    /// Call stack
    pub(crate) frames: Stack<Frame, 256>,
    /// Stack
    pub(crate) stack: Stack<Rc<RefCell<Value>>, 512>,
    /// Global namespace
    pub(crate) global: Environment,
}

impl VM {
    pub fn new(func: UserFunction) -> Self {
        let mut frames = Stack::new();
        frames.push(Frame {
            buffer: func.buffer,
            ip: 0,
            sp: 0,
        });

        let mut vm = Self {
            frames,
            stack: Stack::new(),
            global: Environment::new(),
        };
        vm.prepare_stdlib();
        vm
    }

    fn frame(&self) -> &Frame {
        self.frames.get()
    }

    fn frame_mut(&mut self) -> &mut Frame {
        self.frames.get_mut()
    }

    fn ip(&self) -> usize {
        self.frame().ip
    }

    fn buffer(&self) -> &[Instruction] {
        &self.frame().buffer
    }

    fn is_eof(&self) -> bool {
        self.ip() >= self.buffer().len()
    }

    fn read_constant(&mut self) -> Option<Value> {
        if self.is_eof() {
            return None;
        }

        self.frame_mut().ip += 1;

        Some(self.buffer()[self.ip() - 1].clone().into_operand())
    }

    fn read_number(&mut self) -> f64 {
        self.stack.pop().borrow().as_number()
    }

    fn pop_owned(&mut self) -> Option<Value> {
        Value::try_into_inner(self.stack.pop())
    }

    fn prepare_stdlib(&mut self) {
        self.global.set_var(
            "isNaN",
            Rc::new(RefCell::new(Value::Object(Box::new(Object::Function(
                FunctionKind::Native(NativeFunction("isNaN", |value| {
                    let value = match value.first() {
                        Some(v) => v,
                        None => return Rc::new(RefCell::new(Value::Bool(true))),
                    };

                    let value = value.borrow().as_number();

                    Rc::new(RefCell::new(Value::Bool(value.is_nan())))
                })),
            ))))),
        )
    }

    pub fn interpret(&mut self) -> Result<(), VMError> {
        while !self.is_eof() {
            let instruction = self.buffer()[self.ip()].as_op();

            self.frame_mut().ip += 1;

            match instruction {
                Opcode::Eof => return Ok(()),
                Opcode::Constant => {
                    let constant = self.read_constant().unwrap();

                    self.stack.push(Rc::new(RefCell::new(constant)));
                }
                Opcode::Negate => {
                    let maybe_number = self.read_number();

                    self.stack
                        .push(Rc::new(RefCell::new(Value::Number(-maybe_number))));
                }
                Opcode::Add => {
                    binary_op!(self, +);
                }
                Opcode::Sub => {
                    binary_op!(self, -);
                }
                Opcode::Mul => {
                    binary_op!(self, *);
                }
                Opcode::Div => {
                    binary_op!(self, /);
                }
                Opcode::Rem => {
                    binary_op!(self, %);
                }
                Opcode::SetGlobal => {
                    let name = self.pop_owned().unwrap().into_ident().unwrap();
                    let value = self.stack.pop();

                    self.global.set_var(name, value);
                }
                Opcode::GetGlobal => {
                    let name = self.pop_owned().unwrap().into_ident().unwrap();

                    // TODO: handle case where var is not defined
                    let value = self.global.get_var(&name).unwrap();

                    self.stack.push(value);
                }
                Opcode::SetLocal => {
                    let stack_idx = self.read_number() as usize;
                    let value = self.stack.pop();
                    self.stack.set(stack_idx, value);
                }
                Opcode::GetLocal => {
                    let stack_idx = self.read_number() as usize;

                    self.stack
                        .push(self.stack.peek_relative(self.frame().sp, stack_idx).clone());
                }
                Opcode::ShortJmpIfFalse => {
                    let instruction_count = self.pop_owned().unwrap().as_number() as usize;

                    let condition_cell = self.stack.pop();
                    let condition = condition_cell.borrow().is_truthy();

                    if !condition {
                        self.frame_mut().ip += instruction_count;
                    }
                }
                Opcode::ShortJmpIfTrue => {
                    let instruction_count = self.pop_owned().unwrap().as_number() as usize;

                    let condition_cell = self.stack.get();
                    let condition = condition_cell.borrow().is_truthy();

                    if condition {
                        self.frame_mut().ip += instruction_count;
                    }
                }
                Opcode::ShortJmp => {
                    let instruction_count = self.pop_owned().unwrap().as_number() as usize;
                    self.frame_mut().ip += instruction_count;
                }
                Opcode::BackJmp => {
                    let instruction_count = self.pop_owned().unwrap().as_number() as usize;
                    self.frame_mut().ip -= instruction_count;
                }
                Opcode::Pop => {
                    self.stack.pop();
                }
                Opcode::AdditionAssignment => {
                    let target_cell = self.stack.pop();

                    let value_cell = self.stack.pop();

                    let value = value_cell.borrow();

                    target_cell.borrow_mut().add_assign(&*value);

                    self.stack.push(target_cell);
                }
                Opcode::SubtractionAssignment => {
                    let target_cell = self.stack.pop();

                    let value_cell = self.stack.pop();

                    let value = value_cell.borrow();

                    target_cell.borrow_mut().sub_assign(&*value);

                    self.stack.push(target_cell);
                }
                Opcode::FunctionCall => {
                    let param_count = self.read_number() as usize;
                    let mut params = Vec::new();
                    for _ in 0..param_count {
                        params.push(self.stack.pop());
                    }

                    let func_cell = self.stack.pop();
                    let func_cell_ref = func_cell.borrow();
                    let func = match func_cell_ref.as_function().unwrap() {
                        FunctionKind::Native(f) => {
                            let result = (f.1)(params);
                            self.stack.push(result);
                            continue;
                        }
                        FunctionKind::User(u) => u,
                    };

                    let current_sp = self.stack.get_stack_pointer();
                    self.frame_mut().sp = current_sp;

                    let frame = Frame {
                        buffer: func.buffer.clone(),
                        ip: 0,
                        sp: current_sp,
                    };
                    self.frames.push(frame);
                    for param in params.into_iter().rev() {
                        self.stack.push(param);
                    }
                }
                Opcode::Return => {
                    // Restore VM state to where we were before the function call happened
                    let ret = self.stack.pop();
                    self.frames.pop();
                    if self.frames.get_stack_pointer() == 0 {
                        return Ok(());
                    }

                    self.stack.set_stack_pointer(self.frame().sp);
                    self.stack.push(ret);
                }
                Opcode::Print => {
                    let value_cell = self.stack.pop();
                    let value = value_cell.borrow();

                    println!("{}", value.to_string());
                }
                Opcode::Less => {
                    let rhs_cell = self.stack.pop();
                    let rhs = rhs_cell.borrow();
                    let lhs_cell = self.stack.pop();
                    let lhs = lhs_cell.borrow();

                    let is_less = matches!(lhs.compare(&rhs), Some(Compare::Less));
                    self.stack.push(Rc::new(RefCell::new(Value::Bool(is_less))));
                }
                Opcode::LessEqual => {
                    let rhs_cell = self.stack.pop();
                    let rhs = rhs_cell.borrow();
                    let lhs_cell = self.stack.pop();
                    let lhs = lhs_cell.borrow();

                    let is_less_eq = matches!(
                        lhs.compare(&rhs),
                        Some(Compare::Less) | Some(Compare::Equal)
                    );
                    self.stack
                        .push(Rc::new(RefCell::new(Value::Bool(is_less_eq))));
                }
                Opcode::Greater => {
                    let rhs_cell = self.stack.pop();
                    let rhs = rhs_cell.borrow();
                    let lhs_cell = self.stack.pop();
                    let lhs = lhs_cell.borrow();

                    let is_greater = matches!(lhs.compare(&rhs), Some(Compare::Greater));
                    self.stack
                        .push(Rc::new(RefCell::new(Value::Bool(is_greater))));
                }
                Opcode::GreaterEqual => {
                    let rhs_cell = self.stack.pop();
                    let rhs = rhs_cell.borrow();
                    let lhs_cell = self.stack.pop();
                    let lhs = lhs_cell.borrow();

                    let is_greater_eq = matches!(
                        lhs.compare(&rhs),
                        Some(Compare::Greater) | Some(Compare::Equal)
                    );
                    self.stack
                        .push(Rc::new(RefCell::new(Value::Bool(is_greater_eq))));
                }
                _ => unreachable!(),
            };
        }

        Ok(())
    }
}
