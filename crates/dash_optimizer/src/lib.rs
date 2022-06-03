use dash_middle::parser::statement::Statement;

use self::consteval::Eval;

pub mod consteval;

#[derive(Debug, Copy, Clone)]
pub enum OptLevel {
    None,
    Basic,
    Aggressive,
}

impl OptLevel {
    pub fn enabled(&self) -> bool {
        matches!(self, OptLevel::Basic | OptLevel::Aggressive)
    }
}

impl Default for OptLevel {
    fn default() -> Self {
        Self::Basic
    }
}

impl OptLevel {
    pub fn from_level(s: &str) -> Option<Self> {
        match s {
            "0" => Some(Self::None),
            "1" => Some(Self::Basic),
            "2" => Some(Self::Aggressive),
            _ => None,
        }
    }
}

pub fn optimize_ast<'a>(stmts: &mut Vec<Statement<'a>>, opt: OptLevel) {
    if opt.enabled() {
        stmts.fold(true);
    }
}
