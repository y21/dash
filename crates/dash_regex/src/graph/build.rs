use core::slice;
use std::collections::HashMap;

use crate::graph::node::{BuildGraph, CharacterClassItem, Node, NodeId, NodeKind};
use crate::node::{CharacterClassItem as ParsedCharacterClassItem, GroupCaptureMode};

use crate::node::Node as ParseNode;
use crate::parser::ParsedRegex;

use super::node::Graph;

type CaptureGroupMap = HashMap<*const ParseNode, u32>;

pub fn number_groups(regex: &ParsedRegex) -> CaptureGroupMap {
    fn inner(map: &mut CaptureGroupMap, _counter: &mut u32, nodes: &[ParseNode]) {
        if let Some((node, rest)) = nodes.split_first() {
            match node {
                ParseNode::Group(id, nodes) => {
                    if let GroupCaptureMode::Id(id) = *id {
                        map.insert(node, id.try_into().unwrap());
                    }

                    inner(map, _counter, nodes);
                }
                ParseNode::Optional(node) => inner(map, _counter, slice::from_ref(&**node)),
                ParseNode::Or(left, right) => {
                    inner(map, _counter, left);
                    inner(map, _counter, right);
                }
                ParseNode::Repetition { node, .. } => inner(map, _counter, slice::from_ref(&**node)),
                ParseNode::AnyCharacter
                | ParseNode::MetaSequence(_)
                | ParseNode::LiteralCharacter(_)
                | ParseNode::CharacterClass(_)
                | ParseNode::Anchor(_) => {} // cannot contain group nodes
            }

            inner(map, _counter, rest);
        }
    }

    let mut map = HashMap::new();
    let counter = &mut 0;
    inner(&mut map, counter, &regex.nodes);
    map
}

pub fn build(group_numbers: &CaptureGroupMap, regex: &ParsedRegex) -> (Graph, Option<NodeId>) {
    fn lower_repetition(
        graph: &mut BuildGraph,
        group_numbers: &CaptureGroupMap,
        node: &ParseNode,
        min: u32,
        max: Option<u32>,
        next: Option<NodeId>,
    ) -> NodeId {
        let end_id = graph.push(Node {
            next,
            kind: NodeKind::RepetitionEnd {
                start: NodeId::DUMMY, // will be set later
            },
        });
        let inner_id = inner(graph, group_numbers, slice::from_ref(node), Some(end_id)).unwrap();
        let start_id = graph.push(Node {
            next,
            kind: NodeKind::RepetitionStart {
                min,
                max,
                inner: inner_id,
            },
        });
        let NodeKind::RepetitionEnd { start } = &mut graph[end_id].kind else {
            unreachable!()
        };
        *start = start_id;
        start_id
    }

    fn inner(
        graph: &mut BuildGraph,
        group_numbers: &CaptureGroupMap,
        nodes: &[ParseNode],
        outer_next: Option<NodeId>,
    ) -> Option<NodeId> {
        if let Some((current, rest)) = nodes.split_first() {
            let next = inner(graph, group_numbers, rest, outer_next);
            match *current {
                ParseNode::AnyCharacter => Some(graph.push(Node {
                    next,
                    kind: NodeKind::AnyCharacter,
                })),
                ParseNode::MetaSequence(meta) => Some(graph.push(Node {
                    next,
                    kind: NodeKind::Meta(meta),
                })),
                ParseNode::Repetition { ref node, min, max } => {
                    Some(lower_repetition(graph, group_numbers, node, min, max, next))
                }
                ParseNode::LiteralCharacter(literal) => Some(graph.push(Node {
                    next,
                    kind: NodeKind::Literal(literal),
                })),
                ParseNode::CharacterClass(ref parse_items) => {
                    let items = parse_items
                        .iter()
                        .map(|item| match *item {
                            ParsedCharacterClassItem::Node(ParseNode::AnyCharacter) => CharacterClassItem::AnyCharacter,
                            ParsedCharacterClassItem::Node(ParseNode::LiteralCharacter(literal)) => {
                                CharacterClassItem::Literal(literal)
                            }
                            ParsedCharacterClassItem::Node(ParseNode::MetaSequence(meta)) => {
                                CharacterClassItem::Meta(meta)
                            }
                            ParsedCharacterClassItem::Node(ref node) => {
                                panic!("cannot lower {node:?} in character class")
                            }
                            ParsedCharacterClassItem::Range(from, to) => CharacterClassItem::Range(from, to),
                        })
                        .collect::<Box<[_]>>();

                    Some(graph.push(Node {
                        next,
                        kind: NodeKind::CharacterClass(items),
                    }))
                }
                ParseNode::Anchor(anchor) => Some(graph.push(Node {
                    next,
                    kind: NodeKind::Anchor(anchor),
                })),
                ParseNode::Or(ref left, ref right) => {
                    let left = inner(graph, group_numbers, left, next).unwrap();
                    let right = inner(graph, group_numbers, right, next).unwrap();
                    Some(graph.push(Node {
                        next,
                        kind: NodeKind::Or(left, right),
                    }))
                }
                ParseNode::Optional(ref node) => Some(lower_repetition(graph, group_numbers, node, 0, Some(1), next)),
                ParseNode::Group(_, ref nodes) => {
                    let group_id = group_numbers.get(&(current as *const ParseNode)).copied();
                    let end = graph.push(Node {
                        next,
                        kind: NodeKind::GroupEnd { group_id },
                    });
                    let inner_id = inner(graph, group_numbers, nodes, Some(end)).unwrap();
                    Some(graph.push(Node {
                        next: Some(inner_id),
                        kind: NodeKind::GroupStart { group_id },
                    }))
                }
            }
        } else {
            outer_next
        }
    }

    let mut graph = BuildGraph::new();
    let root = inner(&mut graph, group_numbers, &regex.nodes, None);
    (graph.finalize(), root)
}
