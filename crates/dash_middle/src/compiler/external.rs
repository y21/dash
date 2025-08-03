#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::compiler::scope::BackLocalId;
use crate::index_type;

index_type! {
    #[derive(Debug, Copy, Clone, PartialEq)]
    #[cfg_attr(feature = "format", derive(Serialize, Deserialize))]
    pub struct ExternalId(pub u16);
}

/// A combination of either a local variable or an external variable
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PossiblyExternalId {
    Local(BackLocalId),
    External(ExternalId),
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct External {
    /// Either a [`BackLocalId`] if the referenced local is defined in this function, or,
    /// if this referenced variable is defined in an upper function, stores the [`ExternalId`]
    pub id: PossiblyExternalId,
}
