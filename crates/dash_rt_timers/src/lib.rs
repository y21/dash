use std::sync::Arc;
use std::time::Duration;

use dash_middle::compiler::StaticImportKind;
use dash_middle::util::ThreadSafeStorage;
use dash_rt::event::EventMessage;
use dash_rt::module::ModuleLoader;
use dash_rt::state::State;
use dash_vm::gc::persistent::Persistent;
use dash_vm::localscope::LocalScope;
use dash_vm::throw;
use dash_vm::value::function::native::CallContext;
use dash_vm::value::function::{Function, FunctionKind};
use dash_vm::value::object::{NamedObject, Object, PropertyValue};
use dash_vm::value::ops::conversions::ValueConversion;
use dash_vm::value::string::JsString;
use dash_vm::value::{Unpack, Value, ValueKind};

#[derive(Debug)]
pub struct TimersModule;

impl ModuleLoader for TimersModule {
    fn import(
        &self,
        sc: &mut LocalScope,
        _import_ty: StaticImportKind,
        path: JsString,
    ) -> Result<Option<Value>, Value> {
        if path.res(sc) == "@std/timers" {
            let obj = NamedObject::new(sc);

            let name = sc.intern("setTimeout");
            let set_timeout = Function::new(sc, Some(name.into()), FunctionKind::Native(set_timeout));
            let set_timeout = Value::object(sc.register(set_timeout));

            obj.set_property(sc, name.into(), PropertyValue::static_default(set_timeout))?;

            Ok(Some(Value::object(sc.register(obj))))
        } else {
            Ok(None)
        }
    }
}

fn set_timeout(cx: CallContext) -> Result<Value, Value> {
    let callback = match cx.args.first().unpack() {
        Some(ValueKind::Object(cb)) => cb,
        _ => throw!(cx.scope, TypeError, "missing callback function argument"),
    };

    let callback = Arc::new(ThreadSafeStorage::new(Persistent::new(cx.scope, callback)));

    let delay = match cx.args.get(1) {
        Some(delay) => delay.to_int32(cx.scope)? as u64,
        None => throw!(cx.scope, TypeError, "Missing delay argument"),
    };

    let state = State::from_vm_mut(cx.scope);
    let tx = state.event_sender();
    let tid = state.tasks.add();

    state.rt_handle().spawn(async move {
        let tx2 = tx.clone();
        tokio::time::sleep(Duration::from_millis(delay)).await;

        tx.send(EventMessage::ScheduleCallback(Box::new(move |rt| {
            let mut sc = rt.vm_mut().scope();
            let callback = callback.get();

            if let Err(err) = callback.apply(&mut sc, Value::undefined(), Vec::new()) {
                eprintln!("Unhandled error in timer callback: {err:?}");
            }

            tx2.send(EventMessage::RemoveTask(tid));
        })));
    });

    Ok(Value::undefined())
}
