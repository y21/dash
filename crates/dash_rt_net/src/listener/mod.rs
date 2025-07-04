use std::cell::Cell;

use dash_proc_macro::Trace;
use dash_rt::event::EventMessage;
use dash_rt::state::State;
use dash_rt::wrap_async;
use dash_vm::gc::ObjectId;
use dash_vm::gc::trace::{Trace, TraceCtxt};
use dash_vm::js_std::receiver_t;
use dash_vm::localscope::LocalScope;
use dash_vm::value::arraybuffer::ArrayBuffer;
use dash_vm::value::function::args::CallArgs;
use dash_vm::value::function::native::CallContext;
use dash_vm::value::function::{Function, FunctionKind};
use dash_vm::value::object::{Object, OrdObject, PropertyValue, This};
use dash_vm::value::ops::conversions::ValueConversion;
use dash_vm::value::promise::Promise;
use dash_vm::value::propertykey::{PropertyKey, ToPropertyKey};
use dash_vm::value::{Unpack, Unrooted, Value};
use dash_vm::{PromiseAction, delegate, extract, throw};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug, Trace)]
pub struct TcpListenerConstructor {}

impl Object for TcpListenerConstructor {
    fn get_own_property_descriptor(
        &self,
        _key: PropertyKey,
        _sc: &mut dash_vm::localscope::LocalScope,
    ) -> Result<Option<dash_vm::value::object::PropertyValue>, dash_vm::value::Unrooted> {
        Ok(None)
    }

    fn set_property(
        &self,
        _key: PropertyKey,
        _value: dash_vm::value::object::PropertyValue,
        _sc: &mut dash_vm::localscope::LocalScope,
    ) -> Result<(), dash_vm::value::Value> {
        Ok(())
    }

    fn delete_property(
        &self,
        _key: PropertyKey,
        _sc: &mut dash_vm::localscope::LocalScope,
    ) -> Result<dash_vm::value::Unrooted, dash_vm::value::Value> {
        Ok(Unrooted::new(Value::undefined()))
    }

    fn set_prototype(
        &self,
        _value: dash_vm::value::Value,
        _sc: &mut dash_vm::localscope::LocalScope,
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
        _callee: dash_vm::gc::ObjectId,
        _this: This,
        _args: CallArgs,
        scope: &mut dash_vm::localscope::LocalScope,
    ) -> Result<dash_vm::value::Unrooted, dash_vm::value::Unrooted> {
        throw!(scope, Error, "TcpListener should be called as a constructor")
    }

    fn construct(
        &self,
        _callee: dash_vm::gc::ObjectId,
        _this: This,
        args: CallArgs,
        new_target: ObjectId,
        scope: &mut dash_vm::localscope::LocalScope,
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
        let state = State::from_vm_mut(scope);
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
                            let promise_id = State::from_vm_mut(&mut scope).take_promise(promise_id);
                            let promise = promise_id.extract::<Promise>(&scope).unwrap();

                            let stream_handle = TcpStreamHandle::new(&mut scope, writer_tx, reader_tx).unwrap();
                            let stream_handle = scope.register(stream_handle);

                            scope.drive_promise(
                                PromiseAction::Resolve,
                                promise,
                                promise_id,
                                [Value::object(stream_handle)].into(),
                            );
                            scope.process_async_tasks();
                        })));
                    }
                }
            }
        });

        let handle = TcpListenerHandle::new(OrdObject::instance_for_new_target(new_target, scope)?, tx, scope)?;
        Ok(Value::object(scope.register(handle)).into())
    }

    fn own_keys(&self, _: &mut LocalScope<'_>) -> Result<Vec<dash_vm::value::Value>, dash_vm::value::Value> {
        Ok(Vec::new())
    }

    extract!(self);
}

enum TcpListenerBridgeMessage {
    Accept { promise_id: u64 },
}

#[derive(Debug)]
struct TcpListenerHandle {
    object: OrdObject,
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
    pub fn new(
        object: OrdObject,
        sender: mpsc::Sender<TcpListenerBridgeMessage>,
        sc: &mut LocalScope,
    ) -> Result<Self, Value> {
        let name = sc.intern("accept");
        let accept_fn = Function::new(sc, Some(name.into()), FunctionKind::Native(tcplistener_accept));
        let accept_fn = sc.register(accept_fn);
        object.set_property(
            name.to_key(sc),
            PropertyValue::static_default(Value::object(accept_fn)),
            sc,
        )?;
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
        own_keys
    );

    extract!(self);
}

fn tcplistener_accept(cx: CallContext) -> Result<Value, Value> {
    let handle = receiver_t::<TcpListenerHandle>(cx.scope, &cx.this, "TcpListener.prototype.accept")?;
    let promise = cx.scope.mk_promise();

    let promise_id = State::from_vm_mut(cx.scope).add_pending_promise(promise);

    handle
        .sender
        .try_send(TcpListenerBridgeMessage::Accept { promise_id })
        .expect("queue full");

    Ok(Value::object(promise))
}

#[derive(Debug)]
struct TcpStreamHandle {
    object: OrdObject,
    writer_tx: mpsc::UnboundedSender<Box<[u8]>>,
    reader_tx: mpsc::UnboundedSender<oneshot::Sender<Box<[u8]>>>,
}

impl TcpStreamHandle {
    pub fn new(
        scope: &mut LocalScope,
        writer_tx: mpsc::UnboundedSender<Box<[u8]>>,
        reader_tx: mpsc::UnboundedSender<oneshot::Sender<Box<[u8]>>>,
    ) -> Result<Self, Value> {
        let object = OrdObject::new(scope);
        let name = scope.intern("write");
        let write_fn = Function::new(scope, Some(name.into()), FunctionKind::Native(tcpstream_write));
        let write_fn = scope.register(write_fn);
        object.set_property(
            name.to_key(scope),
            PropertyValue::static_default(Value::object(write_fn)),
            scope,
        )?;
        let name = scope.intern("read");
        let read_fn = Function::new(scope, Some(name.into()), FunctionKind::Native(tcpstream_read));
        let read_fn = scope.register(read_fn);
        object.set_property(
            name.to_key(scope),
            PropertyValue::static_default(Value::object(read_fn)),
            scope,
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
        own_keys
    );

    extract!(self);
}

fn tcpstream_write(cx: CallContext) -> Result<Value, Value> {
    let handle = receiver_t::<TcpStreamHandle>(cx.scope, &cx.this, "TcpStream.prototyep.write")?;
    let Some(arg) = cx.args.first().map(|v| v.unpack()) else {
        throw!(cx.scope, ReferenceError, "TcpStream.write called without an argument")
    };
    let Some(value) = arg.downcast_ref::<ArrayBuffer>(cx.scope) else {
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
    let handle = receiver_t::<TcpStreamHandle>(cx.scope, &cx.this, "TcpStream.prototype.write")?;

    let (tx, rx) = oneshot::channel();

    handle.reader_tx.send(tx).unwrap();

    wrap_async(cx, rx, |sc, ret| {
        let ret = ret.unwrap();
        let buf = Vec::from(ret).into_iter().map(Cell::new).collect::<Vec<_>>();
        let buf = ArrayBuffer::from_storage(sc, buf);

        Ok(Value::object(sc.register(buf)))
    })
}
