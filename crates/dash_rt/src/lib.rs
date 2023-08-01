use std::future::Future;

use dash_vm::gc::persistent::Persistent;
use dash_vm::localscope::LocalScope;
use dash_vm::value::function::native::CallContext;
use dash_vm::value::promise::Promise;
use dash_vm::value::Value;
use dash_vm::PromiseAction;
use event::EventMessage;
use state::State;

pub mod active_tasks;
pub mod event;
pub mod module;
pub mod runtime;
pub mod state;

// TODO: move elsewhere? util module?
pub fn wrap_async<Fut, Fun, T, E>(cx: CallContext, fut: Fut, convert: Fun) -> Result<Value, Value>
where
    Fut: Future<Output = Result<T, E>> + Send + 'static,
    Fun: FnOnce(&mut LocalScope, Result<T, E>) -> Result<Value, Value> + Send + Sync + 'static,
    T: Send + Sync + 'static,
    E: Send + Sync + 'static,
{
    let event_tx = State::from_vm(cx.scope).event_sender();

    let promise = {
        let promise = Promise::new(cx.scope);
        cx.scope.register(promise)
    };

    let (promise_id, rt) = {
        let state = State::from_vm(cx.scope);
        let persistent_promise = Persistent::new(promise.clone());
        let pid = state.add_pending_promise(persistent_promise);
        let rt = state.rt_handle();
        (pid, rt)
    };

    rt.spawn(async move {
        let data = fut.await;

        event_tx.send(EventMessage::ScheduleCallback(Box::new(move |rt| {
            let promise = State::from_vm(rt.vm()).take_promise(promise_id);
            let mut scope = rt.vm_mut().scope();
            let promise = promise.as_any().downcast_ref::<Promise>().unwrap();

            let data = convert(&mut scope, data);

            let (arg, action) = match data {
                Ok(ok) => (ok, PromiseAction::Resolve),
                Err(err) => (err, PromiseAction::Reject),
            };
            scope.drive_promise(action, promise, vec![arg]);
            scope.process_async_tasks();
        })));
    });

    Ok(Value::Object(promise))
}
