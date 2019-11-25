mod error;

use multihash;

use primitives::Bytes;
use rust_cid::{new_cid_v0, Cid};

pub use error::BlockFormatError;
use ipfs_util::hash;

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
}
