use std::fmt::{Debug, Error, Formatter};

pub mod coding;
mod daghelpers;
mod error;
mod merkledag;
mod navipld;
mod walker;

#[cfg(test)]
mod tests;

pub use error::*;

pub trait Resolver {
    type Output;
    /// `resolve` resolves a path through this node, stopping at any link boundary
    /// and returning the object found as well as the remaining path to traverse
    fn resolve(&self, path: &[String]) -> Result<(Self::Output, Vec<String>)>;
    /// `tree` lists all paths within the object under 'path', and up to the given depth.
    /// To list the entire object (similar to `find .`) pass "" and None
    fn tree(&self, path: &str, depth: Option<usize>) -> Vec<String>;
}

/// Node must support deep copy
/// Node is the base interface all IPLD nodes must implement.
///
/// Nodes are **Immutable** and all methods defined on the interface are
pub trait Node: block_format::Block {
    /// `resolve_link` is a helper function that calls resolve and asserts the
    /// output is a link
    fn resolve_link(&self, path: &[String]) -> Result<(Link, Vec<String>)>;

    /// `links` is a helper function that returns all links within this object
    fn links(&self) -> Vec<&Link>;

    /// `stat` is a helper function that returns `NodeStat` ref
    fn stat(&self) -> Result<&NodeStat>;

    /// `size` returns the size in bytes of the serialized object
    fn size(&self) -> u64;
}

/// Link represents an IPFS Merkle DAG Link between Nodes.
pub struct Link {
    pub name: String,
    pub size: u64,
    pub cid: cid::Cid,
}

impl Link {
    pub fn new_default(cid: cid::Cid) -> Link {
        Link {
            name: "".to_string(),
            size: 0,
            cid,
        }
    }
}

/// NodeStat is a statistics object for a Node. Mostly sizes.
pub struct NodeStat {
    pub hash: multihash::Multihash,
    /// number of links in link table
    pub num_links: usize,
    /// size of the raw, encoded data
    pub block_size: usize,
    /// size of the links segment
    pub links_size: usize,
    /// size of the data segment
    pub data_size: usize,
    /// cumulative size of object and its references
    pub cumulative_size: usize,
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
