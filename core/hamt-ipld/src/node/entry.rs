// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::ops::{Deref, DerefMut};
use std::result;
use std::sync::RwLock;

use archery::{RcK, SharedPointer, SharedPointerKind};

use cid::Cid;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub use self::kv::KV;
use crate::error::*;
use crate::ipld::{BasicCborIpldStore, CborIpldStore};
use crate::node::{load_node, Node};
use std::cell::RefCell;
use std::io::Write;

mod kv {
    use serde_cbor::Value;
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

#[derive(Debug, Clone, Eq, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
pub enum PContent {
    #[serde(rename = "0")]
    Link(Cid),
    #[serde(rename = "1")]
    KVs(Vec<KV>),
}

#[derive(Debug)]
pub struct Pointer<B>
where
    B: CborIpldStore,
{
    pub data: PContent,
    pub cache: RefCell<Option<Box<Node>>>,
}

impl<B> PartialEq for Pointer<B>
where
    B: CborIpldStore,
{
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<B> Eq for Pointer<B> where B: CborIpldStore {}

//impl<B> Clone for Pointer<B>
//where
//    B: CborIpldStore,
//{
//    fn clone(&self) -> Self {
//        Pointer {
//            data: self.data.clone(),
//            cache: self.cache.clone(),
//        }
//    }
//}

impl<B> Serialize for Pointer<B>
where
    B: CborIpldStore,
{
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.data.serialize(serializer)
    }
}

impl<'de, B> Deserialize<'de> for Pointer<B>
where
    B: CborIpldStore,
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

impl<B> Pointer<B>
where
    B: CborIpldStore,
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

    pub fn load_child<F, R>(&self, cs: B, bit_width: u32, f: F) -> Result<R>
    where
        F: FnOnce(&mut Node) -> Result<R>,
    {
        {
            // TODO need to check mutable
            if let Some(n) = self.cache.borrow_mut().deref_mut() {
                return f(n);
            }
        }

        if let PContent::Link(ref cid) = self.data {
            let mut node = load_node(cs, bit_width, cid)?;
            let r = f(&mut node);
            *self.cache.borrow_mut() = Some(Box::new(node));
            r
        } else {
            unreachable!("current data must be a link");
        }
    }

    pub fn flush_cache(&mut self, cs: B) -> Result<()> {
        let mut cache_ref = self.cache.borrow_mut();
        if let Some(node) = cache_ref.deref_mut() {
            // TODO need to check if use r
            node.flush()?;
            let cid = cs.put(node)?;
            self.data = PContent::Link(cid);
        }
        *cache_ref = None;
        Ok(())
    }

    pub fn is_shared(&self) -> bool {
        match self.data {
            PContent::Link(_) => true,
            PContent::KVs(_) => false,
        }
    }

    //    pub fn deep_copy(&self) -> Pointer<B> {
    //        let cache = {
    //            let guard = self.cache.read().expect("must could get read lock here");
    //            if let Some(c) = guard.deref() {
    //                Some(c.deep_copy())
    //            } else {
    //                None
    //            }
    //        };
    //
    //        Pointer {
    //            data: self.data.clone(),
    //            cache: SharedPointer::new(RwLock::new(cache)),
    //        }
    //    }
}

// hack for compile pass `Serialize_tuple` and `Deserialize_tuple`, use an empty struct to import `Serialize` and `Deserialize`
#[allow(unused)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct PlaceHolder;
