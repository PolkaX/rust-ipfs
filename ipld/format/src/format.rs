// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use block_format::Block;
use cid::Cid;

use crate::error::Result;
use crate::merkledag::NodeGetter;

/// Node must support deep copy
/// Node is the base interface all IPLD nodes must implement.
///
/// Nodes are **Immutable** and all methods defined on the interface are **Thread Safe**.
pub trait Node: Block {
    /// A helper function that calls resolve and asserts the output is a link.
    fn resolve_link(&self, path: &[&str]) -> Result<(Link, Vec<String>)>;

    /// A helper function that returns all links within this object.
    fn links(&self) -> Vec<&Link>;

    /// A helper function that returns `NodeStat` ref.
    fn stat(&self) -> Result<&NodeStat>;

    /// Returns the size in bytes of the serialized object.
    fn size(&self) -> u64;
}

/// Resolver is the interface that operate path.
pub trait Resolver {
    /// The found object by resolving a path through this node.
    type Output;

    /// Resolves a path through this node, stopping at any link boundary
    /// and returning the object found as well as the remaining path to traverse.
    fn resolve(&self, path: &[&str]) -> Result<(Self::Output, Vec<String>)>;

    /// Lists all paths within the object under 'path', and up to the given depth.
    /// To list the entire object (similar to `find .`) pass "" and None.
    fn tree(&self, path: &str, depth: Option<usize>) -> Vec<String>;
}

/// Link represents an IPFS Merkle DAG Link between Nodes.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Link {
    /// It should be unique per object.
    pub name: String,
    /// The cumulative size of target object.
    pub size: u64,
    /// The CID of the target object.
    pub cid: Cid,
}

impl Link {
    /// Create a new `Link` with the given CID.
    pub fn new_with_cid(cid: Cid) -> Link {
        Link {
            name: "".to_string(),
            size: 0,
            cid,
        }
    }

    /// Creates a `Link` with the given node.
    pub fn new_with_node<T: Node>(node: T) -> Link {
        Link {
            name: Default::default(),
            size: node.size(),
            cid: node.cid().clone(),
        }
    }

    /// Returns the MerkleDAG Node that this link points to.
    pub fn node<T: Node>(&self, ng: &impl NodeGetter<T>) -> Result<impl Node> {
        Ok(ng.get(&self.cid))
    }
}

/// NodeStat is a statistics object for a Node. Mostly sizes.
pub struct NodeStat {
    /// The multihash of node.
    pub hash: multihash::Multihash,
    /// The number of links in link table.
    pub num_links: usize,
    /// The size of the raw, encoded data.
    pub block_size: usize,
    /// The size of the links segment.
    pub links_size: usize,
    /// The size of the data segment.
    pub data_size: usize,
    /// The cumulative size of object and its references.
    pub cumulative_size: usize,
}

impl std::fmt::Debug for NodeStat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeStat")
            .field("NumLinks", &self.num_links)
            .field("BlockSize", &self.block_size)
            .field("LinksSize", &self.links_size)
            .field("DataSize", &self.data_size)
            .field("CumulativeSize", &self.cumulative_size)
            .finish()
    }
}
