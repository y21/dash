use crate::jit::dataflow::StmtOrTerminator;
use crate::jit::dataflow::value::ValueType;
use crate::jit::ir::{ConditionalGotoCondition, GenericStmt, Stmt, Terminator, ValueSource};
use crate::jit::passes::{OptCtxt, PassResult};

pub fn run(cx: &mut OptCtxt) -> PassResult {
    let mut result = PassResult::Unchanged;

    for block in cx.body.blocks.iter_indices() {
        let mut cursor = cx.dataflow.block_cursor_mut(cx.body, block);

        for stmt in cx.body.blocks[block].stmts.iter_indices() {
            let state = cursor.seek_to_before(cx.body, StmtOrTerminator::Stmt(stmt));

            match cx.body.blocks[block].stmts[stmt] {
                Stmt::Generic(GenericStmt::Add, _) => {
                    let [.., lhs, rhs] = &*state.stack else {
                        panic!("Stack mismatch")
                    };

                    if lhs.ty == rhs.ty && lhs.ty == ValueType::Number {
                        cx.body.blocks[block].stmts[stmt] = Stmt::UncheckedNumberAdd {
                            lhs: ValueSource::Stack,
                            rhs: ValueSource::Stack,
                        };
                        result = PassResult::Changed;
                        cursor.invalidate_block(cx.body, StmtOrTerminator::Stmt(stmt));
                    }
                }
                Stmt::Generic(GenericStmt::Lt, _) => {
                    let [.., lhs, rhs] = &*state.stack else {
                        panic!("Stack mismatch")
                    };

                    if lhs.ty == rhs.ty && lhs.ty == ValueType::Number {
                        cx.body.blocks[block].stmts[stmt] = Stmt::UncheckedNumberLt {
                            lhs: ValueSource::Stack,
                            rhs: ValueSource::Stack,
                        };
                        result = PassResult::Changed;
                        cursor.invalidate_block(cx.body, StmtOrTerminator::Stmt(stmt));
                    }
                }
                _ => {}
            }
        }

        let state = cursor.seek_to_before(cx.body, StmtOrTerminator::Terminator);
        let terminator = cx.body.blocks[block].terminator_mut();
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
                    cursor.invalidate_block(cx.body, StmtOrTerminator::Terminator);
                }
                ConditionalGotoCondition::IsTruthy if is_boolean => {
                    *condition = ConditionalGotoCondition::IsTrue;
                    result = PassResult::Changed;
                    cursor.invalidate_block(cx.body, StmtOrTerminator::Terminator);
                }
                _ => {}
            }
        }

        cursor.commit(cx.body);
    }

    result
}
