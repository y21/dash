use std::rc::Rc;

mod frontend;
pub mod legacy;
mod query;
use dash_log::debug;
use dash_log::error;
use dash_log::warn;
use dash_typed_cfg::passes::bb_generation::ConditionalBranchAction;
pub use frontend::Frontend;
use frontend::Trace;

use crate::Vm;
