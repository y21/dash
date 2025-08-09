use std::mem;

use crate::dispatch::HandleResult;
use crate::frame::Frame;
use crate::framestack::FrameId;
use crate::localscope::LocalScope;
use crate::throw;
use crate::value::function::generator::{GeneratorIterator, GeneratorState};
use crate::value::function::native::CallContext;
use crate::value::function::{Function, FunctionKind};
use crate::value::object::{Object, OrdObject, PropertyValue};
use crate::value::propertykey::ToPropertyKey;
use crate::value::root_ext::RootErrExt;
use crate::value::{Root, Unrooted, Value, ValueContext};
use dash_middle::interner::sym;

use super::receiver_t;

/// Sets up the generator frame and does all the necessary setup, calls the closure which can then perform operations within that generator,
/// and after the closure returns, cleans up the frame, and creates a generator result value (or propagates an uncaught exception).
#[deny(
    clippy::question_mark_used,
    reason = "broken generator state needs to be fixed and `?` must not be used in that state"
)]
fn bootstrap_generator(
    scope: &mut LocalScope<'_>,
    this: Value,
    process: &dyn Fn(&mut LocalScope<'_>, Frame) -> Result<HandleResult, Unrooted>,
) -> Result<Value, Value> {
    #[expect(clippy::question_mark_used)]
    let generator = receiver_t::<GeneratorIterator>(scope, &this, "GeneratorIterator.prototype.next")?;

    let frame = {
        let (ip, old_stack, arguments, mut try_blocks, this) = match &mut *generator.state().borrow_mut() {
            GeneratorState::Finished => return create_generator_value(scope, true, None),
            GeneratorState::Running {
                ip,
                stack,
                arguments,
                try_blocks,
                this,
            } => (*ip, mem::take(stack), arguments.take(), mem::take(try_blocks), *this),
        };

        for tb in &mut try_blocks {
            // frame_idx is 0-based, but we haven't pushed the frame yet and will later, which will make this correct.
            tb.frame_idx = FrameId(scope.frames.len());
        }

        let function = generator.function();
        let function = match function.extract::<Function>(scope).map(|fun| fun.kind()) {
            Some(FunctionKind::Generator(generator)) => &generator.function,
            Some(FunctionKind::Async(fun)) => &fun.inner.function,
            _ => throw!(scope, TypeError, "Incompatible generator function"),
        };

        let current_sp = scope.active_sp();
        #[expect(clippy::question_mark_used)] // FIXME: is this correct? the generator state is left empty
        scope.try_extend_stack(old_stack).root_err(scope)?;
        scope.try_blocks.extend(try_blocks);

        let mut frame = Frame::from_function(this, function, None, false, arguments);
        frame.ip = ip;
        frame.sp = current_sp;

        if !generator.did_run() {
            // If it hasn't run before, do the stack space management initially (push undefined values for locals)
            // We only want to do this if the generator hasn't run already, because the locals are already in `old_stack`
            scope.pad_stack_for_frame(&frame);
        }

        frame
    };

    let result = match process(scope, frame) {
        Ok(res) => res,
        Err(err) => {
            generator.state().replace(GeneratorState::Finished);
            return Err(err.root(scope));
        }
    };

    match result {
        HandleResult::Return(value) => {
            generator.state().replace(GeneratorState::Finished);
            let value = value.root(scope);

            create_generator_value(scope, true, Some(value))
        }
        HandleResult::Yield(value) | HandleResult::Await(value) => {
            // Async functions are desugared to generators, so `await` is treated equivalent to `yield`, for now...
            let value = value.root(scope);

            let frame_idx = scope.frames.current_id();
            let frame = scope.pop_frame();
            let stack = scope.drain_stack(frame.sp.0 as usize..).collect::<Vec<_>>();

            // Save any try blocks part of this frame
            let frame_try_blocks = scope
                .try_blocks
                .iter()
                .rev()
                .take_while(|b| b.frame_idx == frame_idx)
                .count();

            let total_try_blocks = scope.try_blocks.len();
            let try_blocks = scope
                .try_blocks
                .drain(total_try_blocks - frame_try_blocks..)
                .collect::<Vec<_>>();

            generator.state().replace(GeneratorState::Running {
                ip: frame.ip,
                stack,
                arguments: frame.arguments,
                try_blocks,
                this: frame.this,
            });

            create_generator_value(scope, false, Some(value))
        }
    }
}

pub fn next(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let val = cx.args.first().unwrap_or_undefined();
    bootstrap_generator(scope, cx.this, &|scope, frame| {
        // We're going to resume the generator after having evaluated a `yield` expression,
        // which expects a value to be on the stack (the resumed value)
        scope.stack.push(val);
        scope.execute_frame_raw(frame)
    })
}

pub fn throw(cx: CallContext, scope: &mut LocalScope<'_>) -> Result<Value, Value> {
    let val = cx.args.first().unwrap_or_undefined();
    bootstrap_generator(scope, cx.this, &|scope, frame| {
        let fp = FrameId(scope.frames.len());
        scope.try_push_frame(frame)?;
        scope.handle_rt_error(val.into(), fp)?; // FIXME: is this `?` fine?
        scope.handle_instruction_loop()
    })
}

fn create_generator_value(scope: &mut LocalScope, done: bool, value: Option<Value>) -> Result<Value, Value> {
    let obj = OrdObject::new(scope);
    obj.set_property(
        sym::done.to_key(scope),
        PropertyValue::static_default(done.into()),
        scope,
    )?;
    obj.set_property(
        sym::value.to_key(scope),
        PropertyValue::static_default(value.unwrap_or_undefined()),
        scope,
    )?;
    Ok(scope.register(obj).into())
}
