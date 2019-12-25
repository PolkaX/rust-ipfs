// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::io::Cursor;

use integer_encoding::{VarIntReader, VarIntWriter};
use multihash::Hash;

use crate::cid::Cid;
use crate::codec::Codec;
use crate::error::{CidError, Result};
use crate::version::Version;

/// Prefix represents all metadata of a CID, without the actual content.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Prefix {
    /// The version of CID.
    pub version: Version,
    /// The codec of CID.
    pub codec: Codec,
    /// The hash type of CID.
    pub mh_type: Hash,
    /// The hash length of CID.
    pub mh_len: usize,
}

impl Prefix {
    /// Create a new prefix from bytes.
    pub fn new_from_bytes(data: &[u8]) -> Result<Prefix> {
        let mut cur = Cursor::new(data);

        let raw_version = cur.read_varint::<u8>()?;
        let raw_codec = cur.read_varint::<u16>()?;
        let raw_mh_type = cur.read_varint::<u16>()?;
        let mh_len = cur.read_varint()?;

        let version = Version::from(raw_version)?;
        let codec = Codec::from(raw_codec)?;
        let mh_type = Hash::from_code(raw_mh_type).ok_or(CidError::UnknownHash(raw_mh_type))?;

        Ok(Prefix {
            version,
            codec,
            mh_type,
            mh_len,
        })
    }

    /// Convert the prefix to bytes.
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(4);
        // io can't fail on Vec
        res.write_varint(u8::from(self.version)).unwrap();
        res.write_varint(self.codec.code()).unwrap();
        res.write_varint(self.mh_type.code()).unwrap();
        res.write_varint(self.mh_len).unwrap();
        res
    }

    /// Sum uses the information in a prefix to perform a multihash.Sum()
    /// and return a newly constructed CID with the resulting multihash.
    pub fn sum(&self, data: &[u8]) -> Result<Cid> {
        if self.version == Version::V0 && (self.mh_type != Hash::SHA2256 || self.mh_len != 32) {
            return Err(CidError::InvalidV0Prefix);
        }

        let mh = multihash::encode(self.mh_type, data)?;
        match self.version {
            Version::V0 => Cid::new_cid_v0(mh),
            Version::V1 => Cid::new_cid_v1(self.codec, mh),
        }
    }
}

/// A helper function to create the prefix of CIDv0.
pub fn new_prefix_v0(hash: Hash) -> Prefix {
    Prefix {
        version: Version::V0,
        codec: Codec::DagProtobuf,
        mh_type: hash,
        mh_len: hash.size() as usize,
    }
}

/// A helper function to create the prefix of CIDv1.
pub fn new_prefix_v1(codec: Codec, hash: Hash) -> Prefix {
    Prefix {
        version: Version::V1,
        codec,
        mh_type: hash,
        mh_len: hash.size() as usize,
    }
}
