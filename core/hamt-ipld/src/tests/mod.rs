// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

mod cbor_test;
mod hamt_test;
mod hash_test;
mod ipld_test;

use std::collections::HashMap;

use block_format::{BasicBlock, Block as BlockT};
use bytes::Bytes;
use cid::Cid;

use super::*;
use crate::error::*;
use crate::node::trait_impl::PartNode;

#[derive(Debug)]
pub struct MockBlocks {
    data: HashMap<Cid, Vec<u8>>,
}

impl MockBlocks {
    pub fn new() -> Self {
        MockBlocks {
            data: Default::default(),
        }
    }
}

impl Blocks for MockBlocks {
    fn get_block(&self, cid: &Cid) -> Result<Box<dyn BlockT>> {
        let blk = self
            .data
            .get(cid)
            .map(|data| BasicBlock::new(Bytes::copy_from_slice(data)))
            .ok_or_else(|| Error::NotFoundForCid(cid.clone()))?;
        Ok(Box::new(blk))
    }

    fn add_block(&mut self, block: impl BlockT) -> Result<()> {
        let cid = block.cid().clone();
        self.data.insert(cid, block.raw_data().to_vec());
        Ok(())
    }
}

pub fn new_cbor_store() -> CborIpldStor<MockBlocks> {
    CborIpldStor::new(MockBlocks::new())
}
