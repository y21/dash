use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use dash_middle::compiler::StaticImportKind;
use dash_middle::util::{SharedOnce, ThreadSafeStorage};
use dash_rt::event::EventMessage;
use dash_rt::module::ModuleLoader;
use dash_rt::state::State;
use dash_vm::gc::persistent::Persistent;
use dash_vm::gc::trace::{Trace, TraceCtxt};
use dash_vm::localscope::LocalScope;
use dash_vm::value::function::native::CallContext;
use dash_vm::value::function::{Function, FunctionKind};
use dash_vm::value::object::{NamedObject, Object, PropertyValue};
use dash_vm::value::ops::conversions::ValueConversion;
use dash_vm::value::root_ext::RootErrExt;
use dash_vm::value::string::JsString;
use dash_vm::value::{Unpack, Value, ValueContext, ValueKind};
use dash_vm::{delegate, extract, throw};
use hyper::Body;
use tokio::sync::oneshot;
use tokio::sync::oneshot::Sender;

#[derive(Debug)]
pub struct HttpModule;

impl ModuleLoader for HttpModule {
    fn import(
        &self,
        sc: &mut LocalScope,
        _import_ty: StaticImportKind,
        path: JsString,
    ) -> Result<Option<Value>, Value> {
        if path.res(sc) != "@std/http" {
            return Ok(None);
        }

        let module = NamedObject::new(sc);
        let listen = Function::new(sc, None, FunctionKind::Native(listen));
        let listen = sc.register(listen);
        let key = sc.intern("listen");
        module.set_property(sc, key.into(), PropertyValue::static_default(listen.into()))?;

        let module = sc.register(module);
        Ok(Some(module.into()))
    }
}

pub fn listen(cx: CallContext) -> Result<Value, Value> {
    let port = cx.args.first().unwrap_or_undefined().to_int32(cx.scope)?;
    let cb = match cx.args.get(1).unpack() {
        Some(ValueKind::Object(o)) => o,
        _ => throw!(cx.scope, TypeError, "Expected callback function as second argument"),
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], port as u16));

    let (task_id, event_tx, rt) = {
        let state = State::from_vm_mut(cx.scope);
        let task_id = state.tasks.add();
        let event_tx = state.event_sender();
        let rt = state.rt_handle();
        (task_id, event_tx, rt)
    };

    let cb_ref = {
        let p = Persistent::new(cx.scope, cb);
        Arc::new(ThreadSafeStorage::new(p))
    };

    rt.spawn(async move {
        let service_etx = event_tx.clone();
        let cb = Arc::clone(&cb_ref);

        let service = hyper::service::make_service_fn(move |_| {
            let etx = service_etx.clone();
            let cb = Arc::clone(&cb);

            let service = hyper::service::service_fn(move |_req| {
                let etx = etx.clone();
                let cb = Arc::clone(&cb);
                let (req_tx, req_rx) = oneshot::channel::<hyper::Body>();

                // Need to call cb here
                etx.send(EventMessage::ScheduleCallback(Box::new(move |rt| {
                    let vm = rt.vm_mut();
                    let mut scope = vm.scope();

                    let cb = cb.get();

                    let ctx = HttpContext::new(&mut scope, req_tx);
                    let name = scope.intern("respond");
                    let fun = Function::new(&scope, Some(name.into()), FunctionKind::Native(ctx_respond));
                    let fun = scope.register(fun);
                    ctx.set_property(&mut scope, name.into(), PropertyValue::static_default(fun.into()))
                        .unwrap();

                    let ctx = Value::object(scope.register(ctx));

                    if let Err(err) = cb.apply(&mut scope, Value::undefined(), vec![ctx]).root_err(&mut scope) {
                        match err.to_js_string(&mut scope) {
                            Ok(err) => eprintln!("Unhandled exception in HTTP handler! {}", err.res(&scope)),
                            Err(..) => eprintln!("Unhandled exception in exception toString method in HTTP handler!"),
                        }
                    }
                })));

                async {
                    let body = req_rx.await.unwrap();
                    Ok::<_, Infallible>(hyper::Response::new(body))
                }
            });
            async move { Ok::<_, Infallible>(service) }
        });

        let server = hyper::server::Server::bind(&addr).serve(service);

        match server.await {
            Ok(..) => {
                // Shutdown server
                event_tx.send(EventMessage::RemoveTask(task_id));
            }
            Err(err) => eprintln!("Failed to start HTTP server! {err}"),
        }
    });

    Ok(Value::undefined())
}

#[derive(Debug)]
struct HttpContext {
    sender: SharedOnce<Sender<Body>>,
    obj: NamedObject,
}

unsafe impl Trace for HttpContext {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        self.obj.trace(cx);
    }
}

impl HttpContext {
    pub fn new(sc: &mut LocalScope, sender: Sender<Body>) -> Self {
        Self {
            sender: SharedOnce::new(sender),
            obj: NamedObject::new(sc),
        }
    }
}

impl Object for HttpContext {
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

fn ctx_respond(cx: CallContext) -> Result<Value, Value> {
    let this = cx.this.unpack();
    let Some(this) = this.downcast_ref::<HttpContext>(cx.scope) else {
        throw!(cx.scope, TypeError, "Missing this");
    };

    let sender = match this.sender.try_take() {
        Some(sender) => sender,
        None => throw!(cx.scope, Error, "Cannot respond twice"),
    };

    let message = cx.args.first().unwrap_or_undefined().to_js_string(cx.scope)?;

    if sender.send(Body::from(message.res(cx.scope).to_owned())).is_err() {
        eprintln!("Failed to respond to HTTP event.");
    }

    Ok(Value::undefined())
}
