use std::any::Any;

use dash_proc_macro::Trace;

use crate::gc::handle::Handle;
use crate::local::LocalScope;
use crate::Vm;

use super::arraybuffer::ArrayBuffer;
use super::object::delegate_get_property;
use super::object::NamedObject;
use super::object::Object;
use super::object::PropertyKey;
use super::object::PropertyValue;
use super::ops::abstractions::conversions::ValueConversion;
use super::Value;

#[derive(Debug)]
pub enum TypedArrayKind {
    Int8Array,
    Uint8Array,
    Uint8ClampedArray,
    Int16Array,
    Uint16Array,
    Int32Array,
    Uint32Array,
    Float32Array,
    Float64Array,
}

impl TypedArrayKind {
    pub const fn bytes_per_element(&self) -> usize {
        match self {
            TypedArrayKind::Int8Array => 1,
            TypedArrayKind::Uint8Array => 1,
            TypedArrayKind::Uint8ClampedArray => 1,
            TypedArrayKind::Int16Array => 2,
            TypedArrayKind::Uint16Array => 2,
            TypedArrayKind::Int32Array => 4,
            TypedArrayKind::Uint32Array => 4,
            TypedArrayKind::Float32Array => 4,
            TypedArrayKind::Float64Array => 8,
        }
    }
}

#[derive(Debug, Trace)]
pub struct TypedArray {
    arraybuffer: Handle<dyn Object>,
    kind: TypedArrayKind,
    obj: NamedObject,
}

impl TypedArray {
    pub fn new(vm: &mut Vm, arraybuffer: Handle<dyn Object>, kind: TypedArrayKind) -> Self {
        let (proto, ctor) = match kind {
            TypedArrayKind::Uint8Array => (&vm.statics.uint8array_prototype, &vm.statics.uint8array_ctor),
            TypedArrayKind::Uint8ClampedArray => (&vm.statics.uint8array_prototype, &vm.statics.uint8array_ctor),
            TypedArrayKind::Int8Array => (&vm.statics.int8array_prototype, &vm.statics.int8array_ctor),
            TypedArrayKind::Int16Array => (&vm.statics.int16array_prototype, &vm.statics.int16array_ctor),
            TypedArrayKind::Uint16Array => (&vm.statics.uint16array_prototype, &vm.statics.uint16array_ctor),
            TypedArrayKind::Int32Array => (&vm.statics.int32array_prototype, &vm.statics.int32array_ctor),
            TypedArrayKind::Uint32Array => (&vm.statics.uint32array_prototype, &vm.statics.uint32array_ctor),
            TypedArrayKind::Float32Array => (&vm.statics.float32array_prototype, &vm.statics.float32array_ctor),
            TypedArrayKind::Float64Array => (&vm.statics.float64array_prototype, &vm.statics.float64array_ctor),
        };

        Self {
            arraybuffer,
            kind,
            obj: NamedObject::with_prototype_and_constructor(proto.clone(), ctor.clone()),
        }
    }
}

impl Object for TypedArray {
    fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        delegate_get_property(self, sc, key)
    }

    fn get_property_descriptor(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Option<PropertyValue>, Value> {
        if let Some(Ok(index)) = key.as_string().map(|k| k.parse::<usize>()) {
            let arraybuffer = self.arraybuffer.as_any().downcast_ref::<ArrayBuffer>();

            if let Some(arraybuffer) = arraybuffer {
                let bytes = arraybuffer.storage();
                let index = index * self.kind.bytes_per_element();

                macro_rules! decode_from {
                    (ty: $ty:ty, size: $size:expr) => {
                        bytes
                            .get(index..index + $size)
                            .map(|x| {
                                let mut arr = [0; $size];
                                for (dest, src) in arr.iter_mut().zip(x.iter()) {
                                    *dest = src.get();
                                }
                                arr
                            })
                            .map(<$ty>::from_ne_bytes)
                            .map(f64::from)
                    };
                }

                let value = match self.kind {
                    TypedArrayKind::Int8Array => decode_from!(ty: i8, size: 1),
                    TypedArrayKind::Uint8Array => decode_from!(ty: u8, size: 1),
                    TypedArrayKind::Uint8ClampedArray => decode_from!(ty: u8, size: 1),
                    TypedArrayKind::Int16Array => decode_from!(ty: i16, size: 2),
                    TypedArrayKind::Uint16Array => decode_from!(ty: u16, size: 2),
                    TypedArrayKind::Int32Array => decode_from!(ty: i32, size: 4),
                    TypedArrayKind::Uint32Array => decode_from!(ty: u32, size: 4),
                    TypedArrayKind::Float32Array => decode_from!(ty: f32, size: 4),
                    TypedArrayKind::Float64Array => decode_from!(ty: f64, size: 8),
                };

                if let Some(value) = value {
                    return Ok(Some(PropertyValue::static_default(Value::number(value))));
                }
            }
        }

        self.obj.get_property_descriptor(sc, key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey<'static>, value: PropertyValue) -> Result<(), Value> {
        if let Some(Ok(index)) = key.as_string().map(|k| k.parse::<usize>()) {
            let arraybuffer = self.arraybuffer.as_any().downcast_ref::<ArrayBuffer>();

            // TODO: not undefined as this
            let value = value.kind().get_or_apply(sc, Value::undefined())?;
            let value = value.to_number(sc)?;
            if let Some(arraybuffer) = arraybuffer {
                let bytes = arraybuffer.storage();
                let index = index * self.kind.bytes_per_element();

                macro_rules! encode_into {
                    (ty: $ty:ty, size: $size:expr) => {{
                        let size = $size;
                        let dest = bytes.get(index..index + size);
                        let src = <$ty>::to_ne_bytes(value as $ty);

                        if let Some(dest) = dest {
                            assert!(dest.len() >= size);

                            for (dest, src) in dest.iter().zip(src.iter().copied()) {
                                dest.set(src);
                            }
                        }

                        return Ok(());
                    }};
                }

                match self.kind {
                    TypedArrayKind::Int8Array => encode_into!(ty: i8, size: 1),
                    TypedArrayKind::Uint8Array => encode_into!(ty: u8, size: 1),
                    TypedArrayKind::Uint8ClampedArray => encode_into!(ty: u8, size: 1),
                    TypedArrayKind::Int16Array => encode_into!(ty: i16, size: 2),
                    TypedArrayKind::Uint16Array => encode_into!(ty: u16, size: 2),
                    TypedArrayKind::Int32Array => encode_into!(ty: i32, size: 4),
                    TypedArrayKind::Uint32Array => encode_into!(ty: u32, size: 4),
                    TypedArrayKind::Float32Array => encode_into!(ty: f32, size: 4),
                    TypedArrayKind::Float64Array => encode_into!(ty: f64, size: 8),
                }
            }
        }

        self.obj.set_property(sc, key, value)
    }

    fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        self.obj.delete_property(sc, key)
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        self.obj.set_prototype(sc, value)
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        self.obj.get_prototype(sc)
    }

    fn apply(
        &self,
        scope: &mut LocalScope,
        callee: Handle<dyn Object>,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        self.obj.apply(scope, callee, this, args)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        self.obj.own_keys()
    }
}
