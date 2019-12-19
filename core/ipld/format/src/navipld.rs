// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use cid::Cid;

use crate::format::Node;
use crate::merkledag::NodeGetter;

/// NavigableIPLDNode implements the `NavigableNode` interface wrapping an IPLD `Node`.
pub struct NavigableIpldNode<N: Node, NG: NodeGetter<N>> {
    node: N,
    node_getter: NG,
    child_cids: Vec<Cid>,
}

impl<N: Node, NG: NodeGetter<N>> NavigableIpldNode<N, NG> {
    /// Create a `NavigableIpldNode` wrapping the provided `node`.
    pub fn new(node: N, node_getter: NG) -> Self {
        let child_cids = node
            .links()
            .into_iter()
            .map(|link| link.cid.clone())
            .collect();
        NavigableIpldNode {
            node,
            node_getter,
            child_cids,
        }
    }

    /// Return the IPLD `Node` wrapped into this structure.
    pub fn ipld_node(&self) -> &N {
        &self.node
    }

    /// Return the number of links (of child nodes) in this node.
    pub fn child_total(&self) -> usize {
        self.node.links().len()
    }
}
