use crate::gc::Handle;

use super::{instruction::Opcode, value::Value, VM};

pub enum DispatchResult {
    Return(Option<Handle<Value>>),
}

mod handlers {
    use std::collections::HashMap;

    use crate::{
        gc::Handle,
        js_std::{self, error::MaybeRc},
        vm::{
            frame::{Frame, Loop, UnwindHandler},
            instruction::{Constant, Instruction, Opcode},
            upvalue::Upvalue,
            value::{
                array::Array,
                function::{CallContext, Closure, FunctionKind, Receiver},
                object::Object,
                ops::compare::Compare,
                Value, ValueKind,
            },
            VM,
        },
    };

    use super::DispatchResult;

    pub fn constant(vm: &mut VM) {
        let mut constant = vm.read_constant().and_then(|c| c.try_into_value()).unwrap();

        // Values emitted by the compiler do not have a [[Prototype]] set
        // so we need to do that here when pushing a value onto the stack
        unsafe {
            constant
                .borrow_mut_unbounded()
                .detect_internal_properties(vm);

            constant.set_marker(vm.gc_marker);
        }

        vm.stack.push(constant);
    }

    pub fn closure(vm: &mut VM) {
        let func = vm.read_user_function().unwrap();

        let upvalue_count = func.upvalues as usize;

        let mut closure = Closure::with_upvalues(func, Vec::with_capacity(upvalue_count));

        for _ in 0..closure.func.upvalues {
            let is_local = matches!(vm.next().unwrap(), Instruction::Op(Opcode::UpvalueLocal));
            let stack_idx = vm.read_constant().and_then(|c| c.into_index()).unwrap();
            if is_local {
                let value = unsafe { vm.stack.peek_unchecked(vm.frame().sp + stack_idx) };
                closure.upvalues.push(Upvalue(value.clone()));
            } else {
                todo!("Resolve upvalues")
            }
        }

        vm.stack.push(
            vm.create_js_value(FunctionKind::Closure(closure))
                .into_handle(vm),
        );
    }

    pub fn negate(vm: &mut VM) {
        let maybe_number = vm.read_number();

        vm.stack
            .push(vm.create_js_value(-maybe_number).into_handle(vm));
    }

    pub fn positive(vm: &mut VM) {
        let maybe_number = vm.read_number();

        vm.stack
            .push(vm.create_js_value(maybe_number).into_handle(vm));
    }

    pub fn logical_not(vm: &mut VM) {
        let is_truthy = unsafe { vm.stack.pop().borrow_unbounded() }.is_truthy();

        vm.stack
            .push(vm.create_js_value(!is_truthy).into_handle(vm));
    }

    pub fn add(vm: &mut VM) {
        let result = vm.with_lhs_rhs_borrowed(Value::add).into_handle(vm);
        vm.stack.push(result);
    }

    pub fn sub(vm: &mut VM) {
        let result = vm.with_lhs_rhs_borrowed(Value::sub).into_handle(vm);
        vm.stack.push(result);
    }

    pub fn mul(vm: &mut VM) {
        let result = vm.with_lhs_rhs_borrowed(Value::mul).into_handle(vm);
        vm.stack.push(result);
    }

    pub fn div(vm: &mut VM) {
        let result = vm.with_lhs_rhs_borrowed(Value::div).into_handle(vm);
        vm.stack.push(result);
    }

    pub fn rem(vm: &mut VM) {
        let result = vm.with_lhs_rhs_borrowed(Value::rem).into_handle(vm);
        vm.stack.push(result);
    }

    pub fn exponentiation(vm: &mut VM) {
        let result = vm.with_lhs_rhs_borrowed(Value::pow).into_handle(vm);
        vm.stack.push(result);
    }

    pub fn left_shift(vm: &mut VM) {
        let result = vm.with_lhs_rhs_borrowed(Value::left_shift).into_handle(vm);
        vm.stack.push(result);
    }

    pub fn right_shift(vm: &mut VM) {
        let result = vm.with_lhs_rhs_borrowed(Value::right_shift).into_handle(vm);
        vm.stack.push(result);
    }

    pub fn unsigned_right_shift(vm: &mut VM) {
        let result = vm
            .with_lhs_rhs_borrowed(Value::unsigned_right_shift)
            .into_handle(vm);
        vm.stack.push(result);
    }

    pub fn bitwise_and(vm: &mut VM) {
        let result = vm.with_lhs_rhs_borrowed(Value::bitwise_and).into_handle(vm);
        vm.stack.push(result);
    }

