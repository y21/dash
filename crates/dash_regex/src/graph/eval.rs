use std::cell::Cell;

use crate::graph::node::CharacterClassItem;
use crate::node::Anchor;

use super::Regex;
use super::node::{Graph, NodeId, NodeKind};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ProcessedGroupState {
    Confirmed,
    Unconfirmed,
}

struct Shared<'a> {
    processed_groups: &'a mut [Option<(u32, u32, ProcessedGroupState)>],
    pending_groups: &'a mut [(Option<u32>, Option<u32>)],
    /// The full input source of this "attempt".
    full_input: &'a [u8],
    graph: &'a Graph,
    /// The offset of `full_input` in the *original* input string.
    offset_from_original: u32,
}
impl Shared<'_> {
    /// Returns the offset of the passed in slice relative to the full input.
    /// The slice must actually be obtained from the full input for the return value to make sense.
    /// The value is unspecified (but not undefined) if passed an input slice from somewhere else.
    pub fn offset_of(&self, remaining: &[u8]) -> u32 {
        (remaining.as_ptr().addr() - self.full_input.as_ptr().addr()) as u32
    }

    /// Same as `offset`, but returns it relative to the original input.
    pub fn offset_of_from_original(&self, remaining: &[u8]) -> u32 {
        self.offset_from_original + self.offset_of(remaining)
    }
}

#[derive(Debug, Clone)]
struct Cx<'a> {
    /// How many iterations have matched so far
    current_repetition_count: Cell<Option<u32>>,
    /// The offset at the start of this iteration, used to determine if we're making any progress.
    /// If this is the same as the offset at the end of an iteration, we can return true early as it will match forever.
    current_repetition_start: Cell<Option<u32>>,
    parent: Option<&'a Cx<'a>>,
}

impl<'a> Cx<'a> {
    pub fn for_node(&'a self, shared: &Shared<'_>, target: NodeId, origin: NodeId, remaining: &[u8]) -> Cx<'a> {
        let mut current_repetition_count = self.current_repetition_count.clone();
        let mut current_repetition_start = self.current_repetition_start.clone();
        let mut parent = self.parent;

        // Moving to a RepetitionStart means we either prepare/initialize a repetition (set to 0),
        // or increment it if we're coming from a RepetitionEnd specifically.
        if let NodeKind::RepetitionStart { .. } = shared.graph[target].kind {
            current_repetition_start = Cell::new(Some(shared.offset_of(remaining)));

            if let NodeKind::RepetitionEnd { .. } = shared.graph[origin].kind {
                *current_repetition_count.get_mut().as_mut().unwrap() += 1;
            } else {
                current_repetition_count = Cell::new(Some(0));
                parent = Some(self);
            }
        }

        Cx {
            current_repetition_count,
            current_repetition_start,
            parent,
        }
    }
}

