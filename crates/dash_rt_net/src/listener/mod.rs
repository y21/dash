use std::cell::Cell;

use dash_proc_macro::Trace;
use dash_rt::event::EventMessage;
use dash_rt::state::State;
use dash_rt::wrap_async;
use dash_vm::gc::persistent::Persistent;
use dash_vm::gc::trace::{Trace, TraceCtxt};
use dash_vm::localscope::LocalScope;
use dash_vm::value::arraybuffer::ArrayBuffer;
use dash_vm::value::function::native::CallContext;
use dash_vm::value::function::{Function, FunctionKind};
use dash_vm::value::object::{NamedObject, Object, PropertyValue};
use dash_vm::value::ops::conversions::ValueConversion;
use dash_vm::value::promise::Promise;
use dash_vm::value::{Unrooted, Value};
use dash_vm::{delegate, throw, PromiseAction};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug, Trace)]
pub struct TcpListenerConstructor {}

impl Object for TcpListenerConstructor {
    fn get_own_property_descriptor(
        &self,
        _sc: &mut dash_vm::localscope::LocalScope,
        _key: dash_vm::value::object::PropertyKey,
    ) -> Result<Option<dash_vm::value::object::PropertyValue>, dash_vm::value::Unrooted> {
        Ok(None)
    }

    fn set_property(
        &self,
        _sc: &mut dash_vm::localscope::LocalScope,
        _key: dash_vm::value::object::PropertyKey,
        _value: dash_vm::value::object::PropertyValue,
    ) -> Result<(), dash_vm::value::Value> {
        Ok(())
    }

    fn delete_property(
        &self,
        _sc: &mut dash_vm::localscope::LocalScope,
        _key: dash_vm::value::object::PropertyKey,
    ) -> Result<dash_vm::value::Unrooted, dash_vm::value::Value> {
        Ok(Unrooted::new(Value::undefined()))
    }

    fn set_prototype(
        &self,
        _sc: &mut dash_vm::localscope::LocalScope,
        _value: dash_vm::value::Value,
    ) -> Result<(), dash_vm::value::Value> {
        Ok(())
    }

    fn get_prototype(
        &self,
        _sc: &mut dash_vm::localscope::LocalScope,
    ) -> Result<dash_vm::value::Value, dash_vm::value::Value> {
        Ok(Value::undefined())
    }

    fn apply(
        &self,
        scope: &mut dash_vm::localscope::LocalScope,
        _callee: dash_vm::gc::handle::Handle,
        _this: dash_vm::value::Value,
        _args: Vec<dash_vm::value::Value>,
    ) -> Result<dash_vm::value::Unrooted, dash_vm::value::Unrooted> {
        throw!(scope, Error, "TcpListener should be called as a constructor")
    }

    fn construct(
        &self,
        scope: &mut dash_vm::localscope::LocalScope,
        _callee: dash_vm::gc::handle::Handle,
        _this: Value,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        let Some(value) = args.first() else {
            throw!(
                scope,
                TypeError,
                "TcpListener requires the first argument be the address"
            );
        };
        let value = String::from(value.to_js_string(scope)?.res(scope));

        let (tx, mut rx) = mpsc::channel(1);
        let state = State::from_vm(scope);
        let event_tx = state.event_sender();
        let async_handle = state.rt_handle();
        async_handle.clone().spawn(async move {
            let listener = TcpListener::bind(value).await.unwrap(); // TODO: handle correctly

            while let Some(message) = rx.recv().await {
                match message {
                    TcpListenerBridgeMessage::Accept { promise_id } => {
                        let (stream, _) = listener.accept().await.unwrap(); // TODO: handle correctly
                        let (mut read_half, mut write_half) = stream.into_split();

                        let (writer_tx, mut writer_rx) = mpsc::unbounded_channel::<Box<[u8]>>();
                        let (reader_tx, mut reader_rx) = mpsc::unbounded_channel::<oneshot::Sender<Box<[u8]>>>();
                        async_handle.spawn(async move {
                            // TcpStream reader end
                            while let Some(reply) = reader_rx.recv().await {
                                let mut buf = Vec::new();
                                read_half.read_buf(&mut buf).await.unwrap();
                                reply.send(buf.into_boxed_slice()).unwrap();
                            }
                        });
                        async_handle.spawn(async move {
                            // TcpStream writer end
                            while let Some(message) = writer_rx.recv().await {
                                write_half.write_all(&message).await.unwrap();
                            }
                        });
                        event_tx.send(EventMessage::ScheduleCallback(Box::new(move |rt| {
                            let mut scope = rt.vm_mut().scope();
                            let promise = State::from_vm(&scope).take_promise(promise_id);
                            let promise = promise.as_any().downcast_ref::<Promise>().unwrap();

                            let stream_handle = TcpStreamHandle::new(&mut scope, writer_tx, reader_tx).unwrap();
                            let stream_handle = scope.register(stream_handle);

                            scope.drive_promise(PromiseAction::Resolve, promise, vec![Value::Object(stream_handle)]);
                            scope.process_async_tasks();
                        })));
                    }
                }
            }
        });

        let handle = TcpListenerHandle::new(tx, scope)?;
        Ok(Value::Object(scope.register(handle)).into())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn own_keys(&self, _: &mut LocalScope<'_>) -> Result<Vec<dash_vm::value::Value>, dash_vm::value::Value> {
        Ok(Vec::new())
    }
}

enum TcpListenerBridgeMessage {
    Accept { promise_id: u64 },
}

#[derive(Debug)]
struct TcpListenerHandle {
    object: NamedObject,
    sender: mpsc::Sender<TcpListenerBridgeMessage>,
}

// SAFETY: all fields are recursively traced, enforced via pattern matching
unsafe impl Trace for TcpListenerHandle {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        let Self { object, sender: _ } = self;
        object.trace(cx);
    }
}

