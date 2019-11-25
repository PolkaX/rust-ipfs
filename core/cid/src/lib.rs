// Copyright 2019-2020 PolkaX.
// This file is part of rust-ipfs.

mod codec;
mod error;
/// ! Implementation of [cid](https://github.com/ipld/cid) in Rust.
/// Fork from project [rust-cid](https://github.com/multiformats/rust-cid)
/// But we provide more useful functions.
mod to_cid;
mod version;

use integer_encoding::{VarInt, VarIntReader, VarIntWriter};
use std::fmt;
use std::io::Cursor;
use thiserror::Error;
use varint::VARINT_64_MAX_BYTES;

use multihash::{Hash as MHashEnum, Multihash};

pub use codec::Codec;
pub use error::{Error, Result};
pub use to_cid::ToCid;
pub use version::Version;

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
    pub mh_type: MHashEnum,
    pub mh_len: usize,
}

impl Cid {
    /// Create a new CID.
    pub fn new(codec: Codec, version: Version, hash: &[u8]) -> Cid {
        Cid {
            version,
            codec,
            hash: hash.into(),
        }
    }

    /// Create a new CID from raw data (binary or multibase encoded string)
    pub fn from<T: ToCid>(data: T) -> Result<Cid> {
        data.to_cid()
    }

    /// Create a new CID from a prefix and some data.
    pub fn new_from_prefix(prefix: &Prefix, data: &[u8]) -> Cid {
        let hash = multihash::encode(prefix.mh_type, data).unwrap();
        let mut hash_vec = hash.into_bytes();
        hash_vec.truncate(prefix.mh_len + 2);
        Cid {
            version: prefix.version,
            codec: prefix.codec.to_owned(),
            hash: hash_vec,
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
        let mh = Multihash::from_bytes(self.hash.clone()).unwrap();

        Prefix {
            version: self.version,
            codec: self.codec.to_owned(),
            mh_type: mh.algorithm(),
            mh_len: mh.digest().len(),
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

        let mh_type = MHashEnum::from_code(raw_mh_type as u16).ok_or(Error::UnknownCodec)?;

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

pub fn new_cid_v0(mhash: Vec<u8>) -> Result<Cid> {
    let multi_hash: Multihash = Multihash::from_bytes(mhash)?;
    if multi_hash.algorithm() != MHashEnum::SHA2256 || multi_hash.digest().len() != 32 {
        return Err(Error::InvalidCidV0(
            multi_hash.algorithm(),
            multi_hash.digest().len(),
        ));
    }

    Ok(Cid::new(
        Codec::DagProtobuf,
        Version::V0,
        multi_hash.as_bytes(),
    ))
}

pub fn new_cid_v1(codec: Codec, mhash: Vec<u8>) -> Result<Cid> {
    let hashlen = mhash.len();
    // two 8 bytes (max) numbers plus hash
    let mut buf = vec![0_u8; 2 * VARINT_64_MAX_BYTES + hashlen];
    let mut n = 1_u64.encode_var(&mut buf);
    let codec_type: u64 = codec.into();
    n += codec_type.encode_var(&mut buf[n..]);

    buf[n..n + hashlen].copy_from_slice(&mhash);

    Ok(Cid::new(codec, Version::V1, &buf[..n + hashlen]))
}
