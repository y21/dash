use dash_middle::compiler::StaticImportKind;
use dash_middle::util::SharedOnce;
use dash_rt::event::EventMessage;
use dash_rt::module::ModuleLoader;
use dash_rt::state::State;
use dash_vm::gc::trace::{Trace, TraceCtxt};
use dash_vm::js_std::receiver_t;
use dash_vm::localscope::LocalScope;
use dash_vm::value::error::Error;
use dash_vm::value::function::native::CallContext;
use dash_vm::value::function::{Function, FunctionKind};
use dash_vm::value::object::{NamedObject, Object, PropertyValue};
use dash_vm::value::promise::Promise;
use dash_vm::value::propertykey::ToPropertyKey;
use dash_vm::value::string::JsString;
use dash_vm::value::{ExceptionContext, Unpack, Value, ValueKind};
use dash_vm::{PromiseAction, Vm, delegate, extract, throw};
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

    let promise = cx.scope.mk_promise();

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

                    obj.set_property(text.to_key(&mut sc), PropertyValue::static_default(text_fun), &mut sc)
                        .unwrap();

                    (Value::object(sc.register(obj)), PromiseAction::Resolve)
                }
                Err(err) => {
                    let err = Error::new(&mut sc, err.to_string());
                    (Value::object(sc.register(err)), PromiseAction::Reject)
                }
            };

            sc.drive_promise(action, promise, [req].into());
            sc.process_async_tasks();
        })));
    });

    Ok(Value::object(promise))
}

fn http_response_text(cx: CallContext) -> Result<Value, Value> {
    let this = receiver_t::<HttpResponse>(cx.scope, &cx.this, "Response.prototype.text")?;

    let (rt, event_tx) = {
        let state = State::from_vm_mut(cx.scope);
        let etx = state.event_sender();
        let rt = state.rt_handle();
        (rt, etx)
    };

    let response = this
        .response
        .try_take()
        .or_err(cx.scope, "HTTP Response already consumed")?;

    let promise = cx.scope.mk_promise();

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

            sc.drive_promise(action, promise, [value].into());
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
