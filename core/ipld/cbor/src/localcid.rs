// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::borrow::Borrow;
use std::ops::{Deref, DerefMut};

use serde::de::{Deserialize, Deserializer, Error};
use serde::ser::{Serialize, Serializer};
use serde_cbor::tags::Tagged;

use crate::error::IpldCborError;
use crate::Cid;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct CborCid(pub Cid);

/// CID_CBOR_TAG is the integer used to represent cid tags in CBOR.
pub const CID_CBOR_TAG: u64 = 42;

impl Serialize for CborCid {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // add 0 at start
        let mut b = vec![0_u8];
        b.extend(self.0.to_bytes());
        // Special process for bytes, due to serde_cbor would treat Vec<u8> as Array u8, not bytes
        let value = serde_bytes::Bytes::new(&b);
        Tagged::new(Some(CID_CBOR_TAG), value).serialize(s)
    }
}

pub fn deserialize_cid_from_bytes(res: &[u8]) -> Result<Cid, IpldCborError> {
    if res.len() == 0 {
        return Err(IpldCborError::DeserializeCid(format!("Value was empty")));
    }

    if res[0] != 0 {
        return Err(IpldCborError::DeserializeCid(format!(
            "Invalid multibase on IPLD link"
        )));
    }

    let cid = Cid::from(&res[1..])?;
    Ok(cid)
}

impl<'de> Deserialize<'de> for CborCid {
    fn deserialize<D>(deserializer: D) -> Result<CborCid, D::Error>
    where
        D: Deserializer<'de>,
    {
        let tagged = Tagged::<serde_bytes::ByteBuf>::deserialize(deserializer)?;
        match tagged.tag {
            Some(CID_CBOR_TAG) | None => {
                let res = tagged.value.to_vec();

                let cid = deserialize_cid_from_bytes(&res)
                    .map_err(|e| D::Error::custom(format!("Cid deserialize failed: {:}", e)))?;
                Ok(CborCid(cid))
            }
            Some(_) => Err(D::Error::custom("unexpected tag")),
        }
    }
}

impl From<Cid> for CborCid {
    fn from(cid: Cid) -> Self {
        CborCid(cid)
    }
}

impl Borrow<Cid> for CborCid {
    fn borrow(&self) -> &Cid {
        &self.0
    }
}

impl AsRef<Cid> for CborCid {
    fn as_ref(&self) -> &Cid {
        &self.0
    }
}

impl Deref for CborCid {
    type Target = Cid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CborCid {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
