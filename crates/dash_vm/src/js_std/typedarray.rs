use crate::throw;
use crate::value::arraybuffer::ArrayBuffer;
use crate::value::function::native::CallContext;
use crate::value::object::Object;
use crate::value::ops::abstractions::conversions::ValueConversion;
use crate::value::typedarray::{TypedArray, TypedArrayKind};
use crate::value::Value;

macro_rules! typedarray {
    (module: $module:ident, kind: $kind:expr) => {
        pub mod $module {
            use super::*;

            pub fn constructor(cx: CallContext) -> Result<Value, Value> {
                let arg = match cx.args.first() {
                    Some(Value::Object(o)) => o,
                    Some(Value::External(o)) => &o.inner,
                    _ => throw!(cx.scope, TypeError, "Missing argument"),
                };
                let Some(this) = arg.as_any().downcast_ref::<ArrayBuffer>() else {
                    throw!(cx.scope, TypeError, "Incompatible receiver")
                };

                const REQUIRED_ALIGN: usize = $kind.bytes_per_element();

                #[allow(clippy::modulo_one)]
                if this.len() % REQUIRED_ALIGN != 0 {
                    throw!(
                        cx.scope,
                        RangeError,
                        "Length of array buffer must be a multiple of {}",
                        REQUIRED_ALIGN
                    );
                }

                let array = TypedArray::new(cx.scope, arg.clone(), $kind);

                Ok(cx.scope.register(array).into())
            }
        }
    };
}

pub fn fill(cx: CallContext) -> Result<Value, Value> {
    let this = match cx.this.downcast_ref::<TypedArray>() {
        Some(this) => this,
        None => throw!(cx.scope, TypeError, "Invalid receiver"),
    };
    let value = match cx.args.first() {
        Some(value) => value.to_number(cx.scope)?,
        None => throw!(cx.scope, TypeError, "Missing fill value"), // TODO: shouldn't throw
    };
    let buf = this.buffer().as_any().downcast_ref::<ArrayBuffer>().unwrap().storage();

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
