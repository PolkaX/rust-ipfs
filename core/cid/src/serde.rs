// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use serde_cbor::tags::Tagged;

use crate::cid::Cid;
use crate::error::CidError;

const BINARY_MULTIBASE_PREFIX: u8 = 0;
/// CID_CBOR_TAG is the integer used to represent cid tags in CBOR.
pub const CID_CBOR_TAG: u64 = 42;

impl serde::Serialize for Cid {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // add 0 at start, that binary multibase prefix...
        let mut b = vec![BINARY_MULTIBASE_PREFIX];
        b.extend(self.to_bytes());
        // Special process for bytes, due to serde_cbor would treat Vec<u8> as Array u8, not bytes
        let value = serde_bytes::Bytes::new(&b);
        Tagged::new(Some(CID_CBOR_TAG), value).serialize(s)
    }
}

/// util function for deserialize bytes into cid, notice this bytes first byte must be 0,
/// for CIDs must have binary multibase
pub fn deserialize_cid_from_bytes(res: &[u8]) -> Result<Cid, CidError> {
    if res.is_empty() || res[0] != BINARY_MULTIBASE_PREFIX {
        return Err(CidError::InvalidBinaryMultibasePrefix);
    }

    let cid = Cid::from(&res[1..])?;
    Ok(cid)
}

impl<'de> serde::Deserialize<'de> for Cid {
    fn deserialize<D>(deserializer: D) -> Result<Cid, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let tagged = Tagged::<serde_bytes::ByteBuf>::deserialize(deserializer)?;
        match tagged.tag {
            Some(CID_CBOR_TAG) | None => {
                let res = tagged.value.to_vec();
                let cid = deserialize_cid_from_bytes(&res)
                    .map_err(|e| D::Error::custom(format!("Cid deserialize failed: {:}", e)))?;
                Ok(cid)
            }
            Some(_) => Err(D::Error::custom("unexpected tag")),
        }
    }
}
