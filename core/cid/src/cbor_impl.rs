use serde::de::{Deserialize, Deserializer, Error};
use serde::ser::{Serialize, Serializer};

use serde_cbor::tags::{DeserializerExt, SerializerExt};

use crate::Cid;

// TODO maybe not put here

/// CID_CBOR_TAG is the integer used to represent cid tags in CBOR.
pub const CID_CBOR_TAG: u64 = 42;

impl Serialize for Cid {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // add 0 at start
        let mut b = vec![0_u8];
        b.extend(self.to_bytes());
        s.serialize_cbor_tagged(CID_CBOR_TAG, &b)
    }
}

impl<'de> Deserialize<'de> for Cid {
    fn deserialize<D>(deserializer: D) -> Result<Cid, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.expect_cbor_tag(CID_CBOR_TAG)?;
        let res = Vec::<u8>::deserialize(deserializer)?;

        if res.len() == 0 {
            return Err(D::Error::custom(format!("Value was emply")));
        }

        if res[0] != 0 {
            return Err(D::Error::custom(format!("Invalid multibase on IPLD link")));
        }

        Ok(Cid::from(res)
            .map_err(|e| D::Error::custom(format!("Cid deserialize failed: {:}", e)))?)
    }
}
