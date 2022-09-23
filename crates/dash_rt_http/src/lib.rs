use std::convert::Infallible;
use std::mem;
use std::net::SocketAddr;

use dash_middle::compiler::StaticImportKind;
use dash_middle::util::SharedOnce;
use dash_rt::event::EventMessage;
use dash_rt::module::ModuleLoader;
use dash_rt::state::State;
use dash_rt::ThreadSafeHandle;
use dash_vm::delegate;
use dash_vm::gc::handle::Handle;
use dash_vm::gc::trace::Trace;
use dash_vm::local::LocalScope;
use dash_vm::throw;
use dash_vm::value::function::native::CallContext;
use dash_vm::value::function::Function;
use dash_vm::value::function::FunctionKind;
use dash_vm::value::object::NamedObject;
use dash_vm::value::object::Object;
use dash_vm::value::object::PropertyValue;
use dash_vm::value::ops::abstractions::conversions::ValueConversion;
use dash_vm::value::Value;
use dash_vm::value::ValueContext;
use hyper::Body;
use tokio::sync::oneshot;
use tokio::sync::oneshot::Sender;

#[derive(Debug)]
pub struct HttpModule;

impl ModuleLoader for HttpModule {
    fn import(&self, sc: &mut LocalScope, _import_ty: StaticImportKind, path: &str) -> Option<Value> {
        if path != "@std/http" {
            return None;
        }

        let module = NamedObject::new(sc);
        let listen = Function::new(sc, None, FunctionKind::Native(listen));
        let listen = sc.register(listen);
        module
            .set_property(sc, "listen".into(), PropertyValue::static_default(listen.into()))
            .ok()?;

        let module = sc.register(module);
        Some(module.into())
    }
}

pub fn listen(mut cx: CallContext) -> Result<Value, Value> {
    let port = cx.args.first().unwrap_or_undefined().to_int32(&mut cx.scope)?;
    let cb = match cx.args.get(1).cloned() {
        Some(Value::Object(o)) => o,
        _ => throw!(cx.scope, "Expected callback function as second argument"),
    };

    {
        // Leak LocalScope containing HTTP callback function so that it is forever marked as reachable
        // TODO: refactor using persistent scopes once we have those.
        let mut t = LocalScope::new(&mut cx.scope);
        t.add_ref(Handle::clone(&cb));
        mem::forget(t);
    }

    let addr = SocketAddr::from(([127, 0, 0, 1], port as u16));

    let state = State::from_vm(&cx.scope);

    let task_id = state.active_tasks().add(); // We (currently) never remove the task
                                              // because the server always runs forever
    let etx = state.event_sender();
    let rt = state.rt_handle();

    let cb = ThreadSafeHandle::new(cb);

    let server = async move {
        let service_etx = etx.clone();
        let service = hyper::service::make_service_fn(move |_| {
            let etx = service_etx.clone();
            let cb = cb.clone();

            let service = hyper::service::service_fn(move |_req| {
                let etx = etx.clone();
                let cb = cb.clone();

                let (ttx, trx) = oneshot::channel::<hyper::Body>();

                etx.send(EventMessage::ScheduleCallback(Box::new(move |rt| {
                    let vm = rt.vm_mut();
                    let mut scope = LocalScope::new(vm);

                    let cb = cb.into_inner();

                    let ctx = HttpContext::new(&mut scope, ttx);
                    let fun = Function::new(&mut scope, Some("respond".into()), FunctionKind::Native(ctx_respond));
                    let fun = scope.register(fun);
                    ctx.set_property(&mut scope, "respond".into(), PropertyValue::static_default(fun.into()))
                        .unwrap();

                    let ctx = Value::Object(scope.register(ctx));

                    if let Err(err) = cb.apply(&mut scope, Value::undefined(), vec![ctx]) {
                        match err.to_string(&mut scope) {
                            Ok(err) => eprintln!("Unhandled exception in HTTP handler! {}", err),
                            Err(..) => eprintln!("Unhandled exception in exception toString method in HTTP handler!"),
                        }
                    }
                })));

                async {
                    let body = trx.await.unwrap();
                    Ok::<_, Infallible>(hyper::Response::new(body))
                }
            });

            async move { Ok::<_, Infallible>(service) }
        });

        let server = hyper::server::Server::bind(&addr).serve(service);

        match server.await {
            Ok(..) => {
                // Shutdown server
                etx.send(EventMessage::RemoveTask(task_id));
            }
            Err(err) => eprintln!("Failed to start HTTP server! {err}"),
        }
    };

    rt.spawn(server);

    Ok(Value::undefined())
}

#[derive(Debug)]
struct HttpContext {
    sender: SharedOnce<Sender<Body>>,
    obj: NamedObject,
}

unsafe impl Trace for HttpContext {
    fn trace(&self) {
        self.obj.trace();
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

fn ctx_respond(cx: CallContext) -> Result<Value, Value> {
    let this = match &cx.this {
        Value::Object(this) | Value::External(this) => match this.as_any().downcast_ref::<HttpContext>() {
            Some(ctx) => ctx,
            None => throw!(cx.scope, "Incompatible receiver"),
        },
        _ => throw!(cx.scope, "Missing this"),
    };

    let sender = match this.sender.try_take() {
        Some(sender) => sender,
        None => throw!(cx.scope, "Cannot respond twice"),
    };

    let message = cx.args.first().unwrap_or_undefined().to_string(cx.scope)?;

    if let Err(_) = sender.send(Body::from(ToString::to_string(&message))) {
        eprintln!("Failed to respond to HTTP event.");
    }

    Ok(Value::undefined())
}