    pub fn bitwise_or(vm: &mut VM) {
        let result = vm.with_lhs_rhs_borrowed(Value::bitwise_or).into_handle(vm);
        vm.stack.push(result);
    }

    pub fn bitwise_xor(vm: &mut VM) {
        let result = vm.with_lhs_rhs_borrowed(Value::bitwise_xor).into_handle(vm);
        vm.stack.push(result);
    }

    pub fn bitwise_not(vm: &mut VM) {
        let result = vm.with_lhs_borrowed(Value::bitwise_not).into_handle(vm);
        vm.stack.push(result);
    }

    pub fn set_global(vm: &mut VM) {
        let name = vm.read_constant().and_then(Constant::into_ident).unwrap();
        let value = vm.stack.pop();

        let mut global = unsafe { vm.global.borrow_mut_unbounded() };
        global.set_property(name, value);
    }

    pub fn set_global_no_value(vm: &mut VM) {
        let name = vm.read_constant().and_then(Constant::into_ident).unwrap();

        let mut global = unsafe { vm.global.borrow_mut_unbounded() };
        global.set_property(name, Value::new(ValueKind::Undefined).into_handle(vm));
    }

    pub fn get_global(vm: &mut VM) -> Result<(), Handle<Value>> {
        let name = vm.read_constant().and_then(Constant::into_ident).unwrap();

        let value = Value::get_property(vm, &vm.global, &name, None).ok_or_else(|| {
            js_std::error::create_error(MaybeRc::Owned(&format!("{} is not defined", name)), vm)
        })?;

        vm.stack.push(value);
        Ok(())
    }

    pub fn set_local(vm: &mut VM) {
        let stack_idx = vm.read_index().unwrap();
        let value = vm.stack.pop();
        vm.stack.set_relative(vm.frame().sp, stack_idx, value);
    }

    pub fn set_local_no_value(vm: &mut VM) {
        let stack_idx = vm.read_index().unwrap();
        vm.stack.set_relative(
            vm.frame().sp,
            stack_idx,
            Value::new(ValueKind::Undefined).into_handle(vm),
        );
    }

    pub fn get_local(vm: &mut VM) {
        let stack_idx = vm.read_index().unwrap();

        unsafe {
            vm.stack.push(
                vm.stack
                    .peek_relative_unchecked(vm.frame().sp, stack_idx)
                    .clone(),
            )
        };
    }

    pub fn get_upvalue(vm: &mut VM) {
        let upvalue_idx = vm.read_index().unwrap();

        let value = {
            let closure_cell = unsafe { vm.frame().func.borrow_unbounded() };
            let closure = match closure_cell.as_function().unwrap() {
                FunctionKind::Closure(c) => c,
                _ => unreachable!(),
            };
            closure.upvalues[upvalue_idx].0.clone()
        };

        vm.stack.push(value);
    }

    pub fn short_jmp_if_false(vm: &mut VM) {
        let instruction_count = vm.read_index().unwrap();

        let condition_cell = unsafe { vm.stack.get_unchecked() };
        let condition = unsafe { condition_cell.borrow_unbounded() }.is_truthy();

        if !condition {
            vm.frame_mut().ip += instruction_count;
        }
    }

    pub fn short_jmp_if_true(vm: &mut VM) {
        let instruction_count = vm.read_index().unwrap();

        let condition_cell = unsafe { vm.stack.get_unchecked() };
        let condition = unsafe { condition_cell.borrow_unbounded() }.is_truthy();

        if condition {
            vm.frame_mut().ip += instruction_count;
        }
    }

    pub fn short_jmp_if_nullish(vm: &mut VM) {
        let instruction_count = vm.read_index().unwrap();

        let condition_cell = unsafe { vm.stack.get_unchecked() };
        let condition = unsafe { condition_cell.borrow_unbounded() }.is_nullish();

        if !condition {
            vm.frame_mut().ip += instruction_count;
        }
    }

    pub fn short_jmp(vm: &mut VM) {
        let instruction_count = vm.read_index().unwrap();
        vm.frame_mut().ip += instruction_count;
    }

    pub fn back_jmp(vm: &mut VM) {
        let instruction_count = vm.read_index().unwrap();
        vm.frame_mut().ip -= instruction_count;
    }

    pub fn pop(vm: &mut VM) {
        vm.stack.pop();
    }

    pub fn pop_unwind_handler(vm: &mut VM) {
        vm.unwind_handlers.pop();
    }

