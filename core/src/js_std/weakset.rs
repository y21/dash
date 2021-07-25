use crate::{
    gc::Handle,
    vm::value::{function::CallContext, Value},
};

/// The WeakSet constructor
///
/// https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-weakset-constructor
pub fn weakset_constructor(_ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo!()
}

/// Implements WeakSet.prototype.has
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-weakset.prototype.has
pub fn has(_ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo!()
}

/// Implements WeakSet.prototype.add
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-weakset.prototype.add
pub fn add(_ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo!()
}

/// Implements WeakSet.prototype.delete
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-weakset.prototype.delete
pub fn delete(_ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo!()
}
