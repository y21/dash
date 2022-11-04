/// Marks the code path leading to this call as cold, or "unlikely"
#[cold]
pub fn cold_path() {}

/// Marks the boolean as unlikely to be true.
pub fn unlikely(b: bool) -> bool {
    if b {
        cold_path();
    }
    b
}
