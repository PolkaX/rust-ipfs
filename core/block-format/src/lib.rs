use multihash;
use thiserror::Error;

use primitives::Bytes;
use rust_cid::Cid;

#[derive(Error, Debug)]
pub enum BlockFormatError {
    #[error("data did not match given hash, fst: {fst:?}, snd: {snd:?}")]
    WrongHash(fst, snd),
}

pub struct BasicBlock {
    cid: cid::Cid,
    data: Bytes,
}

impl BasicBlock {
    pub fn new(data: Bytes) -> BasicBlock {
        BasicBlock { data, cid: }
    }
}