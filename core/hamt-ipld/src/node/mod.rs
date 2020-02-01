// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

//pub mod entry;
pub mod trait_impl;

use std::cell::{RefCell, RefMut};
use std::collections::BTreeMap;

use bigint::U256;
use cid::Cid;
use ipld_cbor::{cbor_value_to_struct, struct_to_cbor_value};
use serde::{de::DeserializeOwned, Serialize};
use serde_cbor::Value;
use std::ops::{Deref, DerefMut};

//use self::entry::{PContent, Pointer};
//pub use self::trait_impl::PartNode;
use crate::error::*;
use crate::hash::{hash, HashBits};
use crate::ipld::{BasicCborIpldStore, CborIpldStore};
use std::borrow::Borrow;

const ARRAY_WIDTH: usize = 3;
pub const DEFAULT_BIT_WIDTH: u32 = 8;

pub struct Hamt<B>
where
    B: CborIpldStore,
{
    root: Node,
    bs: B,
    bit_width: u32,
}

pub type KV = BTreeMap<String, Value>;
pub type KVT = (String, Value);

#[derive(Debug, PartialEq, Eq)]
enum Item {
    Link(Cid),
    Ptr(Box<Node>),
    Leaf(KV),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Node {
    /// bitmap
    bitfield: U256,
    items: Vec<RefCell<Item>>,
}

#[inline]
pub fn set_bit(input: &mut U256, n: u32) {
    let one: U256 = 1.into();
    *input = *input | (one << n as usize)
}

#[inline]
pub fn unset_bit(input: &mut U256, n: u32) {
    let one: U256 = 1.into();
    *input = *input & !(one << n as usize)
}

/// index for bit position in this bitmap
#[inline]
pub fn bit_to_index(bitmap: &U256, bit_pos: u32) -> usize {
    let one: U256 = 1.into();
    let mask: U256 = (one << bit_pos as usize) - one;
    let r: U256 = mask & *bitmap;
    r.0.iter().fold(0, |a, b| a + b.count_ones() as usize)
}

impl<B> Hamt<B>
where
    B: CborIpldStore,
{
    pub fn new_with_bitwidth(store: B, bit_width: u32) -> Self {
        Hamt {
            root: Node::new(),
            bs: store,
            bit_width,
        }
    }
    pub fn new(store: B) -> Self {
        Self::new_with_bitwidth(store, DEFAULT_BIT_WIDTH)
    }

    pub fn load_with_bitwidth(store: B, cid: &Cid, bit_width: u32) -> Result<Self> {
        let root: Node = store.get(cid)?;
        Ok(Hamt {
            root,
            bs: store,
            bit_width,
        })
    }

    pub fn load(store: B, cid: &Cid) -> Result<Self> {
        Self::load_with_bitwidth(store, cid, DEFAULT_BIT_WIDTH)
    }

    pub fn find<Output: DeserializeOwned>(&self, k: &str) -> Result<Output> {
        Err(Error::InvalidFormatHAMT)
    }

    pub fn set<V: Serialize>(&mut self, k: &str, v: V) -> Result<()> {
        let hash = hash(k);
        let mut hash_bits = HashBits::new(hash.as_ref(), self.bit_width);
        let b = struct_to_cbor_value(&v).map_err(Error::IpldCbor)?;
        self.root.set(&self.bs, &mut hash_bits, k, b)
    }

    pub fn delete(&mut self, k: &str) -> Result<()> {
        Ok(())
    }
}

impl Item {
    pub fn from_kvs(kvs: Vec<KVT>) -> Self {
        Item::Leaf(kvs.into_iter().collect())
    }

    pub fn from_link(cid: Cid) -> Self {
        Item::Link(cid)
    }

    pub fn load_item<B>(&mut self, bs: &B) -> Result<()>
    where
        B: CborIpldStore,
    {
        if let Item::Link(cid) = self {
            let node: Node = bs.get(cid)?;
            *self = Item::Ptr(Box::new(node));
        }
        Ok(())
    }
}

impl Node {
    pub fn new() -> Self {
        Node {
            bitfield: U256::zero(),
            items: vec![],
        }
    }

