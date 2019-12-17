mod test_coding;
mod test_dag_walker;

use block_format::{BasicBlock, Block};
use bytes::Bytes;
use cid::{Cid, Codec, Hash, Prefix, Version};

use crate::error::*;
use crate::walker::NavigableNode;
use crate::{FormatError, Link, Node, NodeStat, Resolver};
use std::sync::Arc;

struct EmptyNode {
    cid: Cid,
    data: Bytes,
}

impl EmptyNode {
    fn new() -> Self {
        let p = Prefix {
            version: Version::V1,
            codec: Codec::Raw,
            mh_type: Hash::Identity,
            mh_len: 0,
        };
        EmptyNode {
            cid: p.sum(b"").unwrap(),
            data: Bytes::from_static(b""),
        }
    }
}

impl Node for EmptyNode {
    fn resolve_link(&self, path: &[String]) -> Result<(Link, Vec<String>)> {
        unimplemented!()
    }

    fn links(&self) -> Vec<&Link> {
        unimplemented!()
    }

    fn stat(&self) -> Result<&NodeStat> {
        unimplemented!()
    }

    fn size(&self) -> u64 {
        unimplemented!()
    }
}

impl Block for EmptyNode {
    fn raw_data(&self) -> &Bytes {
        &self.data
    }

    fn cid(&self) -> &Cid {
        &self.cid
    }
}

impl Resolver for EmptyNode {
    type Output = ();
    fn resolve(&self, path: &[String]) -> Result<(Self::Output, Vec<String>)> {
        unimplemented!()
    }

    fn tree(&self, path: &str, depth: Option<usize>) -> Vec<String> {
        unimplemented!()
    }
}

struct N {
    inner: EmptyNode,
    child: Vec<Arc<dyn NavigableNode>>,
}

impl NavigableNode for N {
    fn child_total(&self) -> usize {
        self.child.len()
    }

    fn fetch_child(&self, child_index: usize) -> Result<Arc<dyn NavigableNode>> {
        self.child
            .get(child_index)
            .map(|d| d.clone())
            .ok_or(FormatError::NoChild(child_index))
    }
}
