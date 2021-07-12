/// A tracing garbage collector
mod gc;
/// Handles to heap allocations that can be passed around
mod handle;
/// A collection of heap elements
mod heap;

use gc::*;
use handle::*;
