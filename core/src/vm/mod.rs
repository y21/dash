pub mod conversions;
pub mod environment;
pub mod frame;
pub mod instruction;
pub mod stack;
pub mod statics;
pub mod upvalue;
pub mod value;

use std::{any::Any, cell::RefCell, rc::Rc};

use instruction::{Instruction, Opcode};
use value::Value;

use crate::vm::{
    upvalue::Upvalue,
    value::{CallContext, Closure, Compare, ValueKind},
};

use self::{
    environment::Environment,
    frame::Frame,
    stack::Stack,
    statics::Statics,
    value::{AnyObject, FunctionKind, Object, UserFunction},
};

#[derive(Debug)]
pub enum VMError {}

macro_rules! binary_op {
    ($self:ident, $op:tt) => {
        let (b, a) = (
            $self.read_number(),
            $self.read_number()
        );

        $self.stack.push(Rc::new(RefCell::new(Value::new(ValueKind::Number(a $op b)))));
    }
}

pub struct VM {
    /// Call stack
    pub(crate) frames: Stack<Frame, 256>,
    /// Stack
    pub(crate) stack: Stack<Rc<RefCell<Value>>, 512>,
    /// Global namespace
    pub(crate) global: Environment,
    /// Static values created once when the VM is initialized
    pub(crate) statics: Statics,
    /// Embedder specific slot data
    pub(crate) slot: Option<Box<dyn Any>>,
}

impl VM {
    pub fn new(func: UserFunction) -> Self {
        let mut frames = Stack::new();
        frames.push(Frame {
            buffer: func.buffer.clone(),
            func: Rc::new(RefCell::new(Value::new(ValueKind::Object(Box::new(
                Object::Function(FunctionKind::Closure(Closure::new(func))),
            ))))),
            ip: 0,
            sp: 0,
        });

        let mut vm = Self {
            frames,
            stack: Stack::new(),
            global: Environment::new(),
            statics: Statics::new(),
            slot: None,
        };
        unsafe { vm.prepare_stdlib() };
        vm
    }

