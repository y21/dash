use std::rc::Rc;

use dash_middle::compiler::constant::Function;
use rustc_hash::FxHashMap;

use crate::frame::Ip;
use crate::jit::mmap::MmapFn;

pub struct State {
    pub(super) compiled_fn_cache: FxHashMap<(*const Function, Ip), Rc<MmapFn>>,
}

impl State {
    pub fn new() -> Self {
        Self {
            compiled_fn_cache: FxHashMap::default(),
        }
    }
}
