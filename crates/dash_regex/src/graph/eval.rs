use crate::graph::node::CharacterClassItem;
use crate::node::Anchor;

use super::Regex;
use super::node::{Graph, NodeId, NodeKind};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ProcessedGroupState {
    Confirmed,
    Unconfirmed,
}

struct Cx<'a> {
    processed_groups: &'a mut [Option<(u32, u32, ProcessedGroupState)>],
    pending_groups: &'a mut [(Option<u32>, Option<u32>)],
    /// The full input source of this "attempt".
    full_input: &'a [u8],
    graph: &'a Graph,
    /// The offset of `full_input` in the *original* input string.
    offset_from_original: u32,
    current_repetition_count: Option<u32>,
}

impl Cx<'_> {
    /// Returns the offset of the passed in slice relative to the full input.
    /// The slice must actually be obtained from the full input for the return value to make sense.
    /// The value is unspecified (but not undefined) if passed an input slice from somewhere else.
    pub fn offset(&self, s: &[u8]) -> u32 {
        (s.as_ptr().addr() - self.full_input.as_ptr().addr()) as u32
    }

    /// Same as `offset`, but returns it relative to the original input.
    pub fn offset_from_original(&self, s: &[u8]) -> u32 {
        self.offset_from_original + self.offset(s)
    }

    /// Creates a new context usable for the specified node.
    pub fn for_node(&mut self, node: NodeId, origin: NodeId) -> Cx<'_> {
        let Self {
            processed_groups: &mut ref mut processed_groups,
            pending_groups: &mut ref mut pending_groups,
            full_input,
            graph,
            offset_from_original,
            mut current_repetition_count,
        } = *self;

        if let NodeKind::RepetitionStart { .. } = graph[node].kind {
            if let NodeKind::RepetitionEnd { .. } = graph[origin].kind {
                current_repetition_count = Some(current_repetition_count.unwrap() + 1);
            } else {
                current_repetition_count = Some(0);
            }
        }

        Cx {
            processed_groups,
            pending_groups,
            full_input,
            graph,
            offset_from_original,
            current_repetition_count,
        }
    }
}

