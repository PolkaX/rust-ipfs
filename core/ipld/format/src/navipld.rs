use crate::merkledag::NodeGetter;
use crate::Node;
use cid::Cid;

fn get_link_cids<N: Node>(node: &N) -> Vec<Cid> {
    node.links().iter().map(|l| l.cid.clone()).collect()
}

pub struct NavigableIPLDNode<N: Node, NG: NodeGetter<N>> {
    node: N,
    node_getter: NG,
    child_cids: Vec<Cid>,
}

impl<N: Node, NG: NodeGetter<N>> NavigableIPLDNode<N, NG> {
    fn new(node: N, node_getter: NG) -> Self {
        let child_cids = get_link_cids(&node);
        NavigableIPLDNode {
            node,
            node_getter,
            child_cids,
        }
    }

    //    fn fetch_child(&self)

    /// returns the IPLD `Node` wrapped into this structure.
    pub fn get_ipld_node(&self) -> &N {
        &self.node
    }

    /// returning the number of links (of child nodes) in this node.
    pub fn child_total(&self) -> usize {
        self.node.links().len()
    }
}
