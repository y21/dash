use dash_middle::compiler::StaticImportKind;
use dash_middle::util::SharedOnce;
use dash_rt::event::EventMessage;
use dash_rt::module::ModuleLoader;
use dash_rt::state::State;
use dash_vm::delegate;
use dash_vm::gc::persistent::Persistent;
use dash_vm::gc::trace::Trace;
use dash_vm::localscope::LocalScope;
use dash_vm::throw;
use dash_vm::value::error::Error;
use dash_vm::value::function::native::CallContext;
use dash_vm::value::function::Function;
use dash_vm::value::function::FunctionKind;
use dash_vm::value::object::NamedObject;
use dash_vm::value::object::Object;
use dash_vm::value::object::PropertyKey;
use dash_vm::value::object::PropertyValue;
use dash_vm::value::promise::Promise;
use dash_vm::value::Value;
use dash_vm::PromiseAction;
use dash_vm::Vm;
use once_cell::sync::Lazy;
use reqwest::Client;
use reqwest::Method;

#[derive(Debug)]
pub struct FetchModule;

impl ModuleLoader for FetchModule {
    fn import(&self, sc: &mut LocalScope, _import_ty: StaticImportKind, path: &str) -> Result<Option<Value>, Value> {
        if path != "@std/fetch" {
            return Ok(None);
        }

        let fun = Function::new(sc, Some("fetch".into()), FunctionKind::Native(fetch));
        let fun = sc.register(fun);

        Ok(Some(Value::Object(fun)))
    }
}

static REQWEST: Lazy<Client> = Lazy::new(Client::new);

fn fetch(cx: CallContext) -> Result<Value, Value> {
    let url = match cx.args.first() {
        Some(Value::String(url)) => url.to_string(),
        _ => throw!(cx.scope, TypeError, "Expected a string as the first argument"),
    };

    let (rt, event_tx) = {
        let state = State::from_vm(cx.scope);
        let etx = state.event_sender();
        let rt = state.rt_handle();
        (rt, etx)
    };

    let promise = Promise::new(cx.scope);
    let promise = cx.scope.register(promise);

    let promise_id = {
        let persistent_promise = Persistent::new(promise.clone());
        State::from_vm(cx.scope).add_pending_promise(persistent_promise)
    };

    rt.spawn(async move {
        let req = REQWEST
            .request(Method::GET, url)
            .header("User-Agent", "dash-rt-fetch (https://github.com/y21/dash)")
            .send()
            .await;

        event_tx.send(EventMessage::ScheduleCallback(Box::new(move |rt| {
            let mut sc = rt.vm_mut().scope();
            let promise = State::from_vm(&sc).take_promise(promise_id);
            let promise = promise.as_any().downcast_ref::<Promise>().unwrap();

            let (req, action) = match req {
                Ok(resp) => {
                    let obj = HttpResponse::new(resp, &mut sc);
                    let text_fun =
                        Function::new(&mut sc, Some("text".into()), FunctionKind::Native(http_response_text));
                    let text_fun = Value::Object(sc.register(text_fun));

                    obj.set_property(
                        &mut sc,
                        PropertyKey::String("text".into()),
                        PropertyValue::static_default(text_fun),
                    )
                    .unwrap();

                    (Value::Object(sc.register(obj)), PromiseAction::Resolve)
                }
                Err(err) => {
                    let err = Error::new(&mut sc, err.to_string());
                    (Value::Object(sc.register(err)), PromiseAction::Reject)
                }
            };

            sc.drive_promise(action, promise, vec![req]);
            sc.process_async_tasks();
        })));
    });

    Ok(Value::Object(promise))
}

fn http_response_text(cx: CallContext) -> Result<Value, Value> {
    let this = match &cx.this {
        Value::Object(obj) => obj,
        _ => throw!(cx.scope, TypeError, "Expected a this value"),
    };
    let this = match this.as_any().downcast_ref::<HttpResponse>() {
        Some(resp) => resp,
        None => throw!(cx.scope, TypeError, "Invalid receiver, expected HttpResponse"),
    };

    let (rt, event_tx) = {
        let state = State::from_vm(cx.scope);
        let etx = state.event_sender();
        let rt = state.rt_handle();
        (rt, etx)
    };

    let response = match this.response.try_take() {
        Some(response) => response,
        None => throw!(cx.scope, Error, "HTTP Response already consumed"),
    };

    let promise = Promise::new(cx.scope);
    let promise = cx.scope.register(promise);

    let promise_id = {
        let persistent_promise = Persistent::new(promise.clone());
        State::from_vm(cx.scope).add_pending_promise(persistent_promise)
    };

    rt.spawn(async move {
        let text = response.text().await;

        event_tx.send(EventMessage::ScheduleCallback(Box::new(move |rt| {
            let mut sc = rt.vm_mut().scope();
            let promise = State::from_vm(&sc).take_promise(promise_id);
            let promise = promise.as_any().downcast_ref::<Promise>().unwrap();

            let (value, action) = match text {
                Ok(text) => {
                    let text = Value::String(text.into());
                    (text, PromiseAction::Resolve)
                }
                Err(err) => {
                    let err = Error::new(&mut sc, err.to_string());
                    let err = Value::Object(sc.register(err));
                    (err, PromiseAction::Reject)
                }
            };

            sc.drive_promise(action, promise, vec![value]);
            sc.process_async_tasks();
        })));
    });

    Ok(Value::Object(promise))
}

#[derive(Debug)]
struct HttpResponse {
    response: SharedOnce<reqwest::Response>,
    obj: NamedObject,
}

impl HttpResponse {
    pub fn new(response: reqwest::Response, vm: &mut Vm) -> Self {
        Self {
            response: SharedOnce::new(response),
            obj: NamedObject::new(vm),
        }
    }
}

unsafe impl Trace for HttpResponse {
    fn trace(&self) {
        self.obj.trace();
    }
}

impl Object for HttpResponse {
    delegate!(
        obj,
        get_own_property_descriptor,
        get_property,
        get_property_descriptor,
        set_property,
        delete_property,
        set_prototype,
        get_prototype,
        as_any,
        apply,
        own_keys
    );
}
