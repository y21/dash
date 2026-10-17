use crate::jit::dataflow::DataflowEngine;
use crate::jit::ir::Body;

/// Instruction specialization
mod instspec;

enum PassResult {
    Changed,
    Unchanged,
}

pub fn run(body: &mut Body, dataflow: &mut DataflowEngine) {
    let passes = [instspec::run];

    loop {
        let mut changed = false;

        for &pass in &passes {
            match pass(body, dataflow) {
                PassResult::Changed => changed = true,
                PassResult::Unchanged => (),
            }
        }

        if !changed {
            break;
        }
    }
}
