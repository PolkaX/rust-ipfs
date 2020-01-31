// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

pub mod entry;
pub mod trait_impl;

use archery::{ArcK, RcK, SharedPointer, SharedPointerKind};
use bigint::U256;
use cid::Cid;
use ipld_cbor::{cbor_value_to_struct, struct_to_cbor_value};
use serde::{de::DeserializeOwned, Serialize};
use serde_cbor::Value;
use std::ops::{Deref, DerefMut};

use self::entry::{PContent, Pointer, KV};
pub use self::trait_impl::PartNode;
use crate::error::*;
use crate::hash::{hash, HashBits};
use crate::ipld::{BasicCborIpldStore, CborIpldStore};

const ARRAY_WIDTH: usize = 3;
pub const DEFAULT_BIT_WIDTH: u32 = 8;

#[derive(Debug)]
pub struct Node<B>
where
    B: CborIpldStore,
{
    // we use u64 here, for normally a branch of node would not over 64, 64 branch's wide is so large, if larger then 64, panic
    /// bitmap
    bitfield: U256,
    /// branch node
    pointers: Vec<Pointer<B>>,

    /// for fetching and storing children
    store: B,
    bit_width: u32,
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
pub fn index_for_bitpos(bitmap: &U256, bit_pos: u32) -> u32 {
    let one: U256 = 1.into();
    let mask: U256 = (one << bit_pos as usize) - one;
    let r: U256 = mask & *bitmap;
    r.0.iter().fold(0, |a, b| a + b.count_ones())
}

impl<B> Node<B>
where
    B: CborIpldStore,
{
    #[cfg(test)]
    pub fn test_init(store: B, bitfield: &str, pointers: Vec<Pointer<B>>, bit_width: u32) -> Self {
        Node {
            bitfield: U256::from_dec_str(bitfield).unwrap(),
            pointers,
            store,
            bit_width,
        }
    }

    pub fn new(store: B) -> Node<B> {
        Self::new_with_bitwidth(store, DEFAULT_BIT_WIDTH)
    }

    pub fn new_with_bitwidth(store: B, bit_width: u32) -> Node<B> {
        Node {
            bitfield: 0.into(),
            pointers: vec![],
            store,
            bit_width,
        }
    }

    pub fn load_node(store: B, c: Cid) -> Result<Node<B>> {
        Self::load_node_with_bitwidth(store, c, DEFAULT_BIT_WIDTH)
    }

    pub fn load_node_with_bitwidth(store: B, c: Cid, bit_width: u32) -> Result<Node<B>> {
        let pn: PartNode<B> = store.get(&c)?;
        let node = pn.into_node(store, bit_width);
        Ok(node)
    }

    pub fn get_mut_bitfield(&mut self) -> &mut U256 {
        &mut self.bitfield
    }

    pub fn get_mut_pointers(&mut self) -> &mut Vec<Pointer<B>> {
        &mut self.pointers
    }

    pub fn get_pointers(&self) -> &Vec<Pointer<B>> {
        &self.pointers
    }

    pub fn get_bitwidth(&self) -> u32 {
        self.bit_width
    }

    pub fn get_store(&self) -> B {
        self.store.clone()
    }

    pub fn find<Output: DeserializeOwned>(&self, k: &str) -> Result<Output> {
        let hash = hash(k);
        let mut hash_bits = HashBits::new(hash.as_ref());
        self.get_value(&mut hash_bits, k)
            .and_then(|v| cbor_value_to_struct(v).map_err(Error::IpldCbor))
    }

    pub fn delete(&mut self, k: &str) -> Result<()> {
        let hash = hash(k);
        let mut hash_bits = HashBits::new(hash.as_ref());
        self.modify_value(&mut hash_bits, k, None)
    }

    pub fn set<V: Serialize>(&mut self, k: &str, v: V) -> Result<()> {
        let hash = hash(k);
        let mut hash_bits = HashBits::new(hash.as_ref());
        let b = struct_to_cbor_value(&v).map_err(Error::IpldCbor)?;

        self.modify_value(&mut hash_bits, k, Some(b))
    }

    pub fn flush(&mut self) -> Result<()> {
        for p in self.pointers.iter_mut() {
            p.flush_cache(self.store.clone())?;
            //            p.flush_cache(|pointer, node| {
            //                node.flush()?;
            //                let cid = self.store.clone().put(node)?;
            //                *pointer.data = PContent::Link(cid);
            //            })?;
            //            let mut guard_cache = p.cache.write().map_err(|_| Error::Lock)?;
            //            if let Some(ref mut cache) = guard_cache.deref_mut() {
            //                cache.flush()?;
            //                let cid = self.store.put(cache)?;
            //                p.data = PContent::Link(cid);
            //            }
            //            // clear cache
            //            *guard_cache = None;
        }
        Ok(())
    }

    pub fn check_size(&self) -> Result<u64> {
        let cid = self.store.put(&self)?;
        let blk = self.store.get_block(&cid)?;
        let mut total_size = blk.raw_data().len() as u64;
        for child in self.pointers.iter() {
            if child.is_shared() {
                child.load_child(self.store.clone(), self.bit_width, &mut |node| {
                    let child_size = node.check_size()?;
                    // TODO
                    total_size += child_size;
                    Ok(())
                })?;
                //
                //                let node = child_node.read().map_err(|_| Error::Lock)?;
                //                if let Some(n) = node.deref() {
                //                    let child_size = n.check_size()?;
                //                    total_size += child_size;
                //                } else {
                //                    unreachable!("node cache must be `Some()` here")
                //                }
            }
        }
        Ok(total_size)
    }

    fn get_value<'hash>(&self, hash_bits: &mut HashBits<'hash>, k: &str) -> Result<Value> {
        let idx = hash_bits.next(self.bit_width).ok_or(Error::MaxDepth)?;
        if self.bitfield.bit(idx as usize) == false {
            return Err(Error::NotFound(k.to_string()));
        }
        let child_index = index_for_bitpos(&self.bitfield, idx) as usize;
        let child = self
            .pointers
            .get(child_index)
            .expect("[get_value]should not happen, bit counts must match pointers");
        match child.data {
            // for leaf
            PContent::KVs(ref kvs) => {
                for kv in kvs.iter() {
                    if kv.key == k {
                        return Ok(kv.value.clone());
                    }
                }
                Err(Error::NotFound(k.to_string()))
            }
            // for branch, recursion for `get_value`
            PContent::Link(_) => child.load_child(self.store.clone(), self.bit_width, |node| {
                node.get_value(hash_bits, k)
            }),
        }
    }

    fn modify_value<'hash>(
        &mut self,
        hv: &mut HashBits<'hash>,
        k: &str,
        v: Option<Value>,
    ) -> Result<()> {
        let idx = hv.next(self.bit_width).ok_or(Error::MaxDepth)?;
        // bitmap do not have this bit, it's a new key for this bit position.
        // create a kv pointer to store this value
        if self.bitfield.bit(idx as usize) == false {
            return self.insert_child(idx, k, v);
        }

        let cindex = index_for_bitpos(&self.bitfield, idx);
        let child = self
            .pointers
            .get_mut(cindex as usize)
            .expect("[modify_value]should not happen, bit counts must match pointers");
        let need_delete = v.is_none();
        match child.data {
            // id child is branch
            PContent::Link(_) => {
                child.load_child(self.store.clone(), self.bit_width, |node| {
                    node.modify_value(hv, k, v)?;
                    if need_delete {
                        return self.clean_child(node, cindex);
                    }
                    Ok(())
                })?;
                Ok(())
            }
            // if child is leaf
            PContent::KVs(ref mut kvs) => {
                // when need to remove this key/value pair
                if need_delete {
                    let old_len = kvs.len();
                    // remove pair when key equal to k
                    kvs.retain(|entry| entry.key != k);

                    let result = if kvs.is_empty() {
                        // no pair left, remove this child node,
                        // notice this is removing for current node pointer directly,
                        // not handle for child
                        self.remove_child(cindex, idx);
                        Ok(())
                    } else if old_len == kvs.len() {
                        // no pair could be removed
                        Err(Error::NotFound(k.to_string()))
                    } else {
                        // normally remove one pair from kvs.
                        Ok(())
                    };
                    return result;
                }

                // check if key already exists
                for kv in kvs.iter_mut() {
                    // find key already has a value, replace it to new value
                    if kv.key == k {
                        kv.set_value(v.unwrap());
                        return Ok(());
                    }
                }

                // If the array is full, create a subshard and insert everything into it
                if kvs.len() >= ARRAY_WIDTH {
                    let mut sub = Node::<B>::new_with_bitwidth(self.store.clone(), self.bit_width);
                    let mut hash_copy = hv.clone();
                    sub.modify_value(&mut hash_copy, k, v)?;

                    for p in kvs.iter() {
                        let new_hash = hash(p.key.as_bytes());
                        let mut ch_hv =
                            HashBits::new_with_consumed(new_hash.as_ref(), hv.consumed());
                        sub.modify_value(&mut ch_hv, p.key.as_str(), Some(p.value.clone()))?;
                    }

                    let c = self.store.put(sub)?;
                    // todo set cache as well
                    let pointer = Pointer::from_link(c);
                    return self.set_child(cindex, pointer);
                }

                // otherwise insert the new element into the array in order
                let np = KV::new(k.to_string(), v.unwrap());
                kvs.push(np);
                // TODO need to check string sort rule
                kvs.sort_by(|a, b| a.key.cmp(&b.key));

                Ok(())
            }
        }
    }

    /// insert k,v to this bit position.
    fn insert_child(&mut self, idx: u32, k: &str, v: Option<Value>) -> Result<()> {
        // in insert, the value must exist, `None` represent delete this key.
        let v = v.ok_or_else(|| Error::NotFound(k.to_string()))?;

        let i = index_for_bitpos(&self.bitfield, idx);
        // set bit for index i
        set_bit(&mut self.bitfield, idx);

        // new pointer
        let p = Pointer::from_kvs(vec![KV::new(k.to_string(), v)]);
        self.pointers.insert(i as usize, p);
        Ok(())
    }

    fn clean_child(&mut self, child_node: &mut Node<B>, child_index: u32) -> Result<()> {
        let len = child_node.pointers.len();
        match len {
            0 => Err(Error::InvalidFormatHAMT),
            1 => {
                // TODO: only do this if its a value, cant do this for shards unless pairs requirements are met.

                if child_node.pointers[0].is_shared() {
                    return Ok(());
                }
                let mut empty_ptr = Pointer::from_kvs(vec![]);
                // this is safe, for child_node would be replace in `set_child` later
                std::mem::swap(&mut child_node.pointers[0], &mut empty_ptr);
                self.set_child(child_index, empty_ptr)
            }
            x if x <= ARRAY_WIDTH => {
                let mut chvals = vec![];
                for p in child_node.pointers.iter() {
                    match p.data {
                        PContent::Link(_) => return Ok(()),
                        PContent::KVs(ref kvs) => {
                            for kv in kvs {
                                if chvals.len() == ARRAY_WIDTH {
                                    return Ok(());
                                }
                                // TODO use mem::swap to avoid clone
                                chvals.push(kv.clone());
                            }
                        }
                    }
                }
                self.set_child(child_index, Pointer::from_kvs(chvals))
            }
            _ => Ok(()),
        }
    }

    fn remove_child(&mut self, i: u32, idx: u32) {
        self.pointers.remove(i as usize);
        // set idx pos bit is zero
        unset_bit(&mut self.bitfield, idx);
    }

    fn set_child(&mut self, idx: u32, p: Pointer<B>) -> Result<()> {
        let v = self
            .pointers
            .get_mut(idx as usize)
            .expect("[set_child]should not happen, bit counts must match pointers");
        *v = p;
        Ok(())
    }

    //    pub fn deep_copy(&self) -> Node<B> {
    //        Node::<B> {
    //            bitfield: self.bitfield,
    //            pointers: self.pointers.iter().map(|p| p.deep_copy()).collect(),
    //            store: self.store.clone(),
    //            bit_width: self.bit_width,
    //        }
    //    }
}

pub fn load_node<B>(cs: B, bit_width: u32, cid: &Cid) -> Result<Node<B>>
where
    B: CborIpldStore,
{
    let pn: PartNode<B> = cs.get(cid)?;
    Ok(pn.into_node(cs, bit_width))
}
