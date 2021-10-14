use super::todo;
use crate::{
    gc::Handle,
    vm::value::{function::CallContext, Value},
};

/// The WeakMap constructor
///
/// https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-weakmap-constructor
pub fn weakmap_constructor(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo("WeakMap", ctx.vm)
}

/// Implements WeakMap.prototype.has
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-weakmap.prototype.has
pub fn has(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo("WeakMap.prototype.has", ctx.vm)
}

/// Implements WeakMap.prototype.get
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-weakmap.prototype.get
pub fn get(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo("WeakMap.prototype.get", ctx.vm)
}

/// Implements WeakMap.prototype.add
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-weakmap.prototype.add
pub fn add(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo("WeakMap.prototype.add", ctx.vm)
}

/// Implements WeakMap.prototype.delete
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-weakmap.prototype.delete
pub fn delete(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    todo("WeakMap.prototype.delete", ctx.vm)
}
