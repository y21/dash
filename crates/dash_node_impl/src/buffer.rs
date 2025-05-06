use std::cell::Cell;

use dash_middle::interner::sym;
use dash_proc_macro::Trace;
use dash_rt::state::State;
use dash_rt::typemap::Key;
use dash_vm::gc::ObjectId;
use dash_vm::js_std::receiver_t;
use dash_vm::localscope::LocalScope;
use dash_vm::value::arraybuffer::ArrayBuffer;
use dash_vm::value::function::native::{CallContext, register_native_fn};
use dash_vm::value::function::{Function, FunctionKind};
use dash_vm::value::object::{Object, OrdObject, PropertyValue};
use dash_vm::value::ops::conversions::ValueConversion;
use dash_vm::value::primitive::Number;
use dash_vm::value::propertykey::{PropertyKey, ToPropertyKey};
use dash_vm::value::typedarray::{TypedArray, TypedArrayKind};
use dash_vm::value::{ExceptionContext, Root, Unpack, Value, ValueKind};
use dash_vm::{delegate, extract, throw};

use crate::state::state_mut;
use crate::symbols::NodeSymbols;

pub fn init_module(sc: &mut LocalScope<'_>) -> Result<Value, Value> {
    let NodeSymbols {
        Buffer: buffer_sym,
        alloc: alloc_sym,
        writeUInt32BE: wu32be_sym,
        writeUInt32LE: wu32le_sym,
        ..
    } = state_mut(sc).sym;

    let buffer_prototype = {
        let arraybuffer = sc.register(ArrayBuffer::new(sc));
        sc.register(Buffer {
            inner: TypedArray::new(sc, arraybuffer, TypedArrayKind::Uint8Array),
        })
    };
    let wu32be = register_native_fn(sc, wu32be_sym, |cx| write_byte(cx, Endianness::Big, 4));
    let wu32le = register_native_fn(sc, wu32le_sym, |cx| write_byte(cx, Endianness::Little, 4));
    buffer_prototype.set_property(wu32be_sym.to_key(sc), PropertyValue::static_default(wu32be.into()), sc)?;
    buffer_prototype.set_property(wu32le_sym.to_key(sc), PropertyValue::static_default(wu32le.into()), sc)?;

    let buffer_ctor = Function::new(
        sc,
        Some(buffer_sym.into()),
        FunctionKind::Native(|cx| throw!(cx.scope, Error, "Buffer() constructor unsupported")),
    );
    buffer_ctor.set_fn_prototype(buffer_prototype);
    let buffer_ctor = sc.register(buffer_ctor);
    buffer_prototype.set_property(
        PropertyKey::CONSTRUCTOR,
        PropertyValue::static_default(buffer_ctor.into()),
        sc,
    )?;

    let from_fn = register_native_fn(sc, sym::from, from);
    let alloc_fn = register_native_fn(sc, alloc_sym, alloc);
    buffer_ctor.set_property(sym::from.to_key(sc), PropertyValue::static_default(from_fn.into()), sc)?;
    buffer_ctor.set_property(
        buffer_sym.to_key(sc),
        PropertyValue::static_default(buffer_ctor.into()),
        sc,
    )?;
    buffer_ctor.set_property(alloc_sym.to_key(sc), PropertyValue::static_default(alloc_fn.into()), sc)?;

    State::from_vm_mut(sc)
        .store
        .insert(BufferKey, BufferState { buffer_prototype });

    Ok(buffer_ctor.into())
}

#[derive(Debug)]
enum Endianness {
    Little,
    Big,
}

fn write_byte(cx: CallContext, endianness: Endianness, size: usize) -> Result<Value, Value> {
    // TODO: can we just merge this with the TypedArray builtin logic?
    let Some(ValueKind::Number(Number(value))) = cx.args.first().map(|s| s.unpack()) else {
        throw!(cx.scope, Error, "Invalid 'value' argument type")
    };
    let offset = match cx.args.get(1).map(|v| v.unpack()) {
        Some(ValueKind::Number(Number(n))) => n as usize,
        Some(_) => throw!(cx.scope, TypeError, "Invalid 'offset' argument type"),
        None => 0,
    };

    let buf = receiver_t::<Buffer>(cx.scope, &cx.this, "Buffer.prototype.write*")?;
    let buf = if let Some(buf) = buf.inner.arraybuffer(cx.scope).storage().get(offset..) {
        buf
    } else {
        throw!(cx.scope, Error, "out of range")
    };
    let bytes = match (size, endianness) {
        (1, Endianness::Little) => &u8::to_le_bytes(value as u8) as &[_],
        (1, Endianness::Big) => &u8::to_be_bytes(value as u8),
        (2, Endianness::Little) => &u16::to_le_bytes(value as u16),
        (2, Endianness::Big) => &u16::to_be_bytes(value as u16),
        (4, Endianness::Little) => &u32::to_le_bytes(value as u32),
        (4, Endianness::Big) => &u32::to_be_bytes(value as u32),
        other => todo!("unimplemented byte write op: {other:?}"),
    };
    for (index, byte) in bytes.iter().copied().enumerate() {
        buf[index].set(byte);
    }

    Ok(Value::number((offset + size) as f64))
}

struct BufferKey;
impl Key for BufferKey {
    type State = BufferState;
}

#[derive(Debug, Trace)]
struct BufferState {
    buffer_prototype: ObjectId,
}

#[derive(Debug, Trace)]
struct Buffer {
    inner: TypedArray,
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
        own_keys
    );

    extract!(self, inner);
}

fn from(cx: CallContext) -> Result<Value, Value> {
    let BufferState { buffer_prototype } = State::from_vm(cx.scope).store[BufferKey];

    let source = cx
        .args
        .first()
        .or_type_err(cx.scope, "Missing source to `Buffer.from`")?;

    let length = source.length_of_array_like(cx.scope)?;
    let mut buf = Vec::with_capacity(length);
    for i in 0..length {
        let item = source
            .get_property(i.to_key(cx.scope), cx.scope)
            .root(cx.scope)?
            .to_number(cx.scope)? as u8;
        buf.push(Cell::new(item));
    }

    let arraybuffer = cx.scope.register(ArrayBuffer::from_storage(cx.scope, buf));
    let instn = Buffer {
        inner: TypedArray::with_obj(
            arraybuffer,
            TypedArrayKind::Uint8Array,
            OrdObject::with_prototype(buffer_prototype),
        ),
    };

    Ok(Value::object(cx.scope.register(instn)))
}

fn alloc(cx: CallContext) -> Result<Value, Value> {
    let size = cx
        .args
        .first()
        .or_type_err(cx.scope, "Missing size argument to `Buffer.alloc`")?;
    let size = size.to_number(cx.scope)? as usize;

    let fill = cx.args.get(1).copied();

    let buf = if let Some(fill) = fill {
        let unpacked = fill.unpack();
        if let ValueKind::Number(Number(num)) = unpacked {
            vec![Cell::new(num as u8); size]
        } else {
            throw!(cx.scope, Error, "invalid fill argument to Buffer.alloc: {:?}", unpacked)
        }
    } else {
        vec![Cell::new(0); size]
    };

    let BufferState { buffer_prototype, .. } = State::from_vm(cx.scope).store[BufferKey];
    let arraybuffer = cx.scope.register(ArrayBuffer::from_storage(cx.scope, buf));
    let instn = Buffer {
        inner: TypedArray::with_obj(
            arraybuffer,
            TypedArrayKind::Uint8Array,
            OrdObject::with_prototype(buffer_prototype),
        ),
    };

    Ok(Value::object(cx.scope.register(instn)))
}