fn step(mut cx: Cx, node_id: NodeId, mut input: &[u8]) -> bool {
    // The reason for shadowing cx with a borrow here is so that you're forced to go through `Cx::for_node` when calling `step(...)`.
    // You can't pass the same `cx` when evaluating a sub-node.
    let cx = &mut cx;
    let node = &cx.graph[node_id];

    let mut matches = match node.kind {
        NodeKind::AnyCharacter => {
            if let Some(rest) = input.get(1..) {
                input = rest;
                true
            } else {
                false
            }
        }
        NodeKind::RepetitionStart { min, max, inner } => 'arm: {
            let current_repetition_count = cx.current_repetition_count.unwrap();

            if let Some(max) = max {
                if current_repetition_count >= max {
                    // We've done `max` number of iterations.
                    break 'arm true;
                }
            }

            if step(cx.for_node(inner, node_id), inner, input) {
                // This has automatically also checked the rest input. Don't need to do that again here after the match.
                return true;
            }
            current_repetition_count >= min
        }
        NodeKind::Anchor(Anchor::StartOfString) => input.len() == cx.full_input.len(),
        NodeKind::Anchor(Anchor::EndOfString) => input.is_empty(),
        NodeKind::Meta(meta) => {
            if let Some((_, rest)) = input.split_first().filter(|&(&c, _)| meta.matches(c)) {
                input = rest;
                true
            } else {
                false
            }
        }
        NodeKind::CharacterClass(ref items) => {
            if let Some((_, rest)) = input.split_first().filter(|&(&c, _)| {
                items.iter().copied().any(|item| match item {
                    CharacterClassItem::Literal(lit) => lit == c,
                    CharacterClassItem::AnyCharacter => true,
                    CharacterClassItem::Meta(meta) => meta.matches(c),
                    CharacterClassItem::Range(min, max) => (min..=max).contains(&c),
                })
            }) {
                input = rest;
                true
            } else {
                false
            }
        }
        NodeKind::Literal(lit) => {
            if let Some((_, rest)) = input.split_first().filter(|&(&c, _)| c == lit) {
                input = rest;
                true
            } else {
                false
            }
        }
        NodeKind::Or(left, right) => {
            return step(cx.for_node(left, node_id), left, input) || step(cx.for_node(right, node_id), right, input);
        }
        NodeKind::RepetitionEnd { start } => {
            return step(cx.for_node(start, node_id), start, input);
        }
        NodeKind::GroupStart { group_id } => {
            if let Some(group_id) = group_id {
                let offset = cx.offset_from_original(input);
                cx.pending_groups[group_id as usize] = (Some(offset), None);
            }
            true
        }
        NodeKind::GroupEnd { group_id } => {
            if let Some(group_id) = group_id {
                let group_id = group_id as usize;

                let old = cx.processed_groups[group_id];
                let start = cx.pending_groups[group_id].0.unwrap();
                let end = cx.offset_from_original(input);
                cx.processed_groups[group_id] = Some((start, end, ProcessedGroupState::Unconfirmed));

                return if let Some(next) = node.next {
                    let matches = step(cx.for_node(next, node_id), next, input);
                    cx.pending_groups[group_id] = (Some(start), Some(end));

                    if matches {
                        if cx.processed_groups[group_id].is_none_or(|(.., s)| s == ProcessedGroupState::Unconfirmed) {
                            // This group may have been processed again in a subsequent iteration.
                            // Only overwrite it back with this iteration's if it's still unconfirmed
                            cx.processed_groups[group_id] = Some((start, end, ProcessedGroupState::Confirmed));
                        }

                        true
                    } else {
                        // We did not match. Restore to old.
                        if let Some((a, b, _)) = old {
                            cx.processed_groups[group_id] = Some((a, b, ProcessedGroupState::Unconfirmed));
                        } else {
                            cx.processed_groups[group_id] = None;
                        }
                        false
                    }
                } else {
                    // No next node.
                    cx.processed_groups[group_id].as_mut().unwrap().2 = ProcessedGroupState::Confirmed;
                    true
                };
            }

            true
        }
    };

    if let Some(next) = node.next {
        matches = matches && step(cx.for_node(next, node_id), next, input);
    }
    matches
}

#[derive(Debug)]
pub struct EvalSuccess {
    pub groups: Box<[Option<(u32, u32, ProcessedGroupState)>]>,
}

#[derive(Debug)]
pub struct NoMatch;

pub fn eval(regex: &Regex, mut input: &[u8]) -> Result<EvalSuccess, NoMatch> {
    let Some(root) = regex.root else {
        // Nothing to do for empty regexes.
        return Ok(EvalSuccess { groups: Box::default() });
    };

    let mut processed_groups = vec![None; regex.group_count as usize].into_boxed_slice();
    let mut pending_groups = vec![(None, None); regex.group_count as usize].into_boxed_slice();
    let mut offset_from_original = 0;
    loop {
        // TODO: add a fast reject path where we find the first required character and seek to it in `input`
        processed_groups[0] = Some((
            offset_from_original,
            offset_from_original + input.len() as u32,
            ProcessedGroupState::Confirmed,
        ));
        processed_groups[1..].fill(None);
        pending_groups.fill((None, None));

        let cx = Cx {
            processed_groups: &mut processed_groups,
            pending_groups: &mut pending_groups,
            current_repetition_count: if let NodeKind::RepetitionStart { .. } = regex.graph[root].kind {
                Some(0)
            } else {
                None
            },
            offset_from_original,
            full_input: input,
            graph: &regex.graph,
        };

        if step(cx, root, input) {
            return Ok(EvalSuccess {
                groups: processed_groups,
            });
        }

        if let Some(rest) = input.get(1..) {
            offset_from_original += 1;
            input = rest;
        } else {
            break;
        }
    }

    Err(NoMatch)
}
