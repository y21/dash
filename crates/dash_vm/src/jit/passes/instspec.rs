use crate::jit::dataflow::DataflowEngine;
use crate::jit::dataflow::value::ValueType;
use crate::jit::ir::{Body, GenericStmt, Stmt, ValueSource};
use crate::jit::passes::PassResult;

pub fn run(body: &mut Body, dataflow: &mut DataflowEngine) -> PassResult {
    println!("Running InstSpec pass");

    let mut result = PassResult::Unchanged;

    for block in body.blocks.iter_indices() {
        let mut cursor = dataflow.block_cursor_mut(block);

        for stmt in body.blocks[block].stmts.iter_indices() {
            let state = cursor.seek_to_before(body, stmt);

            match body.blocks[block].stmts[stmt] {
                Stmt::Generic(GenericStmt::Add, _) => {
                    let [.., lhs, rhs] = &*state.stack else {
                        panic!("Stack mismatch")
                    };

                    if lhs.ty == rhs.ty && lhs.ty == ValueType::Number {
                        body.blocks[block].stmts[stmt] = Stmt::UncheckedNumberAdd {
                            lhs: ValueSource::Stack,
                            rhs: ValueSource::Stack,
                        };
                        result = PassResult::Changed;
                        cursor.invalidate_block(body, stmt);
                    }
                }
                Stmt::Generic(GenericStmt::Lt, _) => {
                    let [.., lhs, rhs] = &*state.stack else {
                        panic!("Stack mismatch")
                    };

                    if lhs.ty == rhs.ty && lhs.ty == ValueType::Number {
                        body.blocks[block].stmts[stmt] = Stmt::UncheckedNumberLt {
                            lhs: ValueSource::Stack,
                            rhs: ValueSource::Stack,
                        };
                        result = PassResult::Changed;
                        cursor.invalidate_block(body, stmt);
                    }
                }
                _ => {}
            }
        }

        cursor.commit(body);
    }

    result
}
