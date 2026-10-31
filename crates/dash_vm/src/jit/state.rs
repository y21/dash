use dash_middle::compiler::constant::Function;
use rustc_hash::FxHashMap;

use crate::frame::Ip;
use crate::jit::JitFnHandle;

pub struct State {
    pub(super) compiled_fn_cache: FxHashMap<(*const Function, Ip), JitFnHandle>,
}

impl State {
    pub fn new() -> Self {
        Self {
            compiled_fn_cache: FxHashMap::default(),
        }
    }
}
