use crate::jit::dataflow::value::ValueType;
use crate::jit::dataflow::{DataflowEngine, StmtOrTerminator};
use crate::jit::ir::{Body, ConditionalGotoCondition, GenericStmt, Stmt, Terminator, ValueSource};
use crate::jit::passes::PassResult;

pub fn run(body: &mut Body, dataflow: &mut DataflowEngine) -> PassResult {
    let mut result = PassResult::Unchanged;

    for block in body.blocks.iter_indices() {
        let mut cursor = dataflow.block_cursor_mut(body, block);

        for stmt in body.blocks[block].stmts.iter_indices() {
            let state = cursor.seek_to_before(body, StmtOrTerminator::Stmt(stmt));

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
                        cursor.invalidate_block(body, StmtOrTerminator::Stmt(stmt));
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
                        cursor.invalidate_block(body, StmtOrTerminator::Stmt(stmt));
                    }
                }
                _ => {}
            }
        }

        let state = cursor.seek_to_before(body, StmtOrTerminator::Terminator);
        let terminator = body.blocks[block].terminator_mut();
        if let Terminator::ConditionalGoto {
            true_: _,
            false_: _,
            pop: _,
            condition,
        } = terminator
        {
            let last_value = state.stack.last().unwrap();
            let is_boolean = matches!(last_value.ty, ValueType::Boolean);

            match condition {
                ConditionalGotoCondition::IsFalsy if is_boolean => {
                    *condition = ConditionalGotoCondition::IsFalse;
                    result = PassResult::Changed;
                    cursor.invalidate_block(body, StmtOrTerminator::Terminator);
                }
                ConditionalGotoCondition::IsTruthy if is_boolean => {
                    *condition = ConditionalGotoCondition::IsTrue;
                    result = PassResult::Changed;
                    cursor.invalidate_block(body, StmtOrTerminator::Terminator);
                }
                _ => {}
            }
        }

        cursor.commit(body);
    }

    result
}
