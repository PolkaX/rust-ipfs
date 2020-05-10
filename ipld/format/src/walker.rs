// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::sync::Arc;

use crate::error::{FormatError, Result};

/// An interface the nodes of a DAG need to implement in order to be traversed by the `Walker`.
pub trait NavigableNode {
    /// Returns the number of children of the `ActiveNode`.
    fn child_total(&self) -> usize;

    /// Returns the child of this node pointed to by `child_index`.
    fn fetch_child(&self, child_index: usize) -> Result<Arc<dyn NavigableNode>>;
}

/// Walker provides methods to move through a DAG of nodes that implement the `NavigableNode`
/// interface. It uses iterative algorithms (instead of recursive ones) that expose the `path`
/// of nodes from the root to the `active_node` it currently points to.
///
/// It provides multiple ways to walk through the DAG. When using them,
/// you provide a Visitor function that will be called for each node the Walker traverses.
/// The Visitor can read data from those nodes and, optionally, direct the movement of the Walker.
pub struct Walker {
    stack: Vec<Arc<dyn NavigableNode>>,
}

impl Walker {
    /// Creates a new `Walker` from a `root` NavigableNode.
    pub fn new(root: Arc<dyn NavigableNode>) -> Walker {
        Walker { stack: vec![root] }
    }

    /// Returns the `NavigableNode` that `Walker` is pointing
    /// to at the moment. It changes when `up` or `down` is called.
    pub fn active_node(&self) -> Result<&Arc<dyn NavigableNode>> {
        self.stack.last().ok_or(FormatError::NextNoChild)
    }
}

impl Iterator for Walker {
    type Item = Arc<dyn NavigableNode>;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        for i in (0..node.child_total()).rev() {
            let node = node.fetch_child(i).ok()?;
            self.stack.push(node);
        }
        Some(node)
    }
}
