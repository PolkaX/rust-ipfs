use serde::de::{Deserialize, Deserializer, Error as ErrorT};
use serde::ser::{Serialize, Serializer};
use serde_cbor::tags::Tagged;

use crate::cid::Cid;
use crate::error::Error;

/// CID_CBOR_TAG is the integer used to represent cid tags in CBOR.
pub const CID_CBOR_TAG: u64 = 42;

impl Serialize for Cid {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // add 0 at start, that binary multibase prefix...
        let mut b = vec![0_u8];
        b.extend(self.to_bytes());
        // Special process for bytes, due to serde_cbor would treat Vec<u8> as Array u8, not bytes
        let value = serde_bytes::Bytes::new(&b);
        Tagged::new(Some(CID_CBOR_TAG), value).serialize(s)
    }
}

/// util function for deserialize bytes into cid, notice this bytes first byte must be 0,
/// for CIDs must have binary multibase
pub fn deserialize_cid_from_bytes(res: &[u8]) -> Result<Cid, Error> {
    if res.len() == 0 || res[0] != 0 {
        return Err(multibase::Error::Other(
            "cbor serialized CIDs must have binary multibase or bytes is empty".to_string(),
        )
        .into());
    }

    let cid = Cid::from(&res[1..])?;
    Ok(cid)
}

impl<'de> Deserialize<'de> for Cid {
    fn deserialize<D>(deserializer: D) -> Result<Cid, D::Error>
    where
        D: Deserializer<'de>,
    {
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
