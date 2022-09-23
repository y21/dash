use std::future::Future;

use dash_rt::event::EventMessage;
use dash_rt::state::State;
use dash_vm::gc::persistent::Persistent;
use dash_vm::local::LocalScope;
use dash_vm::value::error::Error;
use dash_vm::value::function::native::CallContext;
use dash_vm::value::function::Function;
use dash_vm::value::function::FunctionKind;
use dash_vm::value::object::NamedObject;
use dash_vm::value::object::Object;
use dash_vm::value::object::PropertyKey;
use dash_vm::value::object::PropertyValue;
use dash_vm::value::ops::abstractions::conversions::ValueConversion;
use dash_vm::value::promise::Promise;
use dash_vm::value::Value;
use dash_vm::value::ValueContext;
use dash_vm::PromiseAction;

pub fn init_module(sc: &mut LocalScope) -> Option<Value> {
    let read_file_value = Function::new(sc, Some("readFile".into()), FunctionKind::Native(read_file));
    let read_file_value = sc.register(read_file_value);

    let module = NamedObject::new(sc);
    module
        .set_property(
            sc,
            PropertyKey::String("readFile".into()),
            PropertyValue::static_default(Value::Object(read_file_value)),
        )
        .unwrap();

    Some(Value::Object(sc.register(module)))
}

fn read_file(cx: CallContext) -> Result<Value, Value> {
    let path = cx.args.first().unwrap_or_undefined().to_string(cx.scope)?;
    let path = ToString::to_string(&path);

    handle_async_fs(cx, tokio::fs::read_to_string(path), |sc, res| match res {
        Ok(s) => Ok(Value::String(s.into())),
        Err(e) => {
            let err = Error::new(sc, e.to_string());
            Err(Value::Object(sc.register(err)))
        }
    })
}

fn handle_async_fs<Fut, Fun, T, E>(cx: CallContext, fut: Fut, convert: Fun) -> Result<Value, Value>
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

    let promise_id = {
        let state = State::from_vm(cx.scope);
        let persistent_promise = Persistent::new(promise.clone());
        state.add_pending_promise(persistent_promise)
    };

    tokio::spawn(async move {
        let data = fut.await;

        event_tx.send(EventMessage::ScheduleCallback(Box::new(move |rt| {
            let promise = State::from_vm(rt.vm()).take_promise(promise_id);
            let mut scope = LocalScope::new(rt.vm_mut());
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
