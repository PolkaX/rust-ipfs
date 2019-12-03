use crate::{Link, Node};
use cid::Cid;

use crate::error::*;
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

pub trait LinkGetter<T: Node>: NodeGetter<T> {
    fn get_links(node: &Cid) -> Result<Vec<Link>>;
}

pub trait DAGService<T: Node>: NodeGetter<T> + NodeAdder<T> {
    fn remove(cid: &Cid) -> Result<()>;
}