    pub fn addition_assignment(vm: &mut VM) {
        let value_cell = vm.stack.pop();
        let target_cell = vm.stack.pop();
        let value = unsafe { value_cell.borrow_unbounded() };
        unsafe { target_cell.borrow_mut_unbounded() }.add_assign(&*value);
        vm.stack.push(target_cell);
    }

    pub fn subtraction_assignment(vm: &mut VM) {
        let value_cell = vm.stack.pop();
        let target_cell = vm.stack.pop();
        let value = unsafe { value_cell.borrow_unbounded() };
        unsafe { target_cell.borrow_mut_unbounded() }.sub_assign(&*value);
        vm.stack.push(target_cell);
    }

    pub fn multiplication_assignment(vm: &mut VM) {
        let value_cell = vm.stack.pop();
        let target_cell = vm.stack.pop();
        let value = unsafe { value_cell.borrow_unbounded() };
        unsafe { target_cell.borrow_mut_unbounded() }.mul_assign(&*value);
        vm.stack.push(target_cell);
    }

    pub fn division_assignment(vm: &mut VM) {
        let value_cell = vm.stack.pop();
        let target_cell = vm.stack.pop();
        let value = unsafe { value_cell.borrow_unbounded() };
        unsafe { target_cell.borrow_mut_unbounded() }.div_assign(&*value);
        vm.stack.push(target_cell);
    }

    pub fn remainder_assignment(vm: &mut VM) {
        let value_cell = vm.stack.pop();
        let target_cell = vm.stack.pop();
        let value = unsafe { value_cell.borrow_unbounded() };
        unsafe { target_cell.borrow_mut_unbounded() }.rem_assign(&*value);
        vm.stack.push(target_cell);
    }

    pub fn exponentiation_assignment(vm: &mut VM) {
        let value_cell = vm.stack.pop();
        let target_cell = vm.stack.pop();
        let value = unsafe { value_cell.borrow_unbounded() };
        unsafe { target_cell.borrow_mut_unbounded() }.pow_assign(&*value);
        vm.stack.push(target_cell);
    }

    pub fn left_shift_assignment(vm: &mut VM) {
        let value_cell = vm.stack.pop();
        let target_cell = vm.stack.pop();
        let value = unsafe { value_cell.borrow_unbounded() };
        unsafe { target_cell.borrow_mut_unbounded() }.left_shift_assign(&*value);
        vm.stack.push(target_cell);
    }

    pub fn right_shift_assignment(vm: &mut VM) {
        let value_cell = vm.stack.pop();
        let target_cell = vm.stack.pop();
        let value = unsafe { value_cell.borrow_unbounded() };
        unsafe { target_cell.borrow_mut_unbounded() }.right_shift_assign(&*value);
        vm.stack.push(target_cell);
    }

    pub fn unsigned_right_shift_assignment(vm: &mut VM) {
        let value_cell = vm.stack.pop();
        let target_cell = vm.stack.pop();
        let value = unsafe { value_cell.borrow_unbounded() };
        unsafe { target_cell.borrow_mut_unbounded() }.unsigned_right_shift_assign(&*value);
        vm.stack.push(target_cell);
    }

    pub fn bitwise_and_assignment(vm: &mut VM) {
        let value_cell = vm.stack.pop();
        let target_cell = vm.stack.pop();
        let value = unsafe { value_cell.borrow_unbounded() };
        unsafe { target_cell.borrow_mut_unbounded() }.bitwise_and_assign(&*value);
        vm.stack.push(target_cell);
    }

    pub fn bitwise_or_assignment(vm: &mut VM) {
        let value_cell = vm.stack.pop();
        let target_cell = vm.stack.pop();
        let value = unsafe { value_cell.borrow_unbounded() };
        unsafe { target_cell.borrow_mut_unbounded() }.bitwise_or_assign(&*value);
        vm.stack.push(target_cell);
    }

    pub fn bitwise_xor_assignment(vm: &mut VM) {
        let value_cell = vm.stack.pop();
        let target_cell = vm.stack.pop();
        let value = unsafe { value_cell.borrow_unbounded() };
        unsafe { target_cell.borrow_mut_unbounded() }.bitwise_xor_assign(&*value);
        vm.stack.push(target_cell);
    }

    pub fn logical_and_assignment(vm: &mut VM) {
        let value_cell = vm.stack.pop();
        let target_cell = vm.stack.pop();
        let value = unsafe { value_cell.borrow_unbounded() };
        unsafe { target_cell.borrow_mut_unbounded() }.logical_and_assign(&*value);
        vm.stack.push(target_cell);
    }

