use super::todo;
use crate::{
    gc::Handle,
    vm::value::{function::CallContext, Value},
};

/// The WeakSet constructor
///
/// https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-weakset-constructor
pub fn weakset_constructor(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo("WeakSet", ctx.vm)
}

/// Implements WeakSet.prototype.has
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-weakset.prototype.has
pub fn has(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo("WeakSet.prototype.has", ctx.vm)
}

/// Implements WeakSet.prototype.add
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-weakset.prototype.add
pub fn add(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo("WeakSet.prototype.add", ctx.vm)
}

/// Implements WeakSet.prototype.delete
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-weakset.prototype.delete
pub fn delete(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo("WeakSet.prototype.delete", ctx.vm)
}
