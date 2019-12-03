mod test_coding;

use block_format::{BasicBlock, Block};
use bytes::Bytes;
use cid::{Cid, Codec, MHashEnum, Prefix, Version};

use crate::error::*;
use crate::{FormatError, Link, Node, NodeStat, Resolver};

struct EmptyNode {
    cid: Cid,
    data: Bytes,
}

impl EmptyNode {
    fn new() -> Self {
        let p = Prefix {
            version: Version::V1,
            codec: Codec::Raw,
            mh_type: MHashEnum::Identity,
            mh_len: 0,
        };
        EmptyNode {
            cid: p.sum(b"").unwrap(),
            data: Bytes::from_static(b""),
        }
    }
}

impl Node for EmptyNode {
    fn resolve_link(&self, path: &str, depth: i32) -> Vec<String> {
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
    fn resolve(&self, path: &[String]) -> Vec<String> {
        unimplemented!()
    }

    fn tree(&self, path: &str, depth: i32) -> Vec<String> {
        unimplemented!()
    }
}
