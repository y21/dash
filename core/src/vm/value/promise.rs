use crate::gc::Handle;

use super::Value;

#[derive(Debug, Clone)]
/// The state of a promise
pub enum PromiseState<P> {
    /// Promise resolved successfully, the value can be read.
    Resolved(P),
    /// Promise rejected, the value can be read.
    Rejected(P),
    /// Promise is still pending.
    Pending,
}

/// A JavaScript promise
// TODO: handler fields should be a Vec, as one can attach multiple handlers
#[derive(Debug, Clone)]
pub struct Promise {
    /// The value of this promise
    pub value: PromiseState<Handle<Value>>,
    /// The `then` handler of this promise
    pub then: Option<Handle<Value>>,
    /// The `catch` handler of this promise
    pub catch: Option<Handle<Value>>,
    /// The `finally` handler of this promise
    pub finally: Option<Handle<Value>>,
}

impl Promise {
    /// Creates a new promise
    pub fn new(value: PromiseState<Handle<Value>>) -> Self {
        Self {
            value,
            then: None,
            catch: None,
            finally: None,
        }
    }
}
