use std::sync::Arc;

use crate::error::*;

pub struct Walker {
    stack: Vec<Arc<dyn NavigableNode>>,
}

pub trait NavigableNode {
    fn child_total(&self) -> usize;
    fn fetch_child(&self, child_index: usize) -> Result<Arc<dyn NavigableNode>>;
}

impl Walker {
    pub fn new(root: Arc<dyn NavigableNode>) -> Walker {
        Walker { stack: vec![root] }
    }

    /// ActiveNode returns the `NavigableNode` that `Walker` is pointing
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