    pub fn logical_or_assignment(vm: &mut VM) {
        let value_cell = vm.stack.pop();
        let target_cell = vm.stack.pop();
        let value = unsafe { value_cell.borrow_unbounded() };
        unsafe { target_cell.borrow_mut_unbounded() }.logical_or_assign(&*value);
        vm.stack.push(target_cell);
    }

    pub fn logical_nullish_assignment(vm: &mut VM) {
        let value_cell = vm.stack.pop();
        let target_cell = vm.stack.pop();
        let value = unsafe { value_cell.borrow_unbounded() };
        unsafe { target_cell.borrow_mut_unbounded() }.nullish_coalescing_assign(&*value);
        vm.stack.push(target_cell);
    }

    pub fn constructor_call(vm: &mut VM) -> Result<(), Handle<Value>> {
        let param_count = vm.read_index().unwrap();
        let mut params = Vec::new();
        for _ in 0..param_count {
            params.push(vm.stack.pop());
        }

        let func_cell = vm.stack.pop();
        let mut func_cell_ref = unsafe { func_cell.borrow_mut_unbounded() };
        let func_cell_kind = func_cell_ref.as_function_mut().unwrap();
        let this = func_cell_kind.construct(&func_cell);
        let func = match func_cell_kind {
            FunctionKind::Native(f) => {
                if !f.ctor.constructable() {
                    // User tried to invoke non-constructor as a constructor
                    return Err(js_std::error::create_error(
                        MaybeRc::Owned(&format!("{} is not a constructor", f.name)),
                        vm,
                    ));
                }

                let receiver = Some(this.into_handle(vm));
                let ctx = CallContext {
                    vm,
                    args: &mut params,
                    ctor: true,
                    receiver,
                    // state: &mut state,
                    // function_call_response: None,
                };
                let result = (f.func)(ctx)?;

                vm.stack.push(result);

                return Ok(());
            }
            FunctionKind::Closure(closure) => {
                closure.func.receiver = Some(Receiver::Bound(this.into_handle(vm)));
                closure
            }
            // There should never be raw user functions
            _ => unreachable!(),
        };

        // By this point we know func_cell is a UserFunction
        // TODO: get rid of this copy paste and share code with Opcode::FunctionCall

        let current_sp = vm.stack.get_stack_pointer();

        // let state = vm.frame_mut().state.take();
        let frame = Frame {
            buffer: func.func.buffer.clone(),
            ip: 0,
            func: Handle::clone(&func_cell),
            sp: current_sp,
            // state,
            // resume: None,
        };

        vm.frames.push(frame);

        let origin_param_count = func.func.params as usize;
        let param_count = params.len();

        for param in params.into_iter().rev() {
            vm.stack.push(param);
        }

        for _ in 0..(origin_param_count.saturating_sub(param_count)) {
            vm.stack
                .push(Value::new(ValueKind::Undefined).into_handle(vm));
        }

        Ok(())
    }

    pub fn get_this(vm: &mut VM) {
        let this = {
            let frame = vm.frame();
            let func = unsafe { frame.func.borrow_unbounded() };
            let raw_func = func
                .as_function()
                .and_then(FunctionKind::as_closure)
                .unwrap();

            let receiver = raw_func.func.receiver.as_ref().unwrap();
            receiver.get().clone()
        };
        vm.stack.push(this);
    }

    pub fn get_global_this(vm: &mut VM) {
        vm.stack.push(Handle::clone(&vm.global));
    }

    pub fn evaluate_module(vm: &mut VM) {
        let (value_cell, buffer) = {
            let module = vm.read_constant().and_then(Constant::into_value).unwrap();

            let buffer = unsafe {
                module
                    .borrow_mut_unbounded()
                    .as_function_mut()
                    .unwrap()
                    .as_module_mut()
                    .unwrap()
                    .buffer
                    .take()
                    .unwrap()
            };

            (module, buffer)
        };

        let current_sp = vm.stack.get_stack_pointer();
        vm.frame_mut().sp = current_sp;

        let frame = Frame {
            func: value_cell,
            buffer,
            ip: 0,
            sp: current_sp,
        };

        vm.frames.push(frame);
    }

    pub fn function_call(vm: &mut VM) -> Result<(), Handle<Value>> {
        let param_count = vm.read_index().unwrap();
        let mut params = Vec::new();
        for _ in 0..param_count {
            params.push(vm.stack.pop());
        }

        let func_cell = vm.stack.pop();
        vm.begin_function_call(func_cell, params)
    }

