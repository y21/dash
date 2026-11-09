use dash_middle::compiler::scope::BackLocalId;
use rustc_hash::FxHashSet;

use crate::jit::dataflow::DataflowEngine;
use crate::jit::dataflow::value::ValueType;
use crate::jit::ir::Body;

/// Instruction specialization
mod instspec;

pub type Assumptions = FxHashSet<Assumption>;

pub struct OptCtxt<'a> {
    pub body: &'a mut Body,
    pub dataflow: &'a mut DataflowEngine,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum Assumption {
    LocalHasType { local: BackLocalId, ty: ValueType },
}

pub enum PassResult {
    Changed,
    Unchanged,
}

pub fn run(body: &mut Body, dataflow: &mut DataflowEngine) {
    let passes = [instspec::run];

    loop {
        let mut changed = false;

        for &pass in &passes {
            let mut cx = OptCtxt { body, dataflow };
            match pass(&mut cx) {
                PassResult::Changed => changed = true,
                PassResult::Unchanged => (),
            }
        }

        if !changed {
            break;
        }
    }
}
