#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct External {
    /// The id of the referenced local in the upper scope
    pub id: u16,
    /// Whether the referenced value is an external itself (i.e. from 2 or more upper scopes)
    ///
    /// This tells the VM where to load the local from when evaluating a function expression/statement.
    /// If this is false, it will load it from the local store
    /// If this is true, it will load it from the externals store
    pub is_nested_external: bool,
}
