// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::borrow::Borrow;

use serde::de::{Deserialize, Deserializer, Error};
use serde::ser::{Serialize, Serializer};

use serde_cbor::tags::{DeserializerExt, SerializerExt};

use crate::Cid;
use std::ops::{Deref, DerefMut};

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
        let bytes = serde_cbor::Value::Bytes(b);
        s.serialize_cbor_tagged(CID_CBOR_TAG, &bytes)
    }
}

impl<'de> Deserialize<'de> for CborCid {
    fn deserialize<D>(deserializer: D) -> Result<CborCid, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.expect_cbor_tag(CID_CBOR_TAG)?;
        let v = serde_cbor::Value::deserialize(deserializer)?;
        let res = if let serde_cbor::Value::Bytes(b) = v {
            b
        } else {
            return Err(D::Error::custom(format!(
                "serde_cbor::Value must be Bytes type"
            )));
        };

        if res.len() == 0 {
            return Err(D::Error::custom(format!("Value was empty")));
        }

        if res[0] != 0 {
            return Err(D::Error::custom(format!("Invalid multibase on IPLD link")));
        }

        let cid = Cid::from(&res[1..])
            .map_err(|e| D::Error::custom(format!("Cid deserialize failed: {:}", e)))?;
        Ok(CborCid(cid))
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
