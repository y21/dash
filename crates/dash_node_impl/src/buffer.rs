use dash_middle::interner::sym;
use dash_proc_macro::Trace;
use dash_rt::state::State;
use dash_rt::typemap::Key;
use dash_vm::gc::ObjectId;
use dash_vm::localscope::LocalScope;
use dash_vm::value::arraybuffer::ArrayBuffer;
use dash_vm::value::function::native::{register_native_fn, CallContext};
use dash_vm::value::function::{Function, FunctionKind};
use dash_vm::value::object::{Object, PropertyValue};
use dash_vm::value::Value;
use dash_vm::{delegate, throw};

use crate::state::state_mut;
use crate::symbols::NodeSymbols;

pub fn init_module(sc: &mut LocalScope<'_>) -> Result<Value, Value> {
    let NodeSymbols { Buffer: buffer_sym, .. } = state_mut(sc).sym;

    // TODO: helper function for creating a (prototype, constructor) tuple
    let buffer_prototype = sc.register(Buffer {
        inner: ArrayBuffer::new(sc),
    });

    let buffer_ctor = Function::new(
        sc,
        Some(buffer_sym.into()),
        FunctionKind::Native(|cx| throw!(cx.scope, Error, "Buffer() constructor unsupported")),
    );
    buffer_ctor.set_fn_prototype(buffer_prototype);
    let buffer_ctor = sc.register(buffer_ctor);

    let from_fn = register_native_fn(sc, sym::from, from);
    buffer_ctor.set_property(sc, sym::from.into(), PropertyValue::static_default(from_fn.into()))?;
    buffer_ctor.set_property(sc, buffer_sym.into(), PropertyValue::static_default(buffer_ctor.into()))?;

    State::from_vm_mut(sc).store.insert(
        BufferKey,
        BufferState {
            buffer_prototype,
            buffer_ctor,
        },
    );

    Ok(buffer_ctor.into())
}

struct BufferKey;
impl Key for BufferKey {
    type State = BufferState;
}

#[derive(Debug, Trace)]
struct BufferState {
    buffer_prototype: ObjectId,
    buffer_ctor: ObjectId,
}

#[derive(Debug, Trace)]
struct Buffer {
    inner: ArrayBuffer,
}

impl Object for Buffer {
    delegate!(
        inner,
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

fn from(cx: CallContext) -> Result<Value, Value> {
    let instn = Buffer {
        inner: ArrayBuffer::new(cx.scope),
    };

    Ok(Value::object(cx.scope.register(instn)))
}