    pub fn try_block(vm: &mut VM) {
        let catch_idx = vm.read_constant().and_then(Constant::into_index).unwrap();
        let should_capture_error = vm.read_op().unwrap() == Opcode::SetLocal;

        let error_catch_idx = if should_capture_error {
            vm.read_index()
        } else {
            None
        };

        let current_ip = vm.ip();
        let handler = UnwindHandler {
            catch_ip: current_ip + catch_idx,
            catch_value_sp: error_catch_idx,
            finally_ip: None, // TODO: support finally
            frame_pointer: vm.frames.get_stack_pointer(),
        };
        vm.unwind_handlers.push(handler)
    }

    pub fn throw(vm: &mut VM) -> Result<(), Handle<Value>> {
        let value = vm.stack.pop();
        Err(value)
    }

    pub fn return_module(vm: &mut VM) {
        let frame = vm.frames.pop();
        let func_ref = unsafe { frame.func.borrow_unbounded() };
        let func = func_ref
            .as_function()
            .and_then(FunctionKind::as_module)
            .unwrap();

        let exports = if let Some(default) = &func.exports.default {
            Handle::clone(default)
        } else {
            vm.create_object().into_handle(vm)
        };

        {
            let mut exports_mut = unsafe { exports.borrow_mut_unbounded() };
            for (key, value) in &func.exports.named {
                exports_mut.set_property(&**key, Handle::clone(value));
            }
        }

        vm.stack
            .discard_multiple(vm.stack.get_stack_pointer() - frame.sp);

        unsafe { vm.stack.set_stack_pointer(frame.sp) };
        vm.stack.push(exports);
    }

    pub fn return_(vm: &mut VM, frame_idx: usize) -> Result<Option<DispatchResult>, Handle<Value>> {
        // We might be in a try block, in which case we need to remove the handler
        let maybe_tc_frame_pointer = unsafe { vm.unwind_handlers.get() }.map(|c| c.frame_pointer);

        let frame_pointer = vm.frames.get_stack_pointer();

        if maybe_tc_frame_pointer == Some(frame_pointer) {
            vm.unwind_handlers.pop();
        }

        // Restore VM state to where we were before the function call happened
        let this = vm.frames.pop();

        let ret = if vm.stack.get_stack_pointer() == 0 {
            None
        } else {
            Some(vm.stack.pop())
        };

        vm.stack
            .discard_multiple(vm.stack.get_stack_pointer() - this.sp);

        if vm.frames.get_stack_pointer() == frame_idx {
            if let Some(value) = ret {
                return Ok(Some(DispatchResult::Return(Some(value))));
            } else {
                return Ok(Some(DispatchResult::Return(None)));
            }
        }

        let func_ref = unsafe { this.func.borrow_unbounded() };
        if let Some(this) = func_ref
            .as_function()
            .and_then(FunctionKind::as_closure)
            .and_then(|c| c.func.receiver.as_ref())
        {
            vm.stack.push(Handle::clone(this.get()));
        } else {
            vm.stack.push(ret.unwrap());
        }

        Ok(None)
    }

    pub fn less(vm: &mut VM) {
        let rhs_cell = vm.stack.pop();
        let rhs = unsafe { rhs_cell.borrow_unbounded() };
        let lhs_cell = vm.stack.pop();
        let lhs = unsafe { lhs_cell.borrow_unbounded() };

        let is_less = matches!(lhs.compare(&rhs), Some(Compare::Less));
        vm.stack.push(vm.create_js_value(is_less).into_handle(vm));
    }

    pub fn less_equal(vm: &mut VM) {
        let rhs_cell = vm.stack.pop();
        let rhs = unsafe { rhs_cell.borrow_unbounded() };
        let lhs_cell = vm.stack.pop();
        let lhs = unsafe { lhs_cell.borrow_unbounded() };

        let is_less_eq = matches!(
            lhs.compare(&rhs),
            Some(Compare::Less) | Some(Compare::Equal)
        );
        vm.stack
            .push(vm.create_js_value(is_less_eq).into_handle(vm));
    }

    pub fn greater(vm: &mut VM) {
        let rhs_cell = vm.stack.pop();
        let rhs = unsafe { rhs_cell.borrow_unbounded() };
        let lhs_cell = vm.stack.pop();
        let lhs = unsafe { lhs_cell.borrow_unbounded() };

        let is_greater = matches!(lhs.compare(&rhs), Some(Compare::Greater));
        vm.stack
            .push(vm.create_js_value(is_greater).into_handle(vm));
    }

