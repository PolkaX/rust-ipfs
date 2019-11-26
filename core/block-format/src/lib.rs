mod error;
#[cfg(test)]
mod tests;

use multihash;

use primitives::Bytes;
use rust_cid::{new_cid_v0, Cid, Multihash};

pub use error::BlockFormatError;
use ipfs_util::hash;
use std::fmt::{Debug, Formatter, Error, Display};

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

    pub fn raw_data(&self) -> &Bytes {
        &self.data
    }

    pub fn cid(&self) -> &Cid {
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