impl TcpListenerHandle {
    pub fn new(sender: mpsc::Sender<TcpListenerBridgeMessage>, sc: &mut LocalScope) -> Result<Self, Value> {
        let object = NamedObject::new(sc);
        let name = sc.intern("accept");
        let accept_fn = Function::new(sc, Some(name.into()), FunctionKind::Native(tcplistener_accept));
        let accept_fn = sc.register(accept_fn);
        object.set_property(sc, name.into(), PropertyValue::static_default(Value::Object(accept_fn)))?;
        Ok(Self { object, sender })
    }
}

impl Object for TcpListenerHandle {
    delegate!(
        object,
        get_own_property_descriptor,
        set_property,
        delete_property,
        set_prototype,
        get_prototype,
        apply,
        as_any,
        own_keys
    );
}

fn tcplistener_accept(cx: CallContext) -> Result<Value, Value> {
    let Some(handle) = cx.this.downcast_ref::<TcpListenerHandle>() else {
        throw!(
            cx.scope,
            TypeError,
            "TcpListener.accept called on non-TcpListener object"
        )
    };
    // TODO: don't use handle after using scope. Enforce this at comptime somehow
    let promise = {
        let promise = Promise::new(cx.scope);
        cx.scope.register(promise)
    };

    let persistent_promise = Persistent::new(cx.scope, promise.clone());
    let promise_id = State::from_vm(cx.scope).add_pending_promise(persistent_promise);

    handle
        .sender
        .try_send(TcpListenerBridgeMessage::Accept { promise_id })
        .expect("queue full");

    Ok(Value::Object(promise))
}

#[derive(Debug)]
struct TcpStreamHandle {
    object: NamedObject,
    writer_tx: mpsc::UnboundedSender<Box<[u8]>>,
    reader_tx: mpsc::UnboundedSender<oneshot::Sender<Box<[u8]>>>,
}

impl TcpStreamHandle {
    pub fn new(
        scope: &mut LocalScope,
        writer_tx: mpsc::UnboundedSender<Box<[u8]>>,
        reader_tx: mpsc::UnboundedSender<oneshot::Sender<Box<[u8]>>>,
    ) -> Result<Self, Value> {
        let object = NamedObject::new(scope);
        let name = scope.intern("write");
        let write_fn = Function::new(scope, Some(name.into()), FunctionKind::Native(tcpstream_write));
        let write_fn = scope.register(write_fn);
        object.set_property(
            scope,
            name.into(),
            PropertyValue::static_default(Value::Object(write_fn)),
        )?;
        let name = scope.intern("read");
        let read_fn = Function::new(scope, Some(name.into()), FunctionKind::Native(tcpstream_read));
        let read_fn = scope.register(read_fn);
        object.set_property(
            scope,
            name.into(),
            PropertyValue::static_default(Value::Object(read_fn)),
        )?;
        Ok(Self {
            object,
            writer_tx,
            reader_tx,
        })
    }
}

unsafe impl Trace for TcpStreamHandle {
    fn trace(&self, cx: &mut TraceCtxt<'_>) {
        let Self {
            object,
            writer_tx: _,
            reader_tx: _,
        } = self;
        object.trace(cx);
    }
}

impl Object for TcpStreamHandle {
    delegate!(
        object,
        get_own_property_descriptor,
        set_property,
        delete_property,
        set_prototype,
        get_prototype,
        apply,
        as_any,
        own_keys
    );
}

fn tcpstream_write(cx: CallContext) -> Result<Value, Value> {
    let Some(handle) = cx.this.downcast_ref::<TcpStreamHandle>() else {
        throw!(cx.scope, TypeError, "TcpStream.write called on non-TcpStream object")
    };
    let Some(value) = cx.args.first().unwrap().downcast_ref::<ArrayBuffer>() else {
        throw!(
            cx.scope,
            TypeError,
            "TcpStream.write called with non-ArrayBuffer argument"
        )
    };

    // As of 8/2/2023, gets correctly optimized to a memcpy
    let buf = value
        .storage()
        .iter()
        .map(|v| v.get())
        .collect::<Vec<u8>>()
        .into_boxed_slice();

    handle.writer_tx.send(buf).expect("TcpStream closed");

    // TODO: return value?
    Ok(Value::undefined())
}

fn tcpstream_read(cx: CallContext) -> Result<Value, Value> {
    let Some(handle) = cx.this.downcast_ref::<TcpStreamHandle>() else {
        throw!(cx.scope, TypeError, "TcpStream.write called on non-TcpStream object")
    };

    let (tx, rx) = oneshot::channel();

    handle.reader_tx.send(tx).unwrap();

    wrap_async(cx, rx, |sc, ret| {
        let ret = ret.unwrap();
        let buf = Vec::from(ret).into_iter().map(Cell::new).collect::<Vec<_>>();
        let buf = ArrayBuffer::from_storage(sc, buf);

        Ok(Value::Object(sc.register(buf)))
    })
}
