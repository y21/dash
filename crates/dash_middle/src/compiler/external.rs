
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct External {
    /// The id of the referenced local in the upper scope
    pub id: u16,
    /// Whether the referenced value is an external itself (i.e. from 2 or more upper scopes)
    pub is_external: bool,
}
