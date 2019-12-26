use bytes::Bytes;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_tuple::*;

use cid::Cid;

#[derive(Debug, Clone, Eq, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct KV {
    pub key: String,
    pub value: Bytes,
}

impl KV {
    pub fn new(key: String, value: Vec<u8>) -> KV {
        KV {
            key,
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
pub enum PContent {
    #[serde(rename = "0")]
    Link(Cid),
    #[serde(rename = "1")]
    KVs(Vec<KV>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Pointer {
    data: PContent,
    // Option<>
}

impl Serialize for Pointer {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        self.data.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Pointer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let link = PContent::deserialize(deserializer)?;
        Ok(Pointer { data: link })
    }
}

impl Pointer {
    pub fn from_kvs(kvs: Vec<KV>) -> Self {
        Pointer {
            data: PContent::KVs(kvs),
        }
    }

    pub fn from_link(cid: Cid) -> Self {
        Pointer {
            data: PContent::Link(cid),
        }
    }

    pub fn content(&self) -> &PContent {
        &self.data
    }

    pub fn content_mut(&mut self) -> &mut PContent {
        &mut self.data
    }
}

pub struct Node {}

// hack for compile pass `Serialize_tuple` and `Deserialize_tuple`, use an empty struct to import `Serialize` and `Deserialize`
#[allow(unused)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct PlaceHolder;
