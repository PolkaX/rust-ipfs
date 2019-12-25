// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::io::Cursor;

use integer_encoding::VarIntReader;
use multihash::Multihash;

use crate::cid::Cid;
use crate::codec::Codec;
use crate::error::{CidError, Result};
use crate::version::Version;

/// A trait for converting data into CID format.
pub trait ToCid {
    /// The only method for converting data into CID format in the trait.
    fn to_cid(&self) -> Result<Cid>;
}

impl ToCid for Vec<u8> {
    /// Create a Cid from a byte vector.
    #[inline]
    fn to_cid(&self) -> Result<Cid> {
        self.as_slice().to_cid()
    }
}

impl<'a> ToCid for &'a [u8] {
    #[inline]
    fn to_cid(&self) -> Result<Cid> {
        ToCid::to_cid(*self)
    }
}

impl ToCid for [u8] {
    /// Create a Cid from a byte slice.
    fn to_cid(&self) -> Result<Cid> {
        if Version::is_v0_binary(self) {
            // Verify that hash can be decoded, this is very cheap
            let hash = Multihash::from_bytes(self.to_vec())?;
            Ok(Cid::new(Version::V0, Codec::DagProtobuf, hash))
        } else {
            let mut cur = Cursor::new(self);
            let raw_version = cur.read_varint::<u8>()?;
            let raw_codec = cur.read_varint::<u16>()?;
            let hash = &self[cur.position() as usize..];

            let version = Version::from(raw_version)?;
            let codec = Codec::from(raw_codec)?;
            // Verify that hash can be decoded, this is very cheap
            let hash = Multihash::from_bytes(hash.to_vec())?;

            Ok(Cid::new(version, codec, hash))
        }
    }
}

impl ToCid for String {
    /// Create a Cid from an owned String.
    #[inline]
    fn to_cid(&self) -> Result<Cid> {
        self.as_str().to_cid()
    }
}

impl<'a> ToCid for &'a str {
    #[inline]
    fn to_cid(&self) -> Result<Cid> {
        ToCid::to_cid(*self)
    }
}

impl ToCid for str {
    fn to_cid(&self) -> Result<Cid> {
        static IPFS_DELIMETER: &str = "/ipfs/";

        let hash = match self.find(IPFS_DELIMETER) {
            Some(index) => &self[index + IPFS_DELIMETER.len()..],
            _ => self,
        };

        if hash.len() < 2 {
            return Err(CidError::InputTooShort);
        }

        if Version::is_v0_str(hash) {
            multibase::decode_base58btc(hash)?.to_cid()
        } else {
            multibase::decode(hash)?.1.to_cid()
        }
    }
}

impl std::str::FromStr for Cid {
    type Err = CidError;

    fn from_str(src: &str) -> Result<Self> {
        src.to_cid()
    }
}