fn step(shared: &mut Shared<'_>, cx: Cx<'_>, node_id: NodeId, mut remaining: &[u8]) -> bool {
    // The reason for shadowing cx with a borrow here is so that you're forced to go through `Cx::for_node` when calling `step(...)`.
    // You can't pass the same `cx` when evaluating a sub-node.
    let mut cx = &cx;
    let node = &shared.graph[node_id];

    let mut matches = match node.kind {
        NodeKind::AnyCharacter => {
            if let Some(rest) = remaining.get(1..) {
                remaining = rest;
                true
            } else {
                false
            }
        }
        NodeKind::RepetitionStart { min, max, inner } => 'arm: {
            let current_repetition_count = cx.current_repetition_count.get().unwrap();

            if let Some(max) = max {
                if current_repetition_count >= max {
                    // We've done `max` number of iterations.
                    break 'arm true;
                }
            }

            if step(shared, cx.for_node(shared, inner, node_id, remaining), inner, remaining) {
                // This has automatically also checked the rest input. Don't (shouldn't) need to do that again here after the match.
                return true;
            }

            // Getting here means the regex cannot match the string with another repetition iteration,
            // and we are on track to backtrack.
            // This requires us to "pop" the current repetition and continue with the outer/parent repetition context,
            // as this might be a nested repetition.
            cx = cx.parent.unwrap();
            current_repetition_count >= min
        }
        NodeKind::Anchor(Anchor::StartOfString) => remaining.len() == shared.full_input.len(),
        NodeKind::Anchor(Anchor::EndOfString) => remaining.is_empty(),
        NodeKind::Meta(meta) => {
            if let Some((_, rest)) = remaining.split_first().filter(|&(&c, _)| meta.matches(c)) {
                remaining = rest;
                true
            } else {
                false
            }
        }
        NodeKind::CharacterClass(ref items) => {
            if let Some((_, rest)) = remaining.split_first().filter(|&(&c, _)| {
                items.iter().copied().any(|item| match item {
                    CharacterClassItem::Literal(lit) => lit == c,
                    CharacterClassItem::AnyCharacter => true,
                    CharacterClassItem::Meta(meta) => meta.matches(c),
                    CharacterClassItem::Range(min, max) => (min..=max).contains(&c),
                })
            }) {
                remaining = rest;
                true
            } else {
                false
            }
        }
        NodeKind::Literal(lit) => {
            if let Some((_, rest)) = remaining.split_first().filter(|&(&c, _)| c == lit) {
                remaining = rest;
                true
            } else {
                false
            }
        }
        NodeKind::Or(left, right) => {
            return step(shared, cx.for_node(shared, left, node_id, remaining), left, remaining)
                || step(shared, cx.for_node(shared, right, node_id, remaining), right, remaining);
        }
        NodeKind::RepetitionEnd { start } => {
            let end_off = shared.offset_of(remaining);
            if cx.current_repetition_start.get().unwrap() == end_off {
                // We haven't made any progress in this repetition iteration and won't.
                return true;
            } else {
                return step(shared, cx.for_node(shared, start, node_id, remaining), start, remaining);
            }
        }
        NodeKind::GroupStart { group_id } => {
            if let Some(group_id) = group_id {
                let offset = shared.offset_of_from_original(remaining);
                shared.pending_groups[group_id as usize] = (Some(offset), None);
            }
            true
        }
        NodeKind::GroupEnd { group_id } => {
            if let Some(group_id) = group_id {
                let group_id = group_id as usize;
                let old = shared.processed_groups[group_id];
                let start = shared.pending_groups[group_id].0.unwrap();
                let end = shared.offset_of_from_original(remaining);
                shared.processed_groups[group_id] = Some((start, end, ProcessedGroupState::Unconfirmed));

                return if let Some(next) = node.next {
                    let matches = step(shared, cx.for_node(shared, next, node_id, remaining), next, remaining);
                    shared.pending_groups[group_id] = (Some(start), Some(end));
                    if matches {
                        if shared.processed_groups[group_id].is_none_or(|(.., s)| s == ProcessedGroupState::Unconfirmed)
                        {
                            // This group may have been processed again in a subsequent iteration.
                            // Only overwrite it back with this iteration's if it's still unconfirmed
                            shared.processed_groups[group_id] = Some((start, end, ProcessedGroupState::Confirmed));
                        }
                        true
                    } else {
                        // We did not match. Restore to old.
                        if let Some((a, b, _)) = old {
                            shared.processed_groups[group_id] = Some((a, b, ProcessedGroupState::Unconfirmed));
                        } else {
                            shared.processed_groups[group_id] = None;
                        }
                        false
                    }
                } else {
                    // No next node.
                    shared.processed_groups[group_id].as_mut().unwrap().2 = ProcessedGroupState::Confirmed;
                    true
                };
            }
            true
        }
    };

    if let Some(next) = node.next {
        matches = matches && step(shared, cx.for_node(shared, next, node_id, remaining), next, remaining);
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

        let outer_cx: Cx<'_> = Cx {
            current_repetition_count: Cell::new(None),
            current_repetition_start: Cell::new(None),
            parent: None,
        };
        let (current_repetition_count, current_repetition_start, outer_cx) =
            if let NodeKind::RepetitionStart { .. } = regex.graph[root].kind {
                (Some(0), Some(0), Some(&outer_cx))
            } else {
                (None, None, None)
            };

        let mut shared = Shared {
            full_input: input,
            graph: &regex.graph,
            offset_from_original,
            pending_groups: &mut pending_groups,
            processed_groups: &mut processed_groups,
        };
        let cx = Cx {
            current_repetition_count: Cell::new(current_repetition_count),
            current_repetition_start: Cell::new(current_repetition_start),
            parent: outer_cx,
        };

        if step(&mut shared, cx, root, input) {
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
