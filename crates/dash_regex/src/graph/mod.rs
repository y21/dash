mod build;
pub mod eval;
pub mod node;

use eval::{EvalSuccess, NoMatch};
use node::{Graph, NodeId};

use crate::Flags;
use crate::parser::ParsedRegex;

/// A finalized, compiled regex.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "format", derive(serde::Serialize, serde::Deserialize))]
pub struct Regex {
    graph: Graph,
    flags: Flags,
    root: NodeId,
    group_count: u32,
}

impl Regex {
    pub fn eval(&self, input: &str) -> Result<EvalSuccess, NoMatch> {
        eval::eval(self, input.as_bytes())
    }

    pub fn matches(&self, input: &str) -> bool {
        self.eval(input).is_ok()
    }

    pub fn flags(&self) -> Flags {
        self.flags
    }
}

pub fn compile(regex: ParsedRegex, flags: Flags) -> Regex {
    // We're going to have a hashmap with pointers as keys.
    // Accidentally moving the regex would invalidate pointers.
    // We never actually dereference them so it doesn't matter for safety, but it would still lead to
    // bugs. So make it a borrow.
    let regex = &regex;

    let numbered = build::number_groups(regex);
    let (graph, root) = build::build(&numbered, regex);
    let group_count = u32::try_from(regex.group_count).unwrap();

    Regex {
        graph,
        group_count,
        flags,
        root,
    }
}
