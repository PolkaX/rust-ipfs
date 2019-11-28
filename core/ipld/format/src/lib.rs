use std::fmt::{Debug, Error, Formatter};

mod error;
mod navipld;

#[cfg(test)]
mod tests;

pub use error::*;

pub trait Resolver {
    /// `resolve` resolves a path through this node, stopping at any link boundary
    /// and returning the object found as well as the remaining path to traverse
    fn resolve(path: &[String]) -> Vec<String>;
    /// `tree` lists all paths within the object under 'path', and up to the given depth.
    /// To list the entire object (similar to `find .`) pass "" and -1
    fn tree(path: &str, depth: i32) -> Vec<String>;
}

/// Node must support deep copy
/// Node is the base interface all IPLD nodes must implement.
///
/// Nodes are **Immutable** and all methods defined on the interface are
pub trait Node: Resolver + block_format::Block + Clone {
    /// `resolve_link` is a helper function that calls resolve and asserts the
    /// output is a link
    fn resolve_link(&self, path: &str, depth: i32) -> Vec<String>;

    /// `links` is a helper function that returns all links within this object
    fn links(&self) -> Vec<&Link>;

    /// `stat` is a helper function that returns `NodeStat` ref
    fn stat(&self) -> Result<&NodeStat>;

    /// `size` returns the size in bytes of the serialized object
    fn size(&self) -> u64;
}

/// Link represents an IPFS Merkle DAG Link between Nodes.
pub struct Link {
    name: String,
    size: u64,
    cid: cid::Cid,
}

/// NodeStat is a statistics object for a Node. Mostly sizes.
pub struct NodeStat {
    hash: multihash::Multihash,
    num_links: usize,
    block_size: usize,
    links_size: usize,
    data_size: usize,
    cumulative_size: usize,
}

impl Debug for NodeStat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "NodeStat{{NumLinks: {:}, BlockSize: {:}, LinksSize: {:}, DataSize: {:}, CumulativeSize: {:}}}",
            self.num_links,
            self.block_size,
            self.links_size,
            self.data_size,
            self.cumulative_size,
        )
    }
}

pub fn make_link<T: Node>(node: T) -> Link {
    let size = node.size();
    Link {
        name: Default::default(),
        size,
        cid: node.cid().to_owned(),
    }
}
