/// Splits up the bytecode into (a chain of) Basic Blocks
/// Each BB may have exactly one or two edges:
///  - One for unconditional branching
///  - Two for conditional branching (if/else)
///
/// and one predecessor edge (the parent)
pub mod bb_generation;
pub mod type_infer;
