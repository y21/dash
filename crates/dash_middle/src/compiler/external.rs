#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct External {
    /// The id of the referenced local in the upper scope
    pub id: u16,
    /// Whether the referenced value is an external itself (i.e. from 2 or more upper scopes)
    pub is_external: bool,
}
