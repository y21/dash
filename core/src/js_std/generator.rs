use crate::gc::Handle;
use crate::vm::dispatch::DispatchResult;
use crate::vm::frame::Frame;
use crate::vm::value::function::FunctionKind;
use crate::vm::value::generator::GeneratorState;
use crate::vm::value::object::Object;
use crate::vm::value::ValueKind;
use crate::vm::value::{function::CallContext, Value};
use crate::vm::VMError;

/// Implements the next function on generator iterators
pub fn next(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    let this = ctx.receiver.unwrap();

    let frame = {
        let mut this_ref = unsafe { this.borrow_mut_unbounded() };
        let this_gen_iter = this_ref
            .as_object_mut()
            .and_then(Object::as_generator_iterator_mut)
            .unwrap();

        // If [[finished]], immediately return value=undefined, done=true
        let (ip, stack) = match this_gen_iter.state {
            GeneratorState::Finished => {
                let mut result = ctx.vm.create_object();
                result.set_property(
                    "value",
                    Value::new(ValueKind::Undefined).into_handle(ctx.vm),
                );
                result.set_property("done", Value::from(true).into_handle(ctx.vm));
                return Ok(result.into_handle(ctx.vm));
            }
            GeneratorState::Running { ip, ref mut stack } => {
                let old_stack = std::mem::take(stack);
                (ip, old_stack)
            }
        };

        let func = unsafe { this_gen_iter.function.borrow_unbounded() };
        let func_ref = func.as_function().unwrap();
        let buffer = match func_ref {
            FunctionKind::Closure(c) => c.func.buffer.clone(),
            FunctionKind::User(u) => u.buffer.clone(),
            _ => unreachable!(),
        };

        let current_sp = ctx.vm.stack.len();

        for value in stack {
            ctx.vm.stack.push(value);
        }

        Frame {
            buffer: buffer.into(),
            sp: current_sp,
            iterator_caller: Some(Handle::clone(&this)),
            func: Handle::clone(&this_gen_iter.function),
            ip,
        }
    };

    let result = ctx
        .vm
        .execute_frame(frame, false)
        .map_err(VMError::into_value)?;

    let mut this_ref = unsafe { this.borrow_mut_unbounded() };
    let iter = this_ref
        .as_object_mut()
        .and_then(Object::as_generator_iterator_mut)
        .unwrap();

    match result {
        DispatchResult::Return(r) => {
            iter.state = GeneratorState::Finished;

            let mut result = ctx.vm.create_object();
            result.set_property("value", Value::unwrap_or_undefined(r, ctx.vm));
            result.set_property("done", Value::from(true).into_handle(ctx.vm));
            Ok(result.into_handle(ctx.vm))
        }
        DispatchResult::Yield(r) => {
            let frame = ctx.vm.frames.pop();

            // Stack capturing etc
            let stack = unsafe { ctx.vm.stack.drain_from_unchecked(frame.sp) };

            iter.state = GeneratorState::Running {
                ip: frame.ip + 1,
                stack,
            };

            let mut result = ctx.vm.create_object();
            result.set_property("value", Value::unwrap_or_undefined(r, ctx.vm));
            result.set_property("done", Value::from(false).into_handle(ctx.vm));
            Ok(result.into_handle(ctx.vm))
        }
    }
}
