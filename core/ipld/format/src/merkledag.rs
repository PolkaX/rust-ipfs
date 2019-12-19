// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use cid::Cid;

use crate::error::Result;
use crate::format::{Link, Node};

/// The basic Node resolution service.
pub trait NodeGetter<T: Node> {
    /// Get retrieves nodes by CID. Depending on the NodeGetter
    /// implementation, this may involve fetching the Node from a remote
    /// machine; consider setting a deadline to stop it.
    fn get(&self, cid: &Cid) -> T;
}

/// NodeAdder adds nodes to a DAG.
pub trait NodeAdder<T: Node> {
    /// Add adds a node to this DAG.
    fn add(&self, node: T);
}

/// NodeGetters can optionally implement this interface to make finding linked objects faster.
pub trait LinkGetter<T: Node>: NodeGetter<T> {
    /// returns the children of the node referred to by the given CID.
    fn get_links(node: &Cid) -> Result<Vec<Link>>;
}

/// DAGService is an IPFS Merkle DAG service.
pub trait DAGService<T: Node>: NodeGetter<T> + NodeAdder<T> {
    /// remove a node, referred to by the given CID, from this DAG.
    fn remove(cid: &Cid) -> Result<()>;
}
