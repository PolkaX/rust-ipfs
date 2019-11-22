// Copyright 2019-2020 PolkaX.
// This file is part of rust-ipfs.

//! Wrapper for https://github.com/multiformats/rust-cid. Maybe use own implementation to replace it in the future

use thiserror::Error;
use integer_encoding::VarInt;
use varint::VARINT_64_MAX_BYTES;

use multihash::{Multihash, Error as MHashError, decode, Hash as MHashEnum};
use cid::{Error, Version};
pub use cid::{Codec, Cid}; //re-export

#[derive(Error, Debug)]
pub enum CidError {
    #[error("Invalid hash bytes for cidv0, code:{}, digest len:{}", .0.name(), .1)]
    InvalidCidV0(MHashEnum, usize),

    // multi hash
    #[error("[Multihash]This type is not supported yet")]
    UnsupportedType,

    #[error("[Multihash]Not matching input length")]
    BadInputLength,

    #[error("[Multihash]Found unknown code")]
    UnknownCode,
}

impl From<MHashError> for CidError {
    fn from(e: MHashError) -> Self {
        match e {
            MHashError::UnsupportedType => CidError::UnsupportedType,
            MHashError::BadInputLength => CidError::BadInputLength,
            MHashError::UnknownCode => CidError::UnknownCode,
        }
    }
}

pub fn new_cid_v0(mhash: Vec<u8>) -> Result<Cid, CidError> {
    let multi_hash: Multihash = decode(&mhash).map_err(CidError::from)?;
    if multi_hash.alg != MHashEnum::SHA2256 || multi_hash.digest.len() != 32 {
        return Err(CidError::InvalidCidV0(multi_hash.alg, multi_hash.digest.len()));
    }

    Ok(Cid::new(
        Codec::DagProtobuf,
        Version::V0,
        &mhash,
    ))
}

pub fn new_cid_v1(codec: Codec, mhash: Vec<u8>) -> Result<Cid, CidError> {
    let hashlen = mhash.len();
    // two 8 bytes (max) numbers plus hash
    let mut buf = vec![0_u8; 2 * VARINT_64_MAX_BYTES + hashlen];
    let mut n = 1_u64.encode_var(&mut buf);
    let codec_type: u64 = codec.into();
    n += codec_type.encode_var(&mut buf[n..]);

    buf[n..n + hashlen].copy_from_slice(&mhash);

    Ok(Cid::new(
        codec,
        Version::V1,
        &buf[..n + hashlen],
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
