// Copyright 2019-2020 PolkaX.
// This file is part of rust-ipfs.

//! Implementation of [cid](https://github.com/ipld/cid) in Rust.
//! Fork from project [rust-cid](https://github.com/multiformats/rust-cid)
//! But we provide more useful functions.

mod codec;
mod error;
mod to_cid;
mod version;

use std::fmt;
use std::io::Cursor;

use integer_encoding::{VarIntReader, VarIntWriter};
use multibase::Base;
pub use multihash::{Hash as MHashEnum, Multihash};

pub use self::codec::Codec;
pub use self::error::{Error, Result};
pub use self::to_cid::ToCid;
pub use self::version::Version;

/// Representation of a CID.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Cid {
    version: Version,
    codec: Codec,
    hash: Multihash,
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
    pub fn new(codec: Codec, version: Version, hash: Multihash) -> Cid {
        Cid {
            version,
            codec,
            hash,
        }
    }

    pub fn new_cid_v0(mhash: Multihash) -> Result<Cid> {
        if mhash.algorithm() != MHashEnum::SHA2256 || mhash.digest().len() != 32 {
            return Err(Error::InvalidCidV0(mhash.algorithm(), mhash.digest().len()));
        }

        Ok(Cid::new(Codec::DagProtobuf, Version::V0, mhash))
    }

    pub fn new_cid_v1(codec: Codec, mhash: Multihash) -> Result<Cid> {
        Ok(Cid::new(codec, Version::V1, mhash))
    }

    /// Create a new CID from raw data (binary or multibase encoded string)
    pub fn from<T: ToCid>(data: T) -> Result<Cid> {
        data.to_cid()
    }

    /// Create a new CID from a prefix and some data.
    pub fn new_from_prefix(prefix: &Prefix, data: &[u8]) -> Result<Cid> {
        let hash = multihash::encode(prefix.mh_type, data)?;
        Ok(Cid {
            version: prefix.version,
            codec: prefix.codec.to_owned(),
            hash,
        })
    }

    fn to_string_v0(&self) -> String {
        let mut string = multibase::encode(Base::Base58Btc, self.hash.as_bytes());

        // Drop the first character as v0 does not know about multibase
        string.remove(0);

        string
    }

    fn to_string_v1(&self, base: Base) -> String {
        multibase::encode(base, self.to_bytes().as_slice())
    }

    pub fn to_string(&self) -> String {
        self.to_string_by_base(multibase::Base::Base32Lower)
    }

    pub fn to_string_by_base(&self, v1_base: Base) -> String {
        match self.version {
            Version::V0 => self.to_string_v0(),
            Version::V1 => self.to_string_v1(v1_base),
        }
    }

    fn to_bytes_v0(&self) -> Vec<u8> {
        self.hash.as_bytes().to_vec()
    }

    fn to_bytes_v1(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(16);
        res.write_varint(u64::from(self.version)).unwrap();
        res.write_varint(u64::from(self.codec)).unwrap();
        res.extend_from_slice(self.hash.as_bytes());

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
        let mh = self.multihash();

        Prefix {
            version: self.version,
            codec: self.codec.to_owned(),
            mh_type: mh.algorithm(),
            mh_len: mh.digest().len(),
        }
    }

    pub fn multihash(&self) -> Multihash {
        self.hash.clone()
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn codec(&self) -> Codec {
        self.codec
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
            version,
            codec,
            mh_type,
            mh_len,
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

    /// Sum uses the information in a prefix to perform a multihash.Sum()
    /// and return a newly constructed Cid with the resulting multihash.
    pub fn sum(&self, data: &[u8]) -> Result<Cid> {
        if self.version == Version::V0 && (self.mh_type != MHashEnum::SHA2256 || self.mh_len != 32)
        {
            return Err(Error::InvalidV0Prefix);
        }

        let mhash = multihash::encode(self.mh_type, data)?;
        match self.version {
            Version::V0 => Cid::new_cid_v0(mhash),
            Version::V1 => Cid::new_cid_v1(self.codec, mhash),
        }
    }
}
