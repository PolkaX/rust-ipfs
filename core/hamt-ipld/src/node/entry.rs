use std::result;

use archery::SharedPointerKind;

use cid::Cid;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::*;

use super::NodeP;

mod kv {
    use bytes::Bytes;
    use serde_derive;
    use serde_tuple::*;
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

        pub fn set_value(&mut self, value: Vec<u8>) {
            self.value = value.into();
        }
    }
}

pub use kv::KV;

#[derive(Debug, Clone, Eq, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
pub enum PContent {
    #[serde(rename = "0")]
    Link(Cid),
    #[serde(rename = "1")]
    KVs(Vec<KV>),
}

#[derive(Debug, Eq, PartialEq)]
pub struct Pointer<P>
where
    P: SharedPointerKind + Clone,
{
    pub data: PContent,
    pub cache: Option<NodeP<P>>,
}

impl<P> Clone for Pointer<P>
where
    P: SharedPointerKind + Clone,
{
    fn clone(&self) -> Self {
        Pointer {
            data: self.data.clone(),
            cache: self.cache.clone(),
        }
    }
}

impl<P> Serialize for Pointer<P>
where
    P: SharedPointerKind + Clone,
{
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.data.serialize(serializer)
    }
}

impl<'de, P> Deserialize<'de> for Pointer<P>
where
    P: SharedPointerKind + Clone,
{
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let link = PContent::deserialize(deserializer)?;
        Ok(Pointer {
            data: link,
            cache: None,
        })
    }
}

impl<P> Pointer<P>
where
    P: SharedPointerKind + Clone,
{
    pub fn from_kvs(kvs: Vec<KV>) -> Self {
        Pointer {
            data: PContent::KVs(kvs),
            cache: None,
        }
    }

    pub fn from_link(cid: Cid) -> Self {
        Pointer {
            data: PContent::Link(cid),
            cache: None,
        }
    }

    pub fn content(&self) -> &PContent {
        &self.data
    }

    pub fn content_mut(&mut self) -> &mut PContent {
        &mut self.data
    }

    pub fn load_child(&self, _bit_width: u32) -> Result<NodeP<P>> {
        if let Some(ref cache) = self.cache {
            return Ok((*cache).clone());
        }
        // TODO
        //        Ok(())
        Err(Error::Tmp)
    }
}

// hack for compile pass `Serialize_tuple` and `Deserialize_tuple`, use an empty struct to import `Serialize` and `Deserialize`
#[allow(unused)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct PlaceHolder;