    pub fn set_slot<T: 'static>(&mut self, value: T) {
        self.slot.insert(Box::new(value) as Box<dyn Any>);
    }

    pub fn get_slot<T: 'static>(&mut self) -> Option<&T> {
        let slot = self.slot.as_ref()?;
        slot.downcast_ref::<T>()
    }

    fn frame(&self) -> &Frame {
        unsafe { self.frames.get_unchecked() }
    }

    fn frame_mut(&mut self) -> &mut Frame {
        unsafe { self.frames.get_mut_unchecked() }
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

    fn next(&mut self) -> Option<&Instruction> {
        if self.is_eof() {
            return None;
        }

        self.frame_mut().ip += 1;

        Some(&self.buffer()[self.ip() - 1])
    }

    fn read_constant(&mut self) -> Option<Value> {
        self.next().cloned().map(|x| x.into_operand())
    }

    fn read_user_function(&mut self) -> Option<UserFunction> {
        self.read_constant()
            .and_then(|c| c.into_object())
            .and_then(|o| match o {
                Object::Function(FunctionKind::User(f)) => Some(f),
                _ => None,
            })
    }

    fn read_number(&mut self) -> f64 {
        self.stack.pop().borrow().as_number()
    }

    fn pop_owned(&mut self) -> Option<Value> {
        Value::try_into_inner(self.stack.pop())
    }

    unsafe fn prepare_stdlib(&mut self) {
        self.global.set_var(
            "isNaN",
            self.statics.get_unchecked(statics::id::ISNAN).clone(),
        );

        let mut math_obj = Value::new(ValueKind::Object(Box::new(Object::Any(AnyObject {}))));
        math_obj.set_property(
            "pow",
            self.statics.get_unchecked(statics::id::MATH_POW).clone(),
        );
        self.global.set_var("Math", Rc::new(RefCell::new(math_obj)));
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
                Opcode::Closure => {
                    let func = self.read_user_function().unwrap();

                    let upvalue_count = func.upvalues as usize;

                    let mut closure =
                        Closure::with_upvalues(func, Vec::with_capacity(upvalue_count));

                    for _ in 0..closure.func.upvalues {
                        let is_local =
                            matches!(self.next().unwrap(), Instruction::Op(Opcode::UpvalueLocal));
                        let stack_idx = self.read_constant().unwrap().as_number() as usize;
                        if is_local {
                            let value =
                                unsafe { self.stack.peek_unchecked(self.frame().sp + stack_idx) };
                            closure.upvalues.push(Upvalue(value.clone()));
                        } else {
                            todo!("Resolve upvalues")
                        }
                    }

                    self.stack
                        .push(Rc::new(RefCell::new(Value::new(ValueKind::Object(
                            Box::new(Object::Function(FunctionKind::Closure(closure))),
                        )))));
                }
                Opcode::Negate => {
                    let maybe_number = self.read_number();

                    self.stack
                        .push(Rc::new(RefCell::new(Value::new(ValueKind::Number(
                            -maybe_number,
                        )))));
                }
                Opcode::LogicalNot => {
                    let is_truthy = self.stack.pop().borrow().is_truthy();

                    self.stack
                        .push(Rc::new(RefCell::new(Value::new(ValueKind::Bool(
                            !is_truthy,
                        )))));
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
                    self.stack.set_relative(self.frame().sp, stack_idx, value);
                }
                Opcode::GetLocal => {
                    let stack_idx = self.read_number() as usize;

                    unsafe {
                        self.stack.push(
                            self.stack
                                .peek_relative_unchecked(self.frame().sp, stack_idx)
                                .clone(),
                        )
                    };
                }
                Opcode::GetUpvalue => {
                    let upvalue_idx = self.read_number() as usize;

                    let value = {
                        let closure_cell = self.frame().func.borrow();
                        let closure = match closure_cell.as_function().unwrap() {
                            FunctionKind::Closure(c) => c,
                            _ => unreachable!(),
                        };
                        closure.upvalues[upvalue_idx].0.clone()
                    };

                    self.stack.push(value);
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

                    let condition_cell = unsafe { self.stack.get_unchecked() };
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
                            let ctx = CallContext {
                                vm: self,
                                args: params,
                                receiver: f.receiver.as_ref().map(|rx| rx.get().clone()),
                            };
                            let result = (f.func)(ctx);
                            self.stack.push(result);
                            continue;
                        }
                        FunctionKind::Closure(u) => u,
                        // There should never be raw user functions
                        _ => unreachable!(),
                    };

                    // By this point we know func_cell is a UserFunction

                    let current_sp = self.stack.get_stack_pointer();
                    self.frame_mut().sp = current_sp;

                    let frame = Frame {
                        buffer: func.func.buffer.clone(),
                        ip: 0,
                        func: func_cell.clone(),
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
                    self.stack
                        .push(Rc::new(RefCell::new(Value::new(ValueKind::Bool(is_less)))));
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
                        .push(Rc::new(RefCell::new(Value::new(ValueKind::Bool(
                            is_less_eq,
                        )))));
                }
                Opcode::Greater => {
                    let rhs_cell = self.stack.pop();
                    let rhs = rhs_cell.borrow();
                    let lhs_cell = self.stack.pop();
                    let lhs = lhs_cell.borrow();

                    let is_greater = matches!(lhs.compare(&rhs), Some(Compare::Greater));
                    self.stack
                        .push(Rc::new(RefCell::new(Value::new(ValueKind::Bool(
                            is_greater,
                        )))));
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
                        .push(Rc::new(RefCell::new(Value::new(ValueKind::Bool(
                            is_greater_eq,
                        )))));
                }
                Opcode::StaticPropertyAccess => {
                    let property = self.pop_owned().unwrap().into_ident().unwrap();
                    let target_cell = self.stack.pop();
                    let value = Value::get_property(&target_cell, &property).unwrap();
                    self.stack.push(value);
                }
                Opcode::ComputedPropertyAccess => todo!(),
                Opcode::Typeof => {
                    let value = self.stack.pop().borrow()._typeof().to_owned();

                    self.stack.push(
                        Value::new(ValueKind::Object(Box::new(Object::String(value)))).into(),
                    );
                }
                _ => unreachable!(),
            };
        }

        Ok(())
    }
}

impl Drop for VM {
    fn drop(&mut self) {
        self.stack.reset();
        self.frames.reset();
    }
}
