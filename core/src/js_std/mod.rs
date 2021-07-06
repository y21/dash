pub mod array;
pub mod boolean;
pub mod console;
pub mod error;
pub mod function;
pub mod functions;
pub mod json;
pub mod math;
pub mod number;
pub mod object;
pub mod string;
pub mod weakmap;
pub mod weakset;

#[macro_export]
macro_rules! unwrap_call_result {
    ($e:expr) => {
        match $e? {
            CallResult::Ready(value) => value,
            CallResult::UserFunction(func, args) => return Ok(CallResult::UserFunction(func, args)),
        }
    };
}
