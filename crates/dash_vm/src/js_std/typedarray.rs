use std::cell::Cell;
use std::convert::Infallible;
use std::ops::ControlFlow;

use crate::js_std::array::for_each_js_iterator_element;
use crate::throw;
use crate::value::arraybuffer::ArrayBuffer;
use crate::value::function::native::CallContext;
use crate::value::ops::conversions::ValueConversion;
use crate::value::typedarray::{TypedArray, TypedArrayKind};
use crate::value::{Root, Unpack, Value, ValueKind};

fn typedarray_constructor(cx: CallContext, kind: TypedArrayKind) -> Result<Value, Value> {
    let Some(&arg) = cx.args.first() else {
        throw!(cx.scope, TypeError, "Missing argument")
    };

    if let ValueKind::Object(obj) = arg.unpack() {
        if let Some(this) = obj.extract::<ArrayBuffer>(cx.scope) {
            if this.len() % kind.bytes_per_element() != 0 {
                throw!(
                    cx.scope,
                    RangeError,
                    "Length of array buffer must be a multiple of {}",
                    kind.bytes_per_element()
                );
            }

            let array = TypedArray::new(cx.scope, obj, kind);
            return Ok(cx.scope.register(array).into());
        }

        if let Some(iterator) = obj
            .get_property(cx.scope, cx.scope.statics.symbol_iterator.into())
            .root(cx.scope)?
            .into_option()
        {
            let iterator = iterator.apply(cx.scope, arg, Vec::new()).root(cx.scope)?;
            let mut values = Vec::new();
            for_each_js_iterator_element(cx.scope, iterator, |scope, value| {
                use TypedArrayKind::*;

                let value = value.to_number(scope)?;

                match kind {
                    Int8Array | Uint8Array => values.push(Cell::new(value as u8)),
                    Uint8ClampedArray => values.push(Cell::new(value.clamp(0.0, u8::MAX as f64) as u8)),
                    Int16Array | Uint16Array => values.extend_from_slice(&(value as u16).to_ne_bytes().map(Cell::new)),
                    Int32Array | Uint32Array => values.extend_from_slice(&(value as u32).to_ne_bytes().map(Cell::new)),
                    Float32Array => values.extend_from_slice(&(value as f32).to_ne_bytes().map(Cell::new)),
                    Float64Array => values.extend_from_slice(&value.to_ne_bytes().map(Cell::new)),
                }
                Ok(ControlFlow::<Infallible, _>::Continue(()))
            })?;

            let buffer = ArrayBuffer::from_storage(cx.scope, values);
            let buffer = cx.scope.register(buffer);

            let array = TypedArray::new(cx.scope, buffer, kind);
            return Ok(cx.scope.register(array).into());
        }
    }

    let size = arg.to_number(cx.scope)? as usize;
    let buffer = ArrayBuffer::with_capacity(cx.scope, size);
    let buffer = cx.scope.register(buffer);

    let array = TypedArray::new(cx.scope, buffer, kind);
    Ok(cx.scope.register(array).into())
}

macro_rules! typedarray {
    (module: $module:ident, kind: $kind:expr) => {
        pub mod $module {
            use super::*;

            pub fn constructor(cx: CallContext) -> Result<Value, Value> {
                typedarray_constructor(cx, $kind)
            }
        }
    };
}

pub fn fill(cx: CallContext) -> Result<Value, Value> {
    let this = cx.this.unpack();
    let this = match this.downcast_ref::<TypedArray>(cx.scope) {
        Some(this) => this,
        None => throw!(cx.scope, TypeError, "Invalid receiver"),
    };
    let value = match cx.args.first() {
        Some(value) => value.to_number(cx.scope)?,
        None => throw!(cx.scope, TypeError, "Missing fill value"), // TODO: shouldn't throw
    };
    let buf = this.arraybuffer(cx.scope).storage();

    macro_rules! fill_typed_array {
        ($ty:ty) => {{
            let value = <$ty>::to_ne_bytes(value as $ty);
            for chunk in buf.chunks_exact(value.len()) {
                // For Uint8Array, it only compiles to a memset if we use an indexed for loop
                // It seems like zipped iterators are not smart enough
                for index in 0..value.len() {
                    chunk[index].set(value[index]);
                }
            }
        }};
    }

    match this.kind() {
        TypedArrayKind::Uint32Array => fill_typed_array!(u32),
        TypedArrayKind::Int8Array => fill_typed_array!(i8),
        TypedArrayKind::Uint8Array => fill_typed_array!(u8),
        TypedArrayKind::Uint8ClampedArray => fill_typed_array!(u8),
        TypedArrayKind::Int16Array => fill_typed_array!(i16),
        TypedArrayKind::Uint16Array => fill_typed_array!(u16),
        TypedArrayKind::Int32Array => fill_typed_array!(i32),
        TypedArrayKind::Float32Array => fill_typed_array!(f32),
        TypedArrayKind::Float64Array => fill_typed_array!(f64),
    }
    Ok(Value::undefined())
}

typedarray!(module: u8array, kind: TypedArrayKind::Uint8Array);
typedarray!(module: i8array, kind: TypedArrayKind::Int8Array);
typedarray!(module: u8clampedarray, kind: TypedArrayKind::Uint8ClampedArray);
typedarray!(module: i16array, kind: TypedArrayKind::Int16Array);
typedarray!(module: u16array, kind: TypedArrayKind::Uint16Array);
typedarray!(module: i32array, kind: TypedArrayKind::Int32Array);
typedarray!(module: u32array, kind: TypedArrayKind::Uint32Array);
typedarray!(module: f32array, kind: TypedArrayKind::Float32Array);
typedarray!(module: f64array, kind: TypedArrayKind::Float64Array);
