use crate::throw;
use crate::vm::value::arraybuffer::ArrayBuffer;
use crate::vm::value::function::native::CallContext;
use crate::vm::value::object::Object;
use crate::vm::value::typedarray::TypedArray;
use crate::vm::value::typedarray::TypedArrayKind;
use crate::vm::value::Value;

macro_rules! typedarray {
    (module: $module:ident, kind: $kind:expr) => {
        pub mod $module {
            use super::*;

            pub fn constructor(cx: CallContext) -> Result<Value, Value> {
                let this = match cx.args.first() {
                    Some(Value::Object(obj) | Value::External(obj)) => {
                        let this = obj.as_any();

                        if let Some(this) = this.downcast_ref::<ArrayBuffer>() {
                            const REQUIRED_ALIGN: usize = $kind.bytes_per_element();

                            if this.len() % REQUIRED_ALIGN != 0 {
                                throw!(
                                    cx.scope,
                                    "Length of array buffer must be a multiple of {}",
                                    REQUIRED_ALIGN
                                );
                            }

                            Some(obj.clone())
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                let this = match this {
                    Some(this) => this,
                    None => throw!(cx.scope, "Incompatible receiver"),
                };

                let array = TypedArray::new(cx.scope, this, $kind);
                Ok(cx.scope.register(array).into())
            }
        }
    };
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
