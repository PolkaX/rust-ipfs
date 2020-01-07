use std::ops::{Deref, DerefMut};
use std::result;
use std::sync::RwLock;

use archery::{RcK, SharedPointer, SharedPointerKind};

use cid::Cid;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::*;
use crate::ipld::{Blocks, CborIpldStor};

use super::{load_node, Node};

mod kv {
    use serde_cbor::Value;
    use serde_derive;
    use serde_tuple::*;
    #[derive(Debug, Clone, Eq, PartialEq, Serialize_tuple, Deserialize_tuple)]
    pub struct KV {
        pub key: String,
        pub value: Value,
    }

    impl KV {
        pub fn new(key: String, value: Value) -> KV {
            KV { key, value }
        }

        pub fn set_value(&mut self, value: Value) {
            self.value = value;
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

#[derive(Debug)]
pub struct Pointer<B, P = RcK>
where
    B: Blocks,
    P: SharedPointerKind,
{
    pub data: PContent,
    pub cache: SharedPointer<RwLock<Option<Node<B, P>>>, P>,
}

impl<B, P> PartialEq for Pointer<B, P>
where
    B: Blocks,
    P: SharedPointerKind,
{
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<B, P> Eq for Pointer<B, P>
where
    B: Blocks,
    P: SharedPointerKind,
{
}

impl<B, P> Clone for Pointer<B, P>
where
    B: Blocks,
    P: SharedPointerKind,
{
    fn clone(&self) -> Self {
        Pointer {
            data: self.data.clone(),
            cache: self.cache.clone(),
        }
    }
}

impl<B, P> Serialize for Pointer<B, P>
where
    B: Blocks,
    P: SharedPointerKind,
{
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.data.serialize(serializer)
    }
}

impl<'de, B, P> Deserialize<'de> for Pointer<B, P>
where
    B: Blocks,
    P: SharedPointerKind,
{
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let link = PContent::deserialize(deserializer)?;
        Ok(Pointer {
            data: link,
            cache: SharedPointer::new(RwLock::new(None)),
        })
    }
}

impl<B, P> Pointer<B, P>
where
    B: Blocks,
    P: SharedPointerKind,
{
    pub fn from_kvs(kvs: Vec<KV>) -> Self {
        Pointer {
            data: PContent::KVs(kvs),
            cache: SharedPointer::new(RwLock::new(None)),
        }
    }

    pub fn from_link(cid: Cid) -> Self {
        Pointer {
            data: PContent::Link(cid),
            cache: SharedPointer::new(RwLock::new(None)),
        }
    }

    pub fn content(&self) -> &PContent {
        &self.data
    }

    pub fn content_mut(&mut self) -> &mut PContent {
        &mut self.data
    }

    pub fn load_child(
        &self,
        cs: CborIpldStor<B>,
        bit_width: u32,
    ) -> Result<SharedPointer<RwLock<Option<Node<B, P>>>, P>> {
        {
            if self.cache.read().map_err(|_| Error::Lock)?.is_some() {
                return Ok(self.cache.clone());
            }
        }

        if let PContent::Link(ref cid) = self.data {
            let node = load_node(cs, bit_width, cid)?;
            {
                let mut guard = self.cache.write().map_err(|_| Error::Lock)?;
                *(guard.deref_mut()) = Some(node);
            }
            Ok(self.cache.clone())
        } else {
            unreachable!("current data must be a link");
        }
    }

    pub fn is_shared(&self) -> bool {
        match self.data {
            PContent::Link(_) => true,
            PContent::KVs(_) => false,
        }
    }

    pub fn deep_copy(&self) -> Pointer<B, P> {
        let cache = {
            let guard = self.cache.read().expect("must could get read lock here");
            if let Some(c) = guard.deref() {
                Some(c.deep_copy())
            } else {
                None
            }
        };

        Pointer {
            data: self.data.clone(),
            cache: SharedPointer::new(RwLock::new(cache)),
        }
    }
}

// hack for compile pass `Serialize_tuple` and `Deserialize_tuple`, use an empty struct to import `Serialize` and `Deserialize`
#[allow(unused)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct PlaceHolder;
