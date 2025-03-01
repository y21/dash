use std::ops::{Deref, DerefMut, Index, IndexMut};

use crate::node::{Anchor, MetaSequence};

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "format", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeId(u32);
impl NodeId {
    pub(super) const DUMMY: NodeId = NodeId(u32::MAX);
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "format", derive(serde::Serialize, serde::Deserialize))]
pub struct Node {
    pub next: Option<NodeId>,
    pub kind: NodeKind,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "format", derive(serde::Serialize, serde::Deserialize))]
pub enum NodeKind {
    AnyCharacter,
    RepetitionStart {
        min: u32,
        max: Option<u32>,
        /// The node being repeated
        inner: NodeId,
    },
    Anchor(Anchor),
    Meta(MetaSequence),
    CharacterClass(Box<[CharacterClassItem]>),
    Literal(u8),
    Or(NodeId, NodeId),
    RepetitionEnd {
        /// The `RepetitionStart` node to jump to when executing the next repetition iteration
        start: NodeId,
    },
    GroupStart {
        group_id: Option<u32>,
    },
    GroupEnd {
        group_id: Option<u32>,
    },
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "format", derive(serde::Serialize, serde::Deserialize))]
pub enum CharacterClassItem {
    Literal(u8),
    AnyCharacter,
    Meta(MetaSequence),
    Range(u8, u8),
}

pub type BuildGraph = Graph<Vec<Node>>;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "format", derive(serde::Serialize, serde::Deserialize))]
pub struct Graph<C = Box<[Node]>> {
    nodes: C,
}

impl BuildGraph {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn push(&mut self, node: Node) -> NodeId {
        let id = u32::try_from(self.nodes.len()).expect("attempted to insert more than 2^32 nodes");
        self.nodes.push(node);
        NodeId(id)
    }

    pub fn finalize(self) -> Graph {
        Graph {
            nodes: self.nodes.into_boxed_slice(),
        }
    }
}

// Requires an indirection through the deref trait because `Box<[T]>` does not implement `Index<usize>`...
impl<C: Deref<Target: Index<usize, Output = Node>>> Index<NodeId> for Graph<C> {
    type Output = Node;
    fn index(&self, index: NodeId) -> &Self::Output {
        &self.nodes[index.0 as usize]
    }
}

impl<C: DerefMut<Target: IndexMut<usize, Output = Node>>> IndexMut<NodeId> for Graph<C> {
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        &mut self.nodes[index.0 as usize]
    }
}
