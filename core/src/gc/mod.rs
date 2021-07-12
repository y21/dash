/// A tracing garbage collector
pub mod gc;
/// Handles to heap allocations that can be passed around
pub mod handle;
/// A collection of heap elements
pub mod heap;

pub use gc::*;
pub use handle::*;
