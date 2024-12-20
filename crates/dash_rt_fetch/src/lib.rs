use dash_middle::compiler::StaticImportKind;
use dash_middle::util::SharedOnce;
use dash_rt::event::EventMessage;
use dash_rt::module::ModuleLoader;
use dash_rt::state::State;
use dash_vm::gc::trace::{Trace, TraceCtxt};
use dash_vm::localscope::LocalScope;
use dash_vm::value::error::Error;
use dash_vm::value::function::native::CallContext;
use dash_vm::value::function::{Function, FunctionKind};
use dash_vm::value::object::{NamedObject, Object, PropertyValue};
use dash_vm::value::promise::Promise;
use dash_vm::value::string::JsString;
use dash_vm::value::{Unpack, Value, ValueKind};
use dash_vm::{delegate, extract, throw, PromiseAction, Vm};
use once_cell::sync::Lazy;
use reqwest::{Client, Method};

#[derive(Debug)]
pub struct FetchModule;

impl ModuleLoader for FetchModule {
    fn import(
        &self,
        sc: &mut LocalScope,
        _import_ty: StaticImportKind,
        path: JsString,
    ) -> Result<Option<Value>, Value> {
        if path.res(sc) != "@std/fetch" {
            return Ok(None);
        }

        init_module(sc).map(Some)
    }
}

static REQWEST: Lazy<Client> = Lazy::new(Client::new);

pub fn init_module(sc: &mut LocalScope<'_>) -> Result<Value, Value> {
    let name = sc.intern("fetch");
    let fun = Function::new(sc, Some(name.into()), FunctionKind::Native(fetch));
    let fun = sc.register(fun);

    Ok(Value::object(fun))
}

fn fetch(cx: CallContext) -> Result<Value, Value> {
    let url = match cx.args.first().unpack() {
        Some(ValueKind::String(url)) => url.res(cx.scope).to_owned(),
        _ => throw!(cx.scope, TypeError, "Expected a string as the first argument"),
    };

    let (rt, event_tx) = {
        let state = State::from_vm_mut(cx.scope);
        let etx = state.event_sender();
        let rt = state.rt_handle();
        (rt, etx)
    };

    let promise = Promise::new(cx.scope);
    let promise = cx.scope.register(promise);

    let promise_id = State::from_vm_mut(cx.scope).add_pending_promise(promise);

    rt.spawn(async move {
        let req = REQWEST
            .request(Method::GET, url)
            .header("User-Agent", "dash-rt-fetch (https://github.com/y21/dash)")
            .send()
            .await;

        event_tx.send(EventMessage::ScheduleCallback(Box::new(move |rt| {
            let mut sc = rt.vm_mut().scope();
            let promise = State::from_vm_mut(&mut sc).take_promise(promise_id);
            let promise = promise.extract::<Promise>(&sc).unwrap();

            let (req, action) = match req {
                Ok(resp) => {
                    let obj = HttpResponse::new(resp, &sc);
                    let text = sc.intern("text");
                    let text_fun = Function::new(&sc, Some(text.into()), FunctionKind::Native(http_response_text));
                    let text_fun = Value::object(sc.register(text_fun));

                    obj.set_property(&mut sc, text.into(), PropertyValue::static_default(text_fun))
                        .unwrap();

                    (Value::object(sc.register(obj)), PromiseAction::Resolve)
                }
                Err(err) => {
                    let err = Error::new(&mut sc, err.to_string());
                    (Value::object(sc.register(err)), PromiseAction::Reject)
                }
            };

            sc.drive_promise(action, promise, vec![req]);
            sc.process_async_tasks();
        })));
    });

    Ok(Value::object(promise))
}

fn http_response_text(cx: CallContext) -> Result<Value, Value> {
    let this = match cx.this.unpack() {
        ValueKind::Object(obj) => obj,
        _ => throw!(cx.scope, TypeError, "Expected a this value"),
    };
    let this = match this.extract::<HttpResponse>(cx.scope) {
        Some(resp) => resp,
        None => throw!(cx.scope, TypeError, "Invalid receiver, expected HttpResponse"),
    };

    let (rt, event_tx) = {
        let state = State::from_vm_mut(cx.scope);
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

    let promise_id = State::from_vm_mut(cx.scope).add_pending_promise(promise);

    rt.spawn(async move {
        let text = response.text().await;

        event_tx.send(EventMessage::ScheduleCallback(Box::new(move |rt| {
            let mut sc = rt.vm_mut().scope();
            let promise = State::from_vm_mut(&mut sc).take_promise(promise_id);
            let promise = promise.extract::<Promise>(&sc).unwrap();

            let (value, action) = match text {
                Ok(text) => {
                    let text = Value::string(sc.intern(text.as_ref()).into());
                    (text, PromiseAction::Resolve)
                }
                Err(err) => {
                    let err = Error::new(&mut sc, err.to_string());
                    let err = Value::object(sc.register(err));
                    (err, PromiseAction::Reject)
                }
            };

            sc.drive_promise(action, promise, vec![value]);
            sc.process_async_tasks();
        })));
    });

    Ok(Value::object(promise))
}

#[derive(Debug)]
struct HttpResponse {
    response: SharedOnce<reqwest::Response>,
    obj: NamedObject,
}

impl HttpResponse {
    pub fn new(response: reqwest::Response, vm: &Vm) -> Self {
        Self {
            response: SharedOnce::new(response),
            obj: NamedObject::new(vm),
        }
    }
}

unsafe impl Trace for HttpResponse {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        self.obj.trace(cx);
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
        apply,
        own_keys
    );

    extract!(self);
}
