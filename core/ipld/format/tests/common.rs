use std::sync::Arc;

use block_format::Block;
use bytes::Bytes;
use cid::{AsCidRef, Cid, Codec, Hash, Prefix, Version};

use rust_ipld_format::{FormatError, Link, NavigableNode, Node, NodeStat, Resolver, Result};

pub struct EmptyNode {
    cid: Cid,
    data: Bytes,
}

impl Default for EmptyNode {
    fn default() -> Self {
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

impl EmptyNode {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Node for EmptyNode {
    fn resolve_link(&self, _path: &[&str]) -> Result<(Link, Vec<String>)> {
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
}

impl AsCidRef for EmptyNode {
    fn cid(&self) -> &Cid {
        &self.cid
    }
}

impl Resolver for EmptyNode {
    type Output = ();
    fn resolve(&self, _path: &[&str]) -> Result<(Self::Output, Vec<String>)> {
        unimplemented!()
    }

    fn tree(&self, _path: &str, _depth: Option<usize>) -> Vec<String> {
        unimplemented!()
    }
}

pub struct N {
    pub inner: EmptyNode,
    pub child: Vec<Arc<dyn NavigableNode>>,
}

impl NavigableNode for N {
    fn child_total(&self) -> usize {
        self.child.len()
    }

    fn fetch_child(&self, child_index: usize) -> Result<Arc<dyn NavigableNode>> {
        self.child
            .get(child_index)
            .cloned()
            .ok_or(FormatError::NoChild(child_index))
    }
}
