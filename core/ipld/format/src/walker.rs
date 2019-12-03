use crate::error::*;

pub struct Walker {
    path: Vec<Box<dyn NavigableNode>>,
    current_depth: Option<usize>,
    child_index: Vec<usize>,
    pause_requested: bool,
}

trait NavigableNode {
    fn child_total(&self) -> usize;
    fn fetch_child(&self, child_index: usize) -> Result<Box<dyn NavigableNode>>;
}

impl Walker {
    pub fn new_walker(root: Box<dyn NavigableNode>) -> Walker {
        Walker {
            path: vec![root],
            // None is "on top" of the root node
            current_depth: None,
            child_index: vec![0],
            pause_requested: false,
        }
    }

    /// ActiveNode returns the `NavigableNode` that `Walker` is pointing
    /// to at the moment. It changes when `up` or `down` is called.
    pub fn active_node(&self) -> Result<&Box<dyn NavigableNode>> {
        let dep = self.current_depth.ok_or(FormatError::DepthNotInit)?;
        Ok(self
            .path
            .get(dep)
            .ok_or(FormatError::DepthError(dep, self.path.len()))?)
    }

    fn extend_path(&mut self, child: Box<dyn NavigableNode>) {
        self.current_depth.map(|ref mut dep| *dep += 1);
    }
}
