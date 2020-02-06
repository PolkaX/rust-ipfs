// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

mod error;

use block_format::BasicBlock;
use cid::Cid;

pub use crate::error::*;

/// Blockstore wraps a Datastore block-centered methods and provides a layer
/// of abstraction which allows to add different caching strategies.
// TODO need to decide BasicBlock or BlockT for params
pub trait Blockstore {
    fn delete_block(&self, cid: &Cid) -> Result<()>;
    fn has(&self, cid: &Cid) -> Result<()>;
    fn get(&self, cid: &Cid) -> Result<BasicBlock>;

    /// GetSize returns the CIDs mapped BlockSize
    fn get_size(&self, cid: &Cid) -> Result<usize>;

    fn put(&mut self, block: BasicBlock) -> Result<()>;
    fn put_many(&mut self, block: &[BasicBlock]) -> Result<()>;
    fn hash_on_read(&mut self, enable: bool);
}
