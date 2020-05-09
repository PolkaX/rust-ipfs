// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use bytes::Bytes;

use cid::Cid;
use multihash::{MultihashRef, Sha2_256};

use crate::error::Result;

/// The trait for getting raw data and cid of block.
pub trait Block: AsRef<Cid> {
    /// Get the raw data of block.
    fn raw_data(&self) -> &Bytes;

    /// Get the cid.
    fn cid(&self) -> &Cid {
        self.as_ref()
    }
}

impl Block for BasicBlock {
    fn raw_data(&self) -> &Bytes {
        &self.data
    }
}

impl AsRef<Cid> for BasicBlock {
    fn as_ref(&self) -> &Cid {
        &self.cid
    }
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
        let sha256_hash = Sha2_256::digest(data.as_ref());
        BasicBlock {
            data,
            cid: Cid::new_v0(sha256_hash).expect("invalid hash for CIDv0"),
        }
    }

    /// Creates a new `BasicBlock` with given bytes and CID.
    pub fn new_with_cid(data: Bytes, cid: Cid) -> Result<BasicBlock> {
        Ok(BasicBlock { data, cid })
    }

    /// Get the multihash of cid of the basic block.
    pub fn multihash(&self) -> MultihashRef {
        self.cid.hash()
    }
}

impl std::fmt::Display for BasicBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Block {:?}]", self)
    }
}