    pub fn set<'hash, B>(
        &mut self,
        bs: &B,
        hv: &mut HashBits<'hash>,
        k: &str,
        v: Value,
    ) -> Result<()>
    where
        B: CborIpldStore,
    {
        let idx = hv.next().ok_or(Error::MaxDepth)?;
        if self.bitfield.bit(idx as usize) == false {
            return self.insert_child(idx, k, v);
        }
        let item_index = bit_to_index(&self.bitfield, idx);
        // TODO
        let item = self.items.get(item_index).expect("");
        // try load node from cid
        let mut item = item.borrow_mut();
        let item = item.deref_mut();
        item.load_item(bs)?;

        match item {
            Item::Link(_) => unreachable!("after `load_item`, should not be Link now"),
            Item::Ptr(node) => {
                // it's branch, recurse to fetch child
                node.set(bs, hv, k, v)
            }
            Item::Leaf(kvs) => {
                let leaf_item = kvs.get_mut(k);
                if let Some(old_v) = leaf_item {
                    // find item for this key, reset to new value
                    *old_v = v;
                    return Ok(());
                }

                // a new key/value, if not beyond leaf capacity, insert it directly
                if kvs.len() < ARRAY_WIDTH {
                    kvs.insert(k.to_string(), v);
                    return Ok(());
                }

                // current leaf is full, create a new branch and move leaf
                // notice the HashBits use different instance
                let mut child = Box::new(Node::new());
                let mut hash_copy = hv.clone();
                child.set(bs, &mut hash_copy, k, v)?;

                for (old_k, old_v) in kvs.iter() {
                    let new_hash = hash(old_k.as_bytes());
                    let mut ch_hv =
                        HashBits::new_with_consumed(new_hash.as_ref(), hv.consumed(), hv.bit_width);
                    // must use clone, not mem::swap, for this `set` function may be failed(e.g. MaxDepth)
                    // if failed, should not change the tree current struct
                    child.set(bs, &mut ch_hv, old_k, old_v.clone())?;
                }

                let child_item = Item::Ptr(child);
                *item = child_item;
                Ok(())
            }
        }
    }

    pub fn remove<'hash, B>(&mut self, bs: &B, hv: &mut HashBits<'hash>, k: &str) -> Result<()>
    where
        B: CborIpldStore,
    {
        let idx = hv.next().ok_or(Error::MaxDepth)?;
        if self.bitfield.bit(idx as usize) == false {
            return Err(Error::NotFound(k.to_string()));
        }
        let item_index = bit_to_index(&self.bitfield, idx);
        // TODO
        let item = self.items.get(item_index).expect("");
        // try load node from cid
        let mut item = item.borrow_mut();
        let item = item.deref_mut();
        match item {
            Item::Link(_) => unreachable!("after `load_item`, should not be Link now"),
            Item::Ptr(node) => {
                // it's branch, recurse to fetch child
                node.remove(bs, hv, k)?;
                node.clean_child(item)
            }
            Item::Leaf(kvs) => {
                let _ = kvs.remove(k).ok_or(Error::NotFound(k.to_string()))?;
                if kvs.is_empty() {
                    // the leaf is empty, should remove this leaf from this node
                    self.items.remove(item_index);
                    // set idx pos bit is zero
                    unset_bit(&mut self.bitfield, idx);
                }
                Ok(())
            }
        }
    }

    fn clean_child(&mut self, item: &mut Item) -> Result<()> {
        match item {
            Item::Ptr(node) => {
                let len = node.items.len();
                match len {
                    0 => Err(Error::InvalidFormatHAMT),
                    1 => {
                        let mut tmp_item = node.items[0].borrow_mut();
                        let mut child_item = tmp_item.deref_mut();
                        if let Item::Leaf(_) = child_item {
                            // it's safe, for child is sub item[0] for item
                            // use sub item[0] replace of current item, then old item would release directly
                            std::mem::swap(item, child_item);
                        }
                        Ok(())
                    }
                    x if x <= ARRAY_WIDTH => {
                        // should use clone instead of mem::swap, for this part may be return directly
                        let mut child_vals = KV::default();
                        for child_item in node.items.iter() {
                            match child_item.borrow().deref() {
                                Item::Leaf(kvs) => {
                                    for (k, v) in kvs.iter() {
                                        if child_vals.len() == ARRAY_WIDTH {
                                            return Ok(());
                                        }
                                        child_vals.insert(k.clone(), v.clone());
                                    }
                                }
                                _ => return Ok(()),
                            }
                        }
                        *item = Item::Leaf(child_vals);
                        Ok(())
                    }
                    _ => Ok(()),
                }
            }
            _ => unreachable!("`clean_child` param must be `Item::Ptr`"),
        }
    }

    //    pub fn check_size(&self) -> Result<u64> {
    //        let cid = self.store.put(&self)?;
    //        let blk = self.store.get_block(&cid)?;
    //        let mut total_size = blk.raw_data().len() as u64;
    //        for child in self.pointers.iter() {
    //            if child.is_shared() {
    //                child.load_child(self.store.clone(), self.bit_width, &mut |node| {
    //                    let child_size = node.check_size()?;
    //                    // TODO
    //                    total_size += child_size;
    //                    Ok(())
    //                })?;
    //                //
    //                //                let node = child_node.read().map_err(|_| Error::Lock)?;
    //                //                if let Some(n) = node.deref() {
    //                //                    let child_size = n.check_size()?;
    //                //                    total_size += child_size;
    //                //                } else {
    //                //                    unreachable!("node cache must be `Some()` here")
    //                //                }
    //            }
    //        }
    //        Ok(total_size)
    //    }

    /// insert k,v to this bit position.
    fn insert_child(&mut self, idx: u32, k: &str, v: Value) -> Result<()> {
        let i = bit_to_index(&self.bitfield, idx);
        // set bit for index i
        set_bit(&mut self.bitfield, idx);
        let leaf = Item::from_kvs(vec![(k.to_string(), v)]);
        self.items.insert(i as usize, RefCell::new(leaf));
        Ok(())
    }
}
