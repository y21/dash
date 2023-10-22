use std::cell::OnceCell;
use std::future::Future;
use std::rc::Rc;

use dash_compiler::FunctionCompiler;
use dash_middle::compiler::CompileResult;
use dash_middle::interner::StringInterner;
use dash_vm::frame::Exports;
use dash_vm::frame::Frame;
use dash_vm::gc::persistent::Persistent;
use dash_vm::localscope::LocalScope;
use dash_vm::value::function::native::CallContext;
use dash_vm::value::ops::abstractions::conversions::ValueConversion;
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
        let persistent_promise = Persistent::new(cx.scope, promise.clone());
        let state = State::from_vm(cx.scope);
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

pub fn format_value(value: Value, scope: &mut LocalScope) -> Result<Rc<str>, Value> {
    thread_local! {
        // Cache bytecode so we can avoid recompiling it every time
        // We can be even smarter if we need to -- cache the whole value at callsite
        static INSPECT_BC: OnceCell<CompileResult> = const { OnceCell::new() };
    }

    let inspect_bc = INSPECT_BC.with(|tls| {
        let inspect = tls.get_or_init(|| {
            FunctionCompiler::compile_str(
                // TODO: can reuse a string interner if worth it
                &mut StringInterner::new(),
                include_str!("../js/inspect.js"),
                Default::default(),
            )
            .unwrap()
        });
        inspect.clone()
    });

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
        .to_string(scope)
        .unwrap();

    Ok(result)
}
