use crate::value::function::native::CallContext;
use crate::value::Value;

pub mod array;
pub mod array_iterator;
pub mod arraybuffer;
pub mod boolean;
pub mod date;
pub mod error;
pub mod function;
pub mod generator;
pub mod global;
pub mod map;
pub mod math;
pub mod number;
pub mod object;
pub mod promise;
pub mod regex;
pub mod set;
pub mod string;
pub mod symbol;
pub mod typedarray;

pub fn identity_this(cx: CallContext) -> Result<Value, Value> {
    Ok(cx.this)
}
