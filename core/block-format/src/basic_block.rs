// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::fmt::{Debug, Display, Error, Formatter};

use bytes::Bytes;
use ipfs_util::hash;
use rust_cid::{new_cid_v0, Cid, Multihash};

use crate::error::BlockFormatError;

pub trait Block {
    fn raw_data(&self) -> &Bytes;
    fn cid(&self) -> &Cid;
}

pub struct BasicBlock {
    cid: Cid,
    data: Bytes,
}

impl BasicBlock {
    pub fn new(data: Bytes) -> BasicBlock {
        let sha256_hash = hash(data.as_ref());
        BasicBlock {
            data,
            cid: new_cid_v0(sha256_hash).expect("invalid hash for cidv0"),
        }
    }
    pub fn new_with_cid(data: Bytes, cid: Cid) -> Result<BasicBlock, BlockFormatError> {
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

impl Debug for BasicBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "[Block {:?}]", self)
    }
}

impl Display for BasicBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        Debug::fmt(self, f)
    }
}
