use crate::{
    gc::Handle,
    vm::value::{function::CallContext, Value},
};

/// The WeakMap constructor
///
/// https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-weakmap-constructor
pub fn weakmap_constructor(_ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo!()
}

/// Implements WeakMap.prototype.has
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-weakmap.prototype.has
pub fn has(_ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo!()
}

/// Implements WeakMap.prototype.get
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-weakmap.prototype.get
pub fn get(_ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo!()
}

/// Implements WeakMap.prototype.add
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-weakmap.prototype.add
pub fn add(_ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo!()
}

/// Implements WeakMap.prototype.delete
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-weakmap.prototype.delete
pub fn delete(_ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo!()
}