    pub fn greater_equal(vm: &mut VM) {
        let rhs_cell = vm.stack.pop();
        let rhs = unsafe { rhs_cell.borrow_unbounded() };
        let lhs_cell = vm.stack.pop();
        let lhs = unsafe { lhs_cell.borrow_unbounded() };

        let is_greater_eq = matches!(
            lhs.compare(&rhs),
            Some(Compare::Greater) | Some(Compare::Equal)
        );
        vm.stack
            .push(vm.create_js_value(is_greater_eq).into_handle(vm));
    }

    pub fn static_property_access(vm: &mut VM) {
        let property = vm.read_constant().and_then(Constant::into_ident).unwrap();
        let is_assignment = vm.read_index().unwrap() == 1;
        let target_cell = vm.stack.pop();

        let value = if is_assignment {
            let maybe_value = Value::get_property(vm, &target_cell, &property, None);
            maybe_value.unwrap_or_else(|| {
                let mut target = unsafe { target_cell.borrow_mut_unbounded() };
                let value = Value::new(ValueKind::Undefined).into_handle(vm);
                target.set_property(property, Handle::clone(&value));
                value
            })
        } else {
            Value::unwrap_or_undefined(Value::get_property(vm, &target_cell, &property, None), vm)
        };
        vm.stack.push(value);
    }

    pub fn equality(vm: &mut VM) {
        let eq = vm.with_lhs_rhs_borrowed(Value::lossy_equal);
        vm.stack.push(vm.create_js_value(eq).into_handle(vm));
    }

    pub fn inequality(vm: &mut VM) {
        let eq = vm.with_lhs_rhs_borrowed(Value::lossy_equal);
        vm.stack.push(vm.create_js_value(!eq).into_handle(vm));
    }

    pub fn strict_equality(vm: &mut VM) {
        let eq = vm.with_lhs_rhs_borrowed(Value::strict_equal);
        vm.stack.push(vm.create_js_value(eq).into_handle(vm));
    }

    pub fn strict_inequality(vm: &mut VM) {
        let eq = vm.with_lhs_rhs_borrowed(Value::strict_equal);
        vm.stack.push(vm.create_js_value(!eq).into_handle(vm));
    }

    pub fn typeof_(vm: &mut VM) {
        let value = unsafe { vm.stack.pop().borrow_unbounded() }
            ._typeof()
            .to_owned();

        vm.stack
            .push(vm.create_js_value(Object::String(value)).into_handle(vm));
    }

    pub fn postfix_increment_decrement(vm: &mut VM, opcode: Opcode) {
        let value_cell = vm.stack.pop();
        let mut value = unsafe { value_cell.borrow_mut_unbounded() };
        let one = vm.create_js_value(1f64);
        let result = if opcode == Opcode::PostfixIncrement {
            value.add_assign(&one);
            value.sub(&one)
        } else {
            value.sub_assign(&one);
            value.add(&one)
        };
        vm.stack.push(result.into_handle(vm));
    }

    pub fn assignment(vm: &mut VM) {
        let value_cell = vm.stack.pop();
        let target_cell = vm.stack.pop();

        let value = unsafe { value_cell.borrow_unbounded() };
        // TODO: cloning might not be the right thing to do
        let value = value.clone();

        let mut target = unsafe { target_cell.borrow_mut_unbounded() };
        **target = value;
        vm.stack.push(target_cell.clone());
    }

    pub fn void(vm: &mut VM) {
        vm.stack.pop();
        vm.stack
            .push(Value::new(ValueKind::Undefined).into_handle(vm));
    }

    pub fn array_literal(vm: &mut VM) {
        let element_count = vm.read_index().unwrap();
        let mut elements = Vec::with_capacity(element_count);
        for _ in 0..element_count {
            elements.push(vm.stack.pop());
        }
        vm.stack
            .push(vm.create_array(Array::new(elements)).into_handle(vm));
    }

    pub fn object_literal(vm: &mut VM) {
        let property_count = vm.read_index().unwrap();

        let mut fields = HashMap::new();
        let mut raw_fields = Vec::new();

        for _ in 0..property_count {
            let value = vm.stack.pop();
            raw_fields.push(value);
        }

        for value in raw_fields.into_iter().rev() {
            let key = vm.read_constant().unwrap().into_ident().unwrap();
            fields.insert(key.into_boxed_str(), value);
        }

        vm.stack
            .push(vm.create_object_with_fields(fields).into_handle(vm));
    }

