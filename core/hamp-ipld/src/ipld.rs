use block_format::Block as BlockT;
use cid::Cid;

use crate::error::*;

pub trait Blocks {
    fn get_block(&self) -> Result<Box<dyn BlockT>>;
    fn add_block(&self, block: impl BlockT) -> Result<()>;
}

#[derive(Debug)]
pub struct CborIpldStor<B: Blocks> {
    blocks: B,
}

pub trait BlockStore {
    fn get(&self, cid: &Cid) -> Result<Box<dyn BlockT>>;
    fn put(&mut self, block: impl BlockT) -> Result<()>;
}

pub struct BsWrapper<BS: BlockStore> {
    bs: BS,
}

impl<BS: BlockStore> BsWrapper<BS> {
    pub fn get_block(&self, cid: &Cid) -> Result<Box<dyn BlockT>> {
        self.bs.get(cid)
    }

    pub fn add_block(&mut self, blk: impl BlockT) -> Result<()> {
        self.bs.put(blk)
    }
}
