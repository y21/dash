use std::cell::OnceCell;
use std::future::Future;

use dash_compiler::FunctionCompiler;
use dash_middle::compiler::CompileResult;
use dash_vm::frame::{Exports, Frame};
use dash_vm::localscope::LocalScope;
use dash_vm::value::function::native::CallContext;
use dash_vm::value::object::Object;
use dash_vm::value::ops::conversions::ValueConversion;
use dash_vm::value::promise::Promise;
use dash_vm::value::{Root, Value};
use dash_vm::PromiseAction;
use event::EventMessage;
use state::State;

pub mod active_tasks;
pub mod event;
pub mod module;
pub mod runtime;
pub mod state;
pub mod typemap;

// TODO: move elsewhere? util module?
pub fn wrap_async<Fut, Fun, T, E>(cx: CallContext, fut: Fut, convert: Fun) -> Result<Value, Value>
where
    Fut: Future<Output = Result<T, E>> + Send + 'static,
    Fun: FnOnce(&mut LocalScope, Result<T, E>) -> Result<Value, Value> + Send + Sync + 'static,
    T: Send + Sync + 'static,
    E: Send + Sync + 'static,
{
    let event_tx = State::from_vm_mut(cx.scope).event_sender();

    let promise = {
        let promise = Promise::new(cx.scope);
        cx.scope.register(promise)
    };

    let (promise_id, rt) = {
        let state = State::from_vm_mut(cx.scope);
        let pid = state.add_pending_promise(promise);
        let rt = state.rt_handle();
        (pid, rt)
    };

    rt.spawn(async move {
        let data = fut.await;

        event_tx.send(EventMessage::ScheduleCallback(Box::new(move |rt| {
            let promise = State::from_vm_mut(rt.vm_mut()).take_promise(promise_id);
            let mut scope = rt.vm_mut().scope();
            let promise = promise.as_any(&scope).downcast_ref::<Promise>().unwrap();

            let data = convert(&mut scope, data);

            let (arg, action) = match data {
                Ok(ok) => (ok, PromiseAction::Resolve),
                Err(err) => (err, PromiseAction::Reject),
            };
            scope.drive_promise(action, promise, vec![arg]);
            scope.process_async_tasks();
        })));
    });

    Ok(Value::object(promise))
}

pub fn format_value<'s>(value: Value, scope: &'s mut LocalScope) -> Result<&'s str, Value> {
    let inspect_bc = FunctionCompiler::compile_str(
        &mut scope.interner,
        include_str!("../js/inspect.js"),
        Default::default(),
    )
    .unwrap();

    // TODO: we need to somehow add `CompileResult` as an external?

    let Exports {
        default: Some(inspect_fn),
        ..
    } = scope.execute_module(Frame::from_compile_result(inspect_bc)).unwrap()
    else {
        panic!("inspect module did not have a default export");
    };

    let result = inspect_fn
        .root(scope)
        .apply(scope, Value::undefined(), vec![value])
        .unwrap()
        .root(scope)
        .to_js_string(scope)
        .unwrap();

    Ok(result.res(scope))
}
