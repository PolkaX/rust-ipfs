// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use cid::Cid;
use serde::{
    de::{Deserialize, Deserializer, Error},
    ser::{Serialize, Serializer},
};
use serde_cbor::tags::Tagged;

use crate::error::IpldCborError;

/// CID_CBOR_TAG is the integer used to represent cid tags in CBOR.
pub(crate) const CID_CBOR_TAG: u64 = 42;

/// A CID Wrapper that implements CBOR serialization/deserialization.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct CborCid(pub(crate) Cid);

impl CborCid {
    /// Convert to inner cid.
    pub fn into_inner(self) -> Cid {
        self.0
    }
}

impl Serialize for CborCid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // add 0 at start
        let mut bytes = vec![0_u8];
        bytes.extend(self.0.to_bytes());
        // Special process for bytes, due to serde_cbor would treat Vec<u8> as Array u8, not bytes
        let value = serde_bytes::Bytes::new(&bytes);
        Tagged::new(Some(CID_CBOR_TAG), value).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CborCid {
    fn deserialize<D>(deserializer: D) -> Result<CborCid, D::Error>
    where
        D: Deserializer<'de>,
    {
        let tagged = Tagged::<serde_bytes::ByteBuf>::deserialize(deserializer)?;
        match tagged.tag {
            Some(CID_CBOR_TAG) | None => {
                let cid = deserialize_cid_from_bytes(&tagged.value.to_vec())
                    .map_err(|e| D::Error::custom(format!("Cid deserialize failed: {:}", e)))?;
                Ok(CborCid(cid))
            }
            Some(_) => Err(D::Error::custom("unexpected tag")),
        }
    }
}

pub(crate) fn deserialize_cid_from_bytes(bytes: &[u8]) -> Result<Cid, IpldCborError> {
    if bytes.is_empty() {
        return Err(IpldCborError::DeserializeCid("Value was empty".to_string()));
    }

    if bytes[0] != 0 {
        return Err(IpldCborError::DeserializeCid(
            "Invalid multibase on IPLD link".to_string(),
        ));
    }

    let cid = Cid::from(&bytes[1..])?;
    Ok(cid)
}

impl From<Cid> for CborCid {
    fn from(cid: Cid) -> Self {
        CborCid(cid)
    }
}

impl std::borrow::Borrow<Cid> for CborCid {
    fn borrow(&self) -> &Cid {
        &self.0
    }
}

impl std::borrow::BorrowMut<Cid> for CborCid {
    fn borrow_mut(&mut self) -> &mut Cid {
        &mut self.0
    }
}

impl AsRef<Cid> for CborCid {
    fn as_ref(&self) -> &Cid {
        &self.0
    }
}

impl AsMut<Cid> for CborCid {
    fn as_mut(&mut self) -> &mut Cid {
        &mut self.0
    }
}

impl std::ops::Deref for CborCid {
    type Target = Cid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for CborCid {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
