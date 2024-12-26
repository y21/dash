use std::mem;

use crate::dispatch::HandleResult;
use crate::frame::{Frame, This};
use crate::localscope::LocalScope;
use crate::throw;
use crate::value::function::generator::{GeneratorIterator, GeneratorState};
use crate::value::function::native::CallContext;
use crate::value::function::{Function, FunctionKind};
use crate::value::object::{NamedObject, Object, PropertyValue};
use crate::value::root_ext::RootErrExt;
use crate::value::{Root, Unpack, Value, ValueContext};
use dash_middle::interner::sym;

pub fn next(cx: CallContext) -> Result<Value, Value> {
    let generator = cx.this.unpack();
    let generator = match generator.downcast_ref::<GeneratorIterator>(cx.scope) {
        Some(it) => it,
        None => throw!(cx.scope, TypeError, "Incompatible receiver"),
    };
    let arg = cx.args.first().unwrap_or_undefined();
    let frame = {
        let (ip, old_stack, arguments, try_blocks) = match &mut *generator.state().borrow_mut() {
            GeneratorState::Finished => return create_generator_value(cx.scope, true, None),
            GeneratorState::Running {
                ip,
                stack,
                arguments,
                try_blocks,
            } => (*ip, mem::take(stack), arguments.take(), mem::take(try_blocks)),
        };

        let function = generator.function();
        let function = match function.extract::<Function>(cx.scope).map(|fun| fun.kind()) {
            Some(FunctionKind::Generator(gen)) => &gen.function,
            Some(FunctionKind::Async(fun)) => &fun.inner.function,
            _ => throw!(cx.scope, TypeError, "Incompatible generator function"),
        };

        cx.scope.try_blocks.extend(try_blocks);
        let current_sp = cx.scope.stack_size();
        cx.scope.try_extend_stack(old_stack).root_err(cx.scope)?;

        let mut frame = Frame::from_function(This::Default, function, None, false, arguments);
        frame.set_ip(ip);
        frame.set_sp(current_sp);

        if !generator.did_run() {
            // If it hasn't run before, do the stack space management initially (push undefined values for locals)
            // We only want to do this if the generator hasn't run already, because the locals are already in `old_stack`
            cx.scope.pad_stack_for_frame(&frame);
        } else {
            // Generator did run before. Push the yielded value onto the stack, which will be what the yield expression
            // evaluates to.
            cx.scope.stack.push(arg);
        }

        frame
    };

    // Generators work a bit different from normal functions, so we do the stack padding management ourselves here
    let result = match cx.scope.execute_frame_raw(frame) {
        Ok(v) => v,
        // TODO: this should not early return because we never reset the temporarily broken generator state
        Err(v) => return Err(v.root(cx.scope)),
    };

    match result {
        HandleResult::Return(value) => {
            generator.state().replace(GeneratorState::Finished);
            let value = value.root(cx.scope);

            create_generator_value(cx.scope, true, Some(value))
        }
        HandleResult::Yield(value) | HandleResult::Await(value) => {
            // Async functions are desugared to generators, so `await` is treated equivalent to `yield`, for now...
            let value = value.root(cx.scope);

            let fp = cx.scope.frames.len();
            let frame = cx.scope.pop_frame().expect("Generator frame is missing");
            let stack = cx.scope.drain_stack(frame.sp..).collect::<Vec<_>>();

            // Save any try blocks part of this frame
            let frame_try_blocks = cx
                .scope
                .try_blocks
                .iter()
                .rev()
                .take_while(|b| b.frame_ip == fp)
                .count();

            let total_try_blocks = cx.scope.try_blocks.len();
            let try_blocks = cx
                .scope
                .try_blocks
                .drain(total_try_blocks - frame_try_blocks..)
                .collect::<Vec<_>>();

            generator.state().replace(GeneratorState::Running {
                ip: frame.ip,
                stack,
                arguments: frame.arguments,
                try_blocks,
            });

            create_generator_value(cx.scope, false, Some(value))
        }
    }
}

fn create_generator_value(scope: &mut LocalScope, done: bool, value: Option<Value>) -> Result<Value, Value> {
    let obj = NamedObject::new(scope);
    obj.set_property(scope, sym::done.into(), PropertyValue::static_default(done.into()))?;
    obj.set_property(
        scope,
        sym::value.into(),
        PropertyValue::static_default(value.unwrap_or_undefined()),
    )?;
    Ok(scope.register(obj).into())
}
