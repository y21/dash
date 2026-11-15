use crate::jit::dataflow::StmtOrTerminator;
use crate::jit::dataflow::value::ValueType;
use crate::jit::ir::{ConditionalGotoCondition, GenericStmt, Stmt, Terminator, UncheckedNumberBinOp, ValueSource};
use crate::jit::passes::{OptCtxt, PassResult};

pub fn run(cx: &mut OptCtxt) -> PassResult {
    let mut result = PassResult::Unchanged;

    for block in cx.body.blocks.iter_indices() {
        let mut cursor = cx.dataflow.block_cursor_mut(cx.body, block);

        for stmt in cx.body.blocks[block].stmts.iter_indices() {
            let state = cursor.seek_to_before(cx.body, StmtOrTerminator::Stmt(stmt));

            let specialized_op = match &cx.body.blocks[block].stmts[stmt] {
                Stmt::Generic(GenericStmt::Add, _) => Some(UncheckedNumberBinOp::Add),
                Stmt::Generic(GenericStmt::Sub, _) => Some(UncheckedNumberBinOp::Sub),
                Stmt::Generic(GenericStmt::Mul, _) => Some(UncheckedNumberBinOp::Mul),
                Stmt::Generic(GenericStmt::Div, _) => Some(UncheckedNumberBinOp::Div),
                Stmt::Generic(GenericStmt::Gt, _) => Some(UncheckedNumberBinOp::Gt),
                Stmt::Generic(GenericStmt::Ge, _) => Some(UncheckedNumberBinOp::Ge),
                Stmt::Generic(GenericStmt::Lt, _) => Some(UncheckedNumberBinOp::Lt),
                Stmt::Generic(GenericStmt::Le, _) => Some(UncheckedNumberBinOp::Le),
                Stmt::Generic(GenericStmt::Eq, _) => Some(UncheckedNumberBinOp::Eq),
                Stmt::Generic(GenericStmt::Ne, _) => Some(UncheckedNumberBinOp::Ne),
                _ => None,
            };

            if let Some(op) = specialized_op {
                let [.., lhs, rhs] = &*state.stack else {
                    panic!("Stack mismatch")
                };

                if lhs.ty == rhs.ty && lhs.ty == ValueType::Number {
                    cx.body.blocks[block].stmts[stmt] = Stmt::UncheckedNumberBinOp {
                        op,
                        lhs: ValueSource::Stack,
                        rhs: ValueSource::Stack,
                    };
                    result = PassResult::Changed;
                    cursor.invalidate_block(cx.body, StmtOrTerminator::Stmt(stmt));
                }
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