    pub fn computed_property_access(vm: &mut VM) {
        let property_cell = vm.stack.pop();
        let is_assignment = vm.read_index().unwrap() == 1;
        let target_cell = vm.stack.pop();
        let property = unsafe { property_cell.borrow_unbounded() };
        let property_s = property.to_string();

        let value = if is_assignment {
            let maybe_value = Value::get_property(vm, &target_cell, &*property_s, None);
            maybe_value.unwrap_or_else(|| {
                let mut target = unsafe { target_cell.borrow_mut_unbounded() };
                let value = Value::new(ValueKind::Undefined).into_handle(vm);
                target.set_property(property_s.to_string(), Handle::clone(&value));
                value
            })
        } else {
            Value::unwrap_or_undefined(
                Value::get_property(vm, &target_cell, &*property_s, None),
                vm,
            )
        };

        vm.stack.push(value);
    }

    pub fn loop_continue(vm: &mut VM) {
        let this = unsafe { vm.loops.get_unchecked() };
        vm.frame_mut().ip = this.condition_ip;
    }

    pub fn loop_break(vm: &mut VM) {
        let this = unsafe { vm.loops.get_unchecked() };
        vm.frame_mut().ip = this.end_ip;
    }

    pub fn loop_start(vm: &mut VM) {
        let condition_offset = vm.read_index().unwrap();
        let end_offset = vm.read_index().unwrap();
        let ip = vm.ip();
        let info = Loop {
            condition_ip: (ip + condition_offset),
            end_ip: (ip + end_offset),
        };
        vm.loops.push(info);
    }

    pub fn loop_end(vm: &mut VM) {
        vm.loops.pop();
    }

    pub fn export_default(vm: &mut VM) -> Result<(), Handle<Value>> {
        let export_status = {
            let value = vm.stack.pop();
            let mut func_ref = unsafe { vm.frame().func.borrow_mut_unbounded() };

            let maybe_module = func_ref
                .as_function_mut()
                .and_then(FunctionKind::as_module_mut);

            if let Some(module) = maybe_module {
                module.exports.default = Some(value);
                true
            } else {
                false
            }
        };

        if !export_status {
            return Err(js_std::error::create_error(
                MaybeRc::Owned("Can only export at the top level in a module"),
                vm,
            ));
        }

        Ok(())
    }

    pub fn to_primitive(vm: &mut VM) -> Result<(), Handle<Value>> {
        let obj_cell = vm.stack.pop();

        {
            // If this is already a primitive value, we do not need to try to convert it
            let obj = unsafe { obj_cell.borrow_unbounded() };
            if obj.is_primitive() {
                vm.stack.push(Handle::clone(&obj_cell));
                return Ok(());
            }
        }

        let to_prim = Value::get_property(vm, &obj_cell, "toString", None)
            .or_else(|| Value::get_property(vm, &obj_cell, "valueOf", None))
            .ok_or_else(|| {
                js_std::error::create_error(
                    MaybeRc::Owned("Cannot convert object to primitive value"),
                    vm,
                )
            })?;

        vm.begin_function_call(to_prim, Vec::new())
    }

    pub fn debugger(vm: &mut VM) {
        vm.agent.debugger();
    }
}

