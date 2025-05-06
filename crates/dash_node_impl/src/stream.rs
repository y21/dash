use dash_proc_macro::Trace;
use dash_rt::state::State;
use dash_rt::typemap::Key;
use dash_vm::gc::ObjectId;
use dash_vm::localscope::LocalScope;
use dash_vm::value::Value;
use dash_vm::value::function::native::register_native_fn;
use dash_vm::value::function::{Function, FunctionKind};
use dash_vm::value::object::{Object, OrdObject, PropertyValue};
use dash_vm::value::propertykey::{PropertyKey, ToPropertyKey};
use dash_vm::{delegate, extract};

use crate::state::state_mut;
use crate::symbols::NodeSymbols;

pub fn init_module(sc: &mut LocalScope<'_>) -> Result<Value, Value> {
    let NodeSymbols {
        Readable: readable_sym,
        Stream: stream_sym,
        ..
    } = state_mut(sc).sym;

    let stream_prototype = sc.register(Stream {
        object: OrdObject::new(sc),
    });

    let stream_ctor = Function::new(
        sc,
        Some(stream_sym.into()),
        FunctionKind::Native(|cx| {
            let StreamState { stream_prototype } = State::from_vm(cx.scope).store[StreamKey];

            Ok(cx
                .scope
                .register(Stream {
                    object: OrdObject::with_prototype(stream_prototype),
                })
                .into())
        }),
    );
    stream_ctor.set_fn_prototype(stream_prototype);
    let stream_ctor = sc.register(stream_ctor);
    stream_prototype.set_property(
        PropertyKey::CONSTRUCTOR,
        PropertyValue::static_default(stream_ctor.into()),
        sc,
    )?;

    let readable_fn = register_native_fn(sc, readable_sym, |_sc| Ok(Value::undefined()));
    stream_ctor.set_property(
        readable_sym.to_key(sc),
        PropertyValue::static_default(readable_fn.into()),
        sc,
    )?;
    stream_ctor.set_property(
        stream_sym.to_key(sc),
        PropertyValue::static_default(stream_ctor.into()),
        sc,
    )?;

    State::from_vm_mut(sc)
        .store
        .insert(StreamKey, StreamState { stream_prototype });

    Ok(stream_ctor.into())
}

struct StreamKey;
impl Key for StreamKey {
    type State = StreamState;
}

#[derive(Debug, Trace)]
struct StreamState {
    stream_prototype: ObjectId,
}

#[derive(Debug, Trace)]
struct Stream {
    object: OrdObject,
}

impl Object for Stream {
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
