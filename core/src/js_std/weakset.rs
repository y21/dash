use super::todo;
use crate::vm::value::function::{CallContext, NativeFunctionCallbackResult};

/// The WeakSet constructor
///
/// https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-weakset-constructor
pub fn weakset_constructor(ctx: CallContext) -> NativeFunctionCallbackResult {
    todo("WeakSet", ctx.vm)
}

/// Implements WeakSet.prototype.has
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-weakset.prototype.has
pub fn has(ctx: CallContext) -> NativeFunctionCallbackResult {
    todo("WeakSet.prototype.has", ctx.vm)
}

/// Implements WeakSet.prototype.add
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-weakset.prototype.add
pub fn add(ctx: CallContext) -> NativeFunctionCallbackResult {
    todo("WeakSet.prototype.add", ctx.vm)
}

/// Implements WeakSet.prototype.delete
///
/// https://tc39.es/ecma262/multipage/text-processing.html#sec-weakset.prototype.delete
pub fn delete(ctx: CallContext) -> NativeFunctionCallbackResult {
    todo("WeakSet.prototype.delete", ctx.vm)
}