pub fn handle(
    vm: &mut VM,
    opcode: Opcode,
    frame_idx: usize,
) -> Result<Option<DispatchResult>, Handle<Value>> {
    match opcode {
        Opcode::Eof => return Ok(Some(DispatchResult::Return(None))),
        Opcode::Constant => handlers::constant(vm),
        Opcode::Closure => handlers::closure(vm),
        Opcode::Negate => handlers::negate(vm),
        Opcode::Positive => handlers::positive(vm),
        Opcode::LogicalNot => handlers::logical_not(vm),
        Opcode::Add => handlers::add(vm),
        Opcode::Sub => handlers::sub(vm),
        Opcode::Mul => handlers::mul(vm),
        Opcode::Div => handlers::div(vm),
        Opcode::Rem => handlers::rem(vm),
        Opcode::Exponentiation => handlers::exponentiation(vm),
        Opcode::LeftShift => handlers::left_shift(vm),
        Opcode::RightShift => handlers::right_shift(vm),
        Opcode::UnsignedRightShift => handlers::unsigned_right_shift(vm),
        Opcode::BitwiseAnd => handlers::bitwise_and(vm),
        Opcode::BitwiseOr => handlers::bitwise_or(vm),
        Opcode::BitwiseXor => handlers::bitwise_xor(vm),
        Opcode::BitwiseNot => handlers::bitwise_not(vm),
        Opcode::SetGlobal => handlers::set_global(vm),
        Opcode::SetGlobalNoValue => handlers::set_global_no_value(vm),
        Opcode::GetGlobal => handlers::get_global(vm)?,
        Opcode::SetLocal => handlers::set_local(vm),
        Opcode::SetLocalNoValue => handlers::set_local_no_value(vm),
        Opcode::GetLocal => handlers::get_local(vm),
        Opcode::GetUpvalue => handlers::get_upvalue(vm),
        Opcode::ShortJmpIfFalse => handlers::short_jmp_if_false(vm),
        Opcode::ShortJmpIfTrue => handlers::short_jmp_if_true(vm),
        Opcode::ShortJmpIfNullish => handlers::short_jmp_if_nullish(vm),
        Opcode::ShortJmp => handlers::short_jmp(vm),
        Opcode::BackJmp => handlers::back_jmp(vm),
        Opcode::Pop | Opcode::PopElide => handlers::pop(vm),
        Opcode::PopUnwindHandler => handlers::pop_unwind_handler(vm),
        Opcode::AdditionAssignment => handlers::addition_assignment(vm),
        Opcode::SubtractionAssignment => handlers::subtraction_assignment(vm),
        Opcode::MultiplicationAssignment => handlers::multiplication_assignment(vm),
        Opcode::DivisionAssignment => handlers::division_assignment(vm),
        Opcode::RemainderAssignment => handlers::remainder_assignment(vm),
        Opcode::ExponentiationAssignment => handlers::exponentiation_assignment(vm),
        Opcode::LeftShiftAssignment => handlers::left_shift_assignment(vm),
        Opcode::RightShiftAssignment => handlers::right_shift_assignment(vm),
        Opcode::UnsignedRightShiftAssignment => handlers::unsigned_right_shift_assignment(vm),
        Opcode::BitwiseAndAssignment => handlers::bitwise_and_assignment(vm),
        Opcode::BitwiseOrAssignment => handlers::bitwise_or_assignment(vm),
        Opcode::BitwiseXorAssignment => handlers::bitwise_xor_assignment(vm),
        Opcode::LogicalAndAssignment => handlers::logical_and_assignment(vm),
        Opcode::LogicalOrAssignment => handlers::logical_or_assignment(vm),
        Opcode::LogicalNullishAssignment => handlers::logical_nullish_assignment(vm),
        Opcode::ConstructorCall => handlers::constructor_call(vm)?,
        Opcode::FunctionCall => handlers::function_call(vm)?,
        Opcode::GetThis => handlers::get_this(vm),
        Opcode::GetGlobalThis => handlers::get_global_this(vm),
        Opcode::EvaluateModule => handlers::evaluate_module(vm),
        Opcode::Try => handlers::try_block(vm),
        Opcode::Throw => handlers::throw(vm)?,
        Opcode::ReturnModule => handlers::return_module(vm),
        Opcode::Return => return handlers::return_(vm, frame_idx),
        Opcode::Less => handlers::less(vm),
        Opcode::LessEqual => handlers::less_equal(vm),
        Opcode::Greater => handlers::greater(vm),
        Opcode::GreaterEqual => handlers::greater_equal(vm),
        Opcode::StaticPropertyAccess => handlers::static_property_access(vm),
        Opcode::Equality => handlers::equality(vm),
        Opcode::Inequality => handlers::inequality(vm),
        Opcode::StrictEquality => handlers::strict_equality(vm),
        Opcode::StrictInequality => handlers::strict_inequality(vm),
        Opcode::Typeof => handlers::typeof_(vm),
        Opcode::PostfixIncrement | Opcode::PostfixDecrement => {
            handlers::postfix_increment_decrement(vm, opcode)
        }
        Opcode::Assignment => handlers::assignment(vm),
        Opcode::Void => handlers::void(vm),
        Opcode::ArrayLiteral => handlers::array_literal(vm),
        Opcode::ObjectLiteral => handlers::object_literal(vm),
        Opcode::ComputedPropertyAccess => handlers::computed_property_access(vm),
        Opcode::Continue => handlers::loop_continue(vm),
        Opcode::Break => handlers::loop_break(vm),
        Opcode::LoopStart => handlers::loop_start(vm),
        Opcode::LoopEnd => handlers::loop_end(vm),
        Opcode::ExportDefault => handlers::export_default(vm)?,
        Opcode::ToPrimitive => handlers::to_primitive(vm)?,
        Opcode::Debugger => handlers::debugger(vm),

        _ => unimplemented!("{:?}", opcode),
    }

    Ok(None)
}
