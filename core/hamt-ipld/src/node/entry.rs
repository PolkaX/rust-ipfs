use log::trace;
use std::cell::RefCell;
use std::result;

use archery::{RcK, SharedPointer, SharedPointerKind};

use cid::Cid;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::*;
use crate::ipld::{Blocks, CborIpldStor};

use super::{load_node, NodeP};

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

#[derive(Debug)]
pub struct Pointer<B, P = RcK>
where
    B: Blocks,
    P: SharedPointerKind,
{
    pub data: PContent,
    pub cache: RefCell<Option<NodeP<B, P>>>,
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
            cache: RefCell::new(None),
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
            cache: RefCell::new(None),
        }
    }

    pub fn from_link(cid: Cid) -> Self {
        Pointer {
            data: PContent::Link(cid),
            cache: RefCell::new(None),
        }
    }

    pub fn content(&self) -> &PContent {
        &self.data
    }

    pub fn content_mut(&mut self) -> &mut PContent {
        &mut self.data
    }

    pub fn load_child(&self, cs: CborIpldStor<B>, bit_width: u32) -> Result<NodeP<B, P>> {
        if let Some(cache) = self.cache.borrow().as_ref() {
            return Ok((*cache).clone());
        }
        if let PContent::Link(ref cid) = self.data {
            let node = load_node(cs, bit_width, cid)?;
            let node_p = SharedPointer::new(node);
            // just try to get borrow mut, if failed, means other thread is borrow this node cache, would not set cache.
            if let Ok(mut r) = self.cache.try_borrow_mut() {
                *r = Some(node_p.clone());
            } else {
                trace!(target: "hamt-pointer", "try to get borrow mut for cache failed, other thread hold this borrow. cid:{:}", cid.to_string());
            }
            Ok(node_p)
        } else {
            // TODO must be link
            Err(Error::Tmp)
        }
    }
}

// hack for compile pass `Serialize_tuple` and `Deserialize_tuple`, use an empty struct to import `Serialize` and `Deserialize`
#[allow(unused)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct PlaceHolder;
