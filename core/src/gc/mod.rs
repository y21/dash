/// A tracing garbage collector
pub mod gc;
/// Handles to heap allocations that can be passed around
pub mod handle;
/// A collection of heap elements
pub mod heap;
/// GC visitor, for traversing reachable objects
pub mod visitor;

pub use gc::*;
pub use handle::*;
pub use visitor::*;
