pub mod conversions;
pub mod environment;
pub mod frame;
pub mod instruction;
pub mod stack;
pub mod statics;
pub mod upvalue;
pub mod value;

use std::{any::Any, cell::RefCell, collections::HashMap, rc::Rc};

use instruction::{Instruction, Opcode};
use value::Value;

use crate::vm::{
    upvalue::Upvalue,
    value::{
        array::Array,
        function::{CallContext, Closure, FunctionKind, UserFunction},
        ops::compare::Compare,
        ValueKind,
    },
};

use self::{
    environment::Environment,
    frame::Frame,
    instruction::Constant,
    stack::Stack,
    statics::Statics,
    value::object::{AnyObject, Object},
};

#[derive(Debug)]
pub enum VMError {}

/*macro_rules! binary_op {
    ($self:ident, $op:tt) => {
        let (b, a) = (
            $self.read_number(),
            $self.read_number()
        );

        $self.stack.push(Rc::new(RefCell::new(Value::new(ValueKind::Number(a $op b)))));
    }
}*/

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

    pub fn global_mut(&mut self) -> &mut Environment {
        &mut self.global
    }

    pub fn set_slot<T: 'static>(&mut self, value: T) {
        self.slot.insert(Box::new(value) as Box<dyn Any>);
    }

    pub fn get_slot<T: 'static>(&self) -> Option<&T> {
        let slot = self.slot.as_ref()?;
        slot.downcast_ref::<T>()
    }

    pub fn get_slot_mut<T: 'static>(&mut self) -> Option<&mut T> {
        let slot = self.slot.as_mut()?;
        slot.downcast_mut::<T>()
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

    fn read_constant(&mut self) -> Option<Constant> {
        self.next().cloned().map(|x| x.into_operand())
    }

    fn read_user_function(&mut self) -> Option<UserFunction> {
        self.read_constant()
            .and_then(|c| c.into_value())
            .and_then(|v| v.into_object())
            .and_then(|o| match o {
                Object::Function(FunctionKind::User(f)) => Some(f),
                _ => None,
            })
    }

    fn read_number(&mut self) -> f64 {
        self.stack.pop().borrow().as_number()
    }

    fn read_index(&mut self) -> Option<usize> {
        self.stack
            .pop()
            .borrow()
            .as_constant()
            .and_then(|c| c.as_index())
    }

    fn pop_owned(&mut self) -> Option<Value> {
        Value::try_into_inner(self.stack.pop())
    }

    fn read_lhs_rhs(&mut self) -> (Rc<RefCell<Value>>, Rc<RefCell<Value>>) {
        let rhs = self.stack.pop();
        let lhs = self.stack.pop();
        (lhs, rhs)
    }

    fn with_lhs_rhs_borrowed<F, T>(&mut self, func: F) -> T
    where
        F: Fn(&Value, &Value) -> T,
    {
        let (lhs_cell, rhs_cell) = self.read_lhs_rhs();
        let lhs = lhs_cell.borrow();
        let rhs = rhs_cell.borrow();
        func(&*lhs, &*rhs)
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

        let mut console_obj = Value::new(ValueKind::Object(Box::new(Object::Any(AnyObject {}))));
        console_obj.set_property(
            "log",
            self.statics.get_unchecked(statics::id::CONSOLE_LOG).clone(),
        );
        self.global
            .set_var("console", Rc::new(RefCell::new(console_obj)));
    }

    pub fn interpret(&mut self) -> Result<Option<Rc<RefCell<Value>>>, VMError> {
        while !self.is_eof() {
            let instruction = self.buffer()[self.ip()].as_op();

            self.frame_mut().ip += 1;

            match instruction {
                Opcode::Eof => return Ok(None),
                Opcode::Constant => {
                    let constant = self.read_constant().map(|c| c.try_into_value()).unwrap();

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
                        let stack_idx = self.read_constant().and_then(|c| c.into_index()).unwrap();
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
                    let result = self.with_lhs_rhs_borrowed(Value::add).into();
                    self.stack.push(result);
                }
                Opcode::Sub => {
                    let result = self.with_lhs_rhs_borrowed(Value::sub).into();
                    self.stack.push(result);
                }
                Opcode::Mul => {
                    let result = self.with_lhs_rhs_borrowed(Value::mul).into();
                    self.stack.push(result);
                }
                Opcode::Div => {
                    let result = self.with_lhs_rhs_borrowed(Value::div).into();
                    self.stack.push(result);
                }
                Opcode::Rem => {
                    let result = self.with_lhs_rhs_borrowed(Value::rem).into();
                    self.stack.push(result);
                }
                Opcode::Exponentiation => {
                    let result = self.with_lhs_rhs_borrowed(Value::pow).into();
                    self.stack.push(result);
                }
                Opcode::LeftShift => {
                    let result = self.with_lhs_rhs_borrowed(Value::left_shift).into();
                    self.stack.push(result);
                }
                Opcode::RightShift => {
                    let result = self.with_lhs_rhs_borrowed(Value::right_shift).into();
                    self.stack.push(result);
                }
                Opcode::UnsignedRightShift => {
                    let result = self
                        .with_lhs_rhs_borrowed(Value::unsigned_right_shift)
                        .into();
                    self.stack.push(result);
                }
                Opcode::BitwiseAnd => {
                    let result = self.with_lhs_rhs_borrowed(Value::bitwise_and).into();
                    self.stack.push(result);
                }
                Opcode::BitwiseOr => {
                    let result = self.with_lhs_rhs_borrowed(Value::bitwise_or).into();
                    self.stack.push(result);
                }
                Opcode::BitwiseXor => {
                    let result = self.with_lhs_rhs_borrowed(Value::bitwise_xor).into();
                    self.stack.push(result);
                }
                Opcode::SetGlobal => {
                    let name = self.pop_owned().unwrap().into_ident().unwrap();
                    let value = self.stack.pop();

                    self.global.set_var(name, value);
                }
                Opcode::SetGlobalNoValue => {
                    let name = self.pop_owned().unwrap().into_ident().unwrap();

                    self.global
                        .set_var(name, Value::new(ValueKind::Undefined).into());
                }
                Opcode::GetGlobal => {
                    let name = self.pop_owned().unwrap().into_ident().unwrap();

                    // TODO: handle case where var is not defined
                    let value = self.global.get_var(&name).unwrap();

                    self.stack.push(value);
                }
                Opcode::SetLocal => {
                    let stack_idx = self.read_index().unwrap();
                    let value = self.stack.pop();
                    self.stack.set_relative(self.frame().sp, stack_idx, value);
                }
                Opcode::SetLocalNoValue => {
                    let stack_idx = self.read_index().unwrap();
                    self.stack.set_relative(
                        self.frame().sp,
                        stack_idx,
                        Value::new(ValueKind::Undefined).into(),
                    );
                }
                Opcode::GetLocal => {
                    let stack_idx = self.read_index().unwrap();

                    unsafe {
                        self.stack.push(
                            self.stack
                                .peek_relative_unchecked(self.frame().sp, stack_idx)
                                .clone(),
                        )
                    };
                }
                Opcode::GetUpvalue => {
                    let upvalue_idx = self.read_index().unwrap();

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
                    let instruction_count = self.read_index().unwrap();

                    let condition_cell = unsafe { self.stack.get_unchecked() };
                    let condition = condition_cell.borrow().is_truthy();

                    if !condition {
                        self.frame_mut().ip += instruction_count;
                    }
                }
                Opcode::ShortJmpIfTrue => {
                    let instruction_count = self.read_index().unwrap();

                    let condition_cell = unsafe { self.stack.get_unchecked() };
                    let condition = condition_cell.borrow().is_truthy();

                    if condition {
                        self.frame_mut().ip += instruction_count;
                    }
                }
                Opcode::ShortJmpIfNullish => {
                    let instruction_count = self.read_index().unwrap();

                    let condition_cell = unsafe { self.stack.get_unchecked() };
                    let condition = condition_cell.borrow().is_nullish();

                    if !condition {
                        self.frame_mut().ip += instruction_count;
                    }
                }
                Opcode::ShortJmp => {
                    let instruction_count = self.read_index().unwrap();
                    self.frame_mut().ip += instruction_count;
                }
                Opcode::BackJmp => {
                    let instruction_count = self.read_index().unwrap();
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
                Opcode::MultiplicationAssignment => {
                    let target_cell = self.stack.pop();
                    let value_cell = self.stack.pop();
                    let value = value_cell.borrow();
                    target_cell.borrow_mut().mul_assign(&*value);
                    self.stack.push(target_cell);
                }
                Opcode::DivisionAssignment => {
                    let target_cell = self.stack.pop();
                    let value_cell = self.stack.pop();
                    let value = value_cell.borrow();
                    target_cell.borrow_mut().div_assign(&*value);
                    self.stack.push(target_cell);
                }
                Opcode::RemainderAssignment => {
                    let target_cell = self.stack.pop();
                    let value_cell = self.stack.pop();
                    let value = value_cell.borrow();
                    target_cell.borrow_mut().rem_assign(&*value);
                    self.stack.push(target_cell);
                }
                Opcode::ExponentiationAssignment => {
                    let target_cell = self.stack.pop();
                    let value_cell = self.stack.pop();
                    let value = value_cell.borrow();
                    target_cell.borrow_mut().pow_assign(&*value);
                    self.stack.push(target_cell);
                }
                Opcode::LeftShiftAssignment => {
                    let target_cell = self.stack.pop();
                    let value_cell = self.stack.pop();
                    let value = value_cell.borrow();
                    target_cell.borrow_mut().left_shift_assign(&*value);
                    self.stack.push(target_cell);
                }
                Opcode::RightShiftAssignment => {
                    let target_cell = self.stack.pop();
                    let value_cell = self.stack.pop();
                    let value = value_cell.borrow();
                    target_cell.borrow_mut().right_shift_assign(&*value);
                    self.stack.push(target_cell);
                }
                Opcode::UnsignedRightShiftAssignment => {
                    let target_cell = self.stack.pop();
                    let value_cell = self.stack.pop();
                    let value = value_cell.borrow();
                    target_cell
                        .borrow_mut()
                        .unsigned_right_shift_assign(&*value);
                    self.stack.push(target_cell);
                }
                Opcode::BitwiseAndAssignment => {
                    let target_cell = self.stack.pop();
                    let value_cell = self.stack.pop();
                    let value = value_cell.borrow();
                    target_cell.borrow_mut().bitwise_and_assign(&*value);
                    self.stack.push(target_cell);
                }
                Opcode::BitwiseOrAssignment => {
                    let target_cell = self.stack.pop();
                    let value_cell = self.stack.pop();
                    let value = value_cell.borrow();
                    target_cell.borrow_mut().bitwise_or_assign(&*value);
                    self.stack.push(target_cell);
                }
                Opcode::BitwiseXorAssignment => {
                    let target_cell = self.stack.pop();
                    let value_cell = self.stack.pop();
                    let value = value_cell.borrow();
                    target_cell.borrow_mut().bitwise_xor_assign(&*value);
                    self.stack.push(target_cell);
                }
                Opcode::LogicalAndAssignment => {
                    let target_cell = self.stack.pop();
                    let value_cell = self.stack.pop();
                    let value = value_cell.borrow();
                    target_cell.borrow_mut().logical_and_assign(&*value);
                    self.stack.push(target_cell);
                }
                Opcode::LogicalOrAssignment => {
                    let target_cell = self.stack.pop();
                    let value_cell = self.stack.pop();
                    let value = value_cell.borrow();
                    target_cell.borrow_mut().logical_and_assign(&*value);
                    self.stack.push(target_cell);
                }
                Opcode::LogicalNullishAssignment => {
                    let target_cell = self.stack.pop();
                    let value_cell = self.stack.pop();
                    let value = value_cell.borrow();
                    target_cell.borrow_mut().nullish_coalescing_assign(&*value);
                    self.stack.push(target_cell);
                }
                Opcode::FunctionCall => {
                    let param_count = self.read_index().unwrap();
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
                    self.frames.pop();
                    if self.frames.get_stack_pointer() == 0 {
                        if self.stack.get_stack_pointer() == 0 {
                            return Ok(None);
                        } else {
                            return Ok(Some(self.stack.pop()));
                        }
                    }

                    let ret = self.stack.pop();

                    // TODO: cleanup stack
                    self.stack.set_stack_pointer(self.frame().sp);
                    self.stack.push(ret);
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
                    let value =
                        Value::unwrap_or_undefined(Value::get_property(&target_cell, &property));
                    self.stack.push(value);
                }
                Opcode::Equality => {
                    let eq = self.with_lhs_rhs_borrowed(Value::lossy_equal);
                    self.stack.push(Value::new(ValueKind::Bool(eq)).into());
                }
                Opcode::Inequality => {
                    let eq = self.with_lhs_rhs_borrowed(Value::lossy_equal);
                    self.stack.push(Value::new(ValueKind::Bool(!eq)).into());
                }
                Opcode::StrictEquality => {
                    let eq = self.with_lhs_rhs_borrowed(Value::strict_equal);
                    self.stack.push(Value::new(ValueKind::Bool(eq)).into());
                }
                Opcode::StrictInequality => {
                    let eq = self.with_lhs_rhs_borrowed(Value::strict_equal);
                    self.stack.push(Value::new(ValueKind::Bool(!eq)).into());
                }
                Opcode::Typeof => {
                    let value = self.stack.pop().borrow()._typeof().to_owned();

                    self.stack.push(
                        Value::new(ValueKind::Object(Box::new(Object::String(value)))).into(),
                    );
                }
                Opcode::PostfixIncrement | Opcode::PostfixDecrement => {
                    let value_cell = self.stack.pop();
                    let mut value = value_cell.borrow_mut();
                    let one = Value::new(ValueKind::Number(1f64));
                    let result = if instruction == Opcode::PostfixIncrement {
                        value.add_assign(&one);
                        value.sub(&one)
                    } else {
                        todo!()
                    };
                    self.stack.push(result.into());
                }
                Opcode::Assignment => {
                    let value_cell = self.stack.pop();
                    let target_cell = self.stack.pop();

                    let value = value_cell.borrow();
                    // TODO: cloning might not be the right thing to do
                    let value = value.clone();

                    let mut target = target_cell.borrow_mut();
                    *target = value;
                    self.stack.push(target_cell.clone());
                }
                Opcode::Void => {
                    self.stack.pop();
                    self.stack.push(Value::new(ValueKind::Undefined).into());
                }
                Opcode::ArrayLiteral => {
                    let element_count = self.read_index().unwrap();
                    let mut elements = Vec::with_capacity(element_count);
                    for _ in 0..element_count {
                        elements.push(self.stack.pop());
                    }
                    self.stack.push(
                        Value::new(ValueKind::Object(Box::new(Object::Array(Array::new(
                            elements,
                        )))))
                        .into(),
                    )
                }
                Opcode::ObjectLiteral => {
                    let property_count = self.read_index().unwrap();

                    let mut fields = HashMap::new();
                    let mut raw_fields = Vec::new();

                    for _ in 0..property_count {
                        let value = self.stack.pop();
                        raw_fields.push(value);
                    }

                    for value in raw_fields.into_iter().rev() {
                        let key = self.read_constant().unwrap().into_ident().unwrap();
                        fields.insert(key.into_boxed_str(), value);
                    }

                    let value = Value {
                        constructor: None,
                        fields,
                        kind: ValueKind::Object(Box::new(Object::Any(AnyObject {}))),
                    };
                    self.stack.push(value.into());
                }
                Opcode::ComputedPropertyAccess => {
                    let property_cell = self.stack.pop();
                    let target_cell = self.stack.pop();
                    let property = property_cell.borrow();
                    let property_s = property.as_string_lossy().unwrap();

                    let prop = Value::get_property(&target_cell, &*property_s)
                        .unwrap_or_else(|| Value::new(ValueKind::Undefined).into());

                    self.stack.push(prop);
                }
                _ => unreachable!("{:?}", instruction),
            };
        }

        Ok(None)
    }
}

impl Drop for VM {
    fn drop(&mut self) {
        self.stack.dump();
        self.stack.reset();
        self.frames.reset();
    }
}
