use std::cell::RefCell;
use std::convert::Infallible;
use std::net::SocketAddr;

use dash_core::gc::handle::Handle;
use dash_core::gc::trace::Trace;
use dash_core::throw;
use dash_core::vm::local::LocalScope;
use dash_core::vm::value::function::native::CallContext;
use dash_core::vm::value::object::NamedObject;
use dash_core::vm::value::object::Object;
use dash_core::vm::value::object::PropertyKey;
use dash_core::vm::value::ops::abstractions::conversions::ValueConversion;
use dash_core::vm::value::Value;
use dash_core::vm::value::ValueContext;
use dash_core::vm::Vm;
use hyper::Body;
use tokio::sync::oneshot;

use crate::event::EventMessage;
use crate::state::State;

#[derive(Debug)]
pub struct HttpContext {
    sender: RefCell<Option<oneshot::Sender<Body>>>,
    obj: NamedObject,
}

impl HttpContext {
    pub fn new(vm: &mut Vm, sender: oneshot::Sender<Body>) -> Self {
        Self {
            sender: RefCell::new(Some(sender)),
            obj: NamedObject::new(vm),
        }
    }
}

unsafe impl Trace for HttpContext {
    fn trace(&self) {
        self.obj.trace();
    }
}

impl Object for HttpContext {
    fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        self.obj.get_property(sc, key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey<'static>, value: Value) -> Result<(), Value> {
        self.obj.set_property(sc, key, value)
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        self.obj.set_prototype(sc, value)
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        self.obj.get_prototype(sc)
    }

    fn apply<'s>(
        &self,
        scope: &mut LocalScope,
        callee: Handle<dyn Object>,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        self.obj.apply(scope, callee, this, args)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        self.obj.own_keys()
    }
}

pub fn ctx_respond(mut cx: CallContext) -> Result<Value, Value> {
    let this = match &cx.this {
        Value::Object(this) => match this.as_any().downcast_ref::<HttpContext>() {
            Some(ctx) => ctx,
            None => throw!(cx.scope, "Incompatible receiver"),
        },
        _ => throw!(cx.scope, "Missing this"),
    };

    let sender = match this.sender.borrow_mut().take() {
        Some(sender) => sender,
        None => throw!(cx.scope, "Cannot respond twice"),
    };

    let message = cx.args.first().unwrap_or_undefined().to_string(&mut cx.scope)?;

    sender.send(Body::from(message.to_string())).unwrap();

    Ok(Value::undefined())
}

pub fn listen(mut cx: CallContext) -> Result<Value, Value> {
    let port = cx.args.first().unwrap_or_undefined().to_int32(&mut cx.scope)?;
    let cb = match cx.args.get(1) {
        Some(Value::Object(o)) => o,
        _ => throw!(cx.scope, "Expected callback function as second argument"),
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], port as u16));

    let state = State::try_from_vm(&cx.scope).unwrap();
    state.set_http_handler(cb);

    let etx = state.event_sender();
    let rt = state.rt_handle();

    let server = async move {
        let service = hyper::service::make_service_fn(move |_| {
            let etx = etx.clone();

            let service = hyper::service::service_fn(move |req| {
                let etx = etx.clone();

                let (ttx, trx) = oneshot::channel();

                etx.send(EventMessage::HttpRequest(req, ttx)).unwrap();

                async {
                    let body = trx.await.unwrap();
                    Ok::<_, Infallible>(hyper::Response::new(body))
                }
            });

            async move { Ok::<_, Infallible>(service) }
        });

        let server = hyper::server::Server::bind(&addr).serve(service);

        server.await
    };

    rt.spawn(server);

    Ok(Value::undefined())
}
