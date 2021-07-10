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
/// Implements `WeakMap`
pub mod weakmap;
/// Implements `WeakSet`
pub mod weakset;

/// Unwraps a native [CallResult]
///
/// This macro is a shorthand for either getting the value if a CallResult is ready, or
/// returning the callresult from this function
#[macro_export]
macro_rules! unwrap_call_result {
    ($e:expr) => {
        match $e? {
            CallResult::Ready(value) => value,
            CallResult::UserFunction(func, args) => return Ok(CallResult::UserFunction(func, args)),
        }
    };
}
