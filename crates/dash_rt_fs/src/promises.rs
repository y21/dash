use std::rc::Rc;

use dash_rt::event::EventMessage;
use dash_rt::state::State;
use dash_vm::gc::persistent::Persistent;
use dash_vm::local::LocalScope;
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

    let event_tx = State::from_vm(cx.scope).event_sender();

    let promise = Promise::new(cx.scope);
    let promise = cx.scope.register(promise);

    let state = State::from_vm(cx.scope);
    let persistent_promise = Persistent::new(promise.clone());
    // let id = state.add_pending_promise(Value::Object(promise.clone()));
    // TODO: somehow prevent promise from being GC'd

    tokio::task::spawn_blocking(move || {
        let path = path;
        let event_tx = event_tx;

        // TODO: no unwrap
        let file = std::fs::read_to_string(path).unwrap();

        event_tx.send(EventMessage::ScheduleCallback(Box::new(move |rt| {
            // let promise = State::from_vm(rt.vm()).take_promise(id);
            // let mut scope = LocalScope::new(rt.vm_mut());
            // let promise = match &promise {
            //     Value::Object(o) => o.as_any().downcast_ref::<Promise>().unwrap(),
            //     _ => unreachable!(),
            // };
            // scope.drive_promise(PromiseAction::Resolve, promise, vec![Value::String(file.into())]);
            // scope.process_async_tasks();
        })));
    });
    Ok(Value::Object(promise))
}
