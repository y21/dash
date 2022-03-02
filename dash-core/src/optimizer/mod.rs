use crate::parser::statement::Statement;

use self::consteval::Eval;
use self::consteval::OptLevel;

pub mod consteval;

pub fn optimize_ast<'a>(stmts: &mut Vec<Statement<'a>>, opt: OptLevel) {
    let len = stmts.len();
    if matches!(opt, OptLevel::Basic | OptLevel::Aggressive) && len >= 1 {
        stmts[..len].fold(true);
    }
}
