use crate::{
    gc::Handle,
    vm::value::{function::CallContext, Value},
};

/// Implements `Array`
pub mod array;
/// Implements `Boolean`
pub mod boolean;
/// Implements the non-standard console API
pub mod console;
/// Implements `Error`
pub mod error;
/// Implements `Function`
pub mod function;
/// Implements global functions
pub mod functions;
/// Implements generators
pub mod generator;
/// Implements `JSON`
pub mod json;
/// Implements `Math`
pub mod math;
/// Implements `Number`
pub mod number;
/// Implements `Object`
pub mod object;
/// Implements `Promise`
pub mod promise;
/// Implements `String`
pub mod string;
/// Implements `Symbol`
pub mod symbol;
/// Implements `WeakMap`
pub mod weakmap;
/// Implements `WeakSet`
pub mod weakset;

/// The identify function
///
/// Returns its `this` value
pub fn identity(ctx: CallContext) -> Result<Handle<Value>, Handle<Value>> {
    Ok(Value::unwrap_or_undefined(ctx.receiver, ctx.vm))
}
