// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use integer_encoding::VarIntWriter;
use multibase::Base;
use multihash::{Hash, Multihash};

use crate::codec::Codec;
use crate::error::{Error, Result};
use crate::prefix::Prefix;
use crate::to_cid::ToCid;
use crate::version::Version;

/// Representation of a CID.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Cid {
    /// The version of CID.
    version: Version,
    /// The codec of CID.
    codec: Codec,
    /// The hash of CID.
    hash: Multihash,
}

impl Cid {
    /// Create a new CID.
    pub fn new(version: Version, codec: Codec, hash: Multihash) -> Cid {
        Cid {
            version,
            codec,
            hash,
        }
    }

    /// A helper function to create CIDv0.
    pub fn new_cid_v0(mh: Multihash) -> Result<Cid> {
        if mh.algorithm() != Hash::SHA2256 || mh.digest().len() != 32 {
            return Err(Error::InvalidCidV0(mh.algorithm(), mh.digest().len()));
        }

        Ok(Cid::new(Version::V0, Codec::DagProtobuf, mh))
    }

    /// A helper function to create CIDv1.
    pub fn new_cid_v1(codec: Codec, mh: Multihash) -> Result<Cid> {
        Ok(Cid::new(Version::V1, codec, mh))
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
        multibase::encode_base58btc(self.hash.as_bytes())
    }

    fn to_string_v1(&self, base: Base) -> String {
        multibase::encode(base, self.to_bytes().as_slice())
    }

    /// Stringify the CID.
    pub fn to_string_by_base(&self, v1_base: Base) -> String {
        match self.version {
            Version::V0 => self.to_string_v0(),
            Version::V1 => self.to_string_v1(v1_base),
        }
    }

    /// Stringify the CID with Base32 for `Version::V1`
    pub fn to_string(&self) -> String {
        self.to_string_by_base(Base::Base32Lower)
    }

    fn to_bytes_v0(&self) -> Vec<u8> {
        self.hash.as_bytes().to_vec()
    }

    fn to_bytes_v1(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(16);
        res.write_varint(u8::from(self.version)).unwrap();
        res.write_varint(self.codec.code()).unwrap();
        res.extend_from_slice(self.hash.as_bytes());
        res
    }

    /// Convert CID to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        match self.version {
            Version::V0 => self.to_bytes_v0(),
            Version::V1 => self.to_bytes_v1(),
        }
    }

    /// Return the prefix of the CID.
    pub fn prefix(&self) -> Prefix {
        // Unwrap is safe, as this should have been validated on creation
        let mh = self.multihash();

        Prefix {
            version: self.version,
            codec: self.codec,
            mh_type: mh.algorithm(),
            mh_len: mh.digest().len(),
        }
    }

    /// Return the version of CID.
    pub fn version(&self) -> Version {
        self.version
    }

    /// Return the codec of CID.
    pub fn codec(&self) -> Codec {
        self.codec
    }

    /// Return the multihash of CID.
    pub fn multihash(&self) -> Multihash {
        self.hash.clone()
    }
}

impl std::hash::Hash for Cid {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_bytes().hash(state);
    }
}

impl std::fmt::Display for Cid {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.to_string_by_base(multibase::Base::Base32Lower)
        )
    }
}
