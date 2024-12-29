use std::future::Future;

use dash_vm::PromiseAction;
use dash_vm::localscope::LocalScope;
use dash_vm::value::Value;
use dash_vm::value::function::native::CallContext;
use dash_vm::value::promise::Promise;
use event::EventMessage;
use inspect::InspectOptions;
use state::State;

pub mod active_tasks;
pub mod event;
pub mod inspect;
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

    let promise = cx.scope.mk_promise();

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
            let promise = promise.extract::<Promise>(&scope).unwrap();

            let data = convert(&mut scope, data);

            let (arg, action) = match data {
                Ok(ok) => (ok, PromiseAction::Resolve),
                Err(err) => (err, PromiseAction::Reject),
            };
            scope.drive_promise(action, promise, [arg].into());
            scope.process_async_tasks();
        })));
    });

    Ok(Value::object(promise))
}

pub fn format_value(value: Value, scope: &mut LocalScope) -> Result<String, Value> {
    inspect::inspect(value, scope, InspectOptions::default())
}
