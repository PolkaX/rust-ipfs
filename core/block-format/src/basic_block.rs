// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use bytes::Bytes;
use cid::{Cid, Multihash};
use util::sha2_256_hash;

use crate::error::{BlockFormatError, Result};

/// The trait for getting raw data and cid of block.
pub trait Block {
    /// Get the raw data of block.
    fn raw_data(&self) -> &Bytes;

    /// Get the cid of block.
    fn cid(&self) -> &Cid;
}

/// The basic block.
#[derive(Clone, Debug)]
pub struct BasicBlock {
    cid: Cid,
    data: Bytes,
}

impl BasicBlock {
    /// Creates a new `BasicBlock` with given bytes, and its CID is version 0.
    pub fn new(data: Bytes) -> BasicBlock {
        let sha256_hash = sha2_256_hash(data.as_ref());
        BasicBlock {
            data,
            cid: Cid::new_cid_v0(sha256_hash).expect("invalid hash for CIDv0"),
        }
    }

    /// Creates a new `BasicBlock` with given bytes and CID.
    pub fn new_with_cid(data: Bytes, cid: Cid) -> Result<BasicBlock> {
        #[cfg(debug_assertions)]
        {
            let checked_cid = cid
                .prefix()
                .sum(data.as_ref())
                .map_err(BlockFormatError::CidError)?;
            if checked_cid != cid {
                return Err(BlockFormatError::WrongHash(checked_cid, cid));
            }
        }

        Ok(BasicBlock { data, cid })
    }

    /// Get the multihash of cid of the basic block.
    pub fn multihash(&self) -> Multihash {
        self.cid.multihash()
    }
}

impl Block for BasicBlock {
    fn raw_data(&self) -> &Bytes {
        &self.data
    }

    fn cid(&self) -> &Cid {
        &self.cid
    }
}

impl std::fmt::Display for BasicBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Block {:?}]", self)
    }
}
