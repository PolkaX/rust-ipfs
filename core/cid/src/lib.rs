// Copyright 2019-2020 PolkaX.
// This file is part of rust-ipfs.

/// ! Implementation of [cid](https://github.com/ipld/cid) in Rust.
/// Fork from project [rust-cid](https://github.com/multiformats/rust-cid)
/// But we provide more useful functions.

mod to_cid;
mod error;
mod codec;
mod version;

pub use to_cid::ToCid;
pub use version::Version;
pub use codec::Codec;
pub use error::{Error, Result};

use integer_encoding::{VarIntReader, VarIntWriter};
use std::fmt;
use std::io::Cursor;

use thiserror::Error;
use integer_encoding::VarInt;
use varint::VARINT_64_MAX_BYTES;

//#[derive(Error, Debug)]
//pub enum CidError {
//    #[error("Invalid hash bytes for cidv0, code:{}, digest len:{}", .0.name(), .1)]
//    InvalidCidV0(MHashEnum, usize),
//
//    // multi hash
//    #[error("[Multihash]This type is not supported yet")]
//    UnsupportedType,
//
//    #[error("[Multihash]Not matching input length")]
//    BadInputLength,
//
//    #[error("[Multihash]Found unknown code")]
//    UnknownCode,
//}
//
//impl From<MHashError> for CidError {
//    fn from(e: MHashError) -> Self {
//        match e {
//            MHashError::UnsupportedType => CidError::UnsupportedType,
//            MHashError::BadInputLength => CidError::BadInputLength,
//            MHashError::UnknownCode => CidError::UnknownCode,
//        }
//    }
//}
//
//pub fn new_cid_v0(mhash: Vec<u8>) -> Result<Cid, CidError> {
//    let multi_hash: Multihash = decode(&mhash).map_err(CidError::from)?;
//    if multi_hash.alg != MHashEnum::SHA2256 || multi_hash.digest.len() != 32 {
//        return Err(CidError::InvalidCidV0(multi_hash.alg, multi_hash.digest.len()));
//    }
//
//    Ok(Cid::new(
//        Codec::DagProtobuf,
//        Version::V0,
//        &mhash,
//    ))
//}
//
//pub fn new_cid_v1(codec: Codec, mhash: Vec<u8>) -> Result<Cid, CidError> {
//    let hashlen = mhash.len();
//    // two 8 bytes (max) numbers plus hash
//    let mut buf = vec![0_u8; 2 * VARINT_64_MAX_BYTES + hashlen];
//    let mut n = 1_u64.encode_var(&mut buf);
//    let codec_type: u64 = codec.into();
//    n += codec_type.encode_var(&mut buf[n..]);
//
//    buf[n..n + hashlen].copy_from_slice(&mhash);
//
//    Ok(Cid::new(
//        codec,
//        Version::V1,
//        &buf[..n + hashlen],
//    ))
//}


/// Representation of a CID.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Cid {
    pub version: Version,
    pub codec: Codec,
    pub hash: Vec<u8>,
}

/// Prefix represents all metadata of a CID, without the actual content.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Prefix {
    pub version: Version,
    pub codec: Codec,
    pub mh_type: multihash::Hash,
    pub mh_len: usize,
}

impl Cid {
    /// Create a new CID.
    pub fn new(codec: Codec, version: Version, hash: &[u8]) -> Cid {
        Cid {
            version: version,
            codec: codec,
            hash: hash.into(),
        }
    }

    /// Create a new CID from raw data (binary or multibase encoded string)
    pub fn from<T: ToCid>(data: T) -> Result<Cid> {
        data.to_cid()
    }

    /// Create a new CID from a prefix and some data.
    pub fn new_from_prefix(prefix: &Prefix, data: &[u8]) -> Cid {
        let mut hash = multihash::encode(prefix.mh_type, data).unwrap();
        hash.truncate(prefix.mh_len + 2);
        Cid {
            version: prefix.version,
            codec: prefix.codec.to_owned(),
            hash: hash.into(),
        }
    }

    fn to_string_v0(&self) -> String {
        use multibase::{encode, Base};

        let mut string = encode(Base::Base58btc, self.hash.as_slice());

        // Drop the first character as v0 does not know
        // about multibase
        string.remove(0);

        string
    }

    fn to_string_v1(&self) -> String {
        use multibase::{encode, Base};

        encode(Base::Base58btc, self.to_bytes().as_slice())
    }

    pub fn to_string(&self) -> String {
        match self.version {
            Version::V0 => self.to_string_v0(),
            Version::V1 => self.to_string_v1(),
        }
    }

    fn to_bytes_v0(&self) -> Vec<u8> {
        self.hash.clone()
    }

    fn to_bytes_v1(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(16);
        res.write_varint(u64::from(self.version)).unwrap();
        res.write_varint(u64::from(self.codec)).unwrap();
        res.extend_from_slice(&self.hash);

        res
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self.version {
            Version::V0 => self.to_bytes_v0(),
            Version::V1 => self.to_bytes_v1(),
        }
    }

    pub fn prefix(&self) -> Prefix {
        // Unwrap is safe, as this should have been validated on creation
        let mh = multihash::Multihash::from_bytes(self.hash.clone()).unwrap();

        Prefix {
            version: self.version,
            codec: self.codec.to_owned(),
            mh_type: mh.alg,
            mh_len: mh.digest.len(),
        }
    }
}

impl std::hash::Hash for Cid {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_bytes().hash(state);
    }
}

impl fmt::Display for Cid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", Cid::to_string(self))
    }
}

impl Prefix {
    pub fn new_from_bytes(data: &[u8]) -> Result<Prefix> {
        let mut cur = Cursor::new(data);

        let raw_version = cur.read_varint()?;
        let raw_codec = cur.read_varint()?;
        let raw_mh_type: u64 = cur.read_varint()?;

        let version = Version::from(raw_version)?;
        let codec = Codec::from(raw_codec)?;

        let mh_type = multihash::Hash::from_code(raw_mh_type as u16).ok_or(Error::UnknownCodec)?;

        let mh_len = cur.read_varint()?;

        Ok(Prefix {
            version: version,
            codec: codec,
            mh_type: mh_type,
            mh_len: mh_len,
        })
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(4);

        // io can't fail on Vec
        res.write_varint(u64::from(self.version)).unwrap();
        res.write_varint(u64::from(self.codec)).unwrap();
        res.write_varint(self.mh_type.code() as u64).unwrap();
        res.write_varint(self.mh_len as u64).unwrap();

        res
    }
}
