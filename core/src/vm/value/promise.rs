use std::{cell::RefCell, rc::Rc};

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
    pub value: PromiseState<Rc<RefCell<Value>>>,
    /// The `then` handler of this promise
    pub then: Option<Rc<RefCell<Value>>>,
    /// The `catch` handler of this promise
    pub catch: Option<Rc<RefCell<Value>>>,
    /// The `finally` handler of this promise
    pub finally: Option<Rc<RefCell<Value>>>,
}

impl Promise {
    /// Creates a new promise
    pub fn new(value: PromiseState<Rc<RefCell<Value>>>) -> Self {
        Self {
            value,
            then: None,
            catch: None,
            finally: None,
        }
    }
}
