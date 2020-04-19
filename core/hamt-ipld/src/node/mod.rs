// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

pub mod trait_impl;

use std::cell::RefCell;
use std::collections::BTreeMap;

use bigint::U256;
use cid::Cid;
use ipld_cbor::{cbor_value_to_struct, struct_to_cbor_value};
use serde::{de::DeserializeOwned, Serialize};
use serde_cbor::Value;

use crate::error::*;
use crate::hash::{hash, HashBits};
use crate::ipld::CborIpldStore;

const ARRAY_WIDTH: usize = 3;
pub const DEFAULT_BIT_WIDTH: u32 = 8;

/// Hamt struct, hold root node. for public, we use `Hamt`, not `Node`
/// current `Hamt` is not thread safe, if want to use `Hamt` is multi thread, must use
/// lock to wrap `Hamt`
pub struct Hamt<B>
where
    B: CborIpldStore,
{
    /// root node of `Hamt`
    root: Node,
    /// database to store the relationship of cid and node
    bs: B,
    bit_width: u32,
}

pub type KV = BTreeMap<String, Value>;
pub type KVT = (String, Value);

/// Item would be `Link` `Ptr` and `Leaf`, but in factor, `Ptr` is the cache of `Link`.
/// when call `load_item`, the `Link` would convert to `Ptr`.
/// when call `flush`, the `Ptr` would refresh the `Link`
/// when serialize/deserialize, should not serialize `Ptr`, otherwise would panic.
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(Clone))]
pub enum Item {
    Link(Cid),
    Ptr(Box<Node>),
    Leaf(KV),
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(Clone))]
pub struct Node {
    /// bitmap, we use U256 replace bigint, for we think the bit_width and HashBits couldn't
    /// more then 256bit
    bitfield: U256,
    /// `Item` is wrapped by `Refcell` due to items would load in immutable `get` call.
    items: RefCell<Vec<Item>>,
}

#[cfg(not(feature = "nightly"))]
impl<'a> cid_ext::HasCid for &'a Node {
    fn has_cid(&self) -> Option<&Cid> {
        None
    }
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
    /// create a new empty Hamt with bit_width
    pub fn new_with_bitwidth(store: B, bit_width: u32) -> Self {
        Hamt {
            root: Node::new(),
            bs: store,
            bit_width,
        }
    }

    /// create a new empty Hamt
    pub fn new(store: B) -> Self {
        Self::new_with_bitwidth(store, DEFAULT_BIT_WIDTH)
    }

    /// load Hamt from cid with bitwidth
    pub fn load_with_bitwidth(store: B, cid: &Cid, bit_width: u32) -> Result<Self> {
        let root: Node = store.get(cid)?;
        Ok(Hamt {
            root,
            bs: store,
            bit_width,
        })
    }

    /// load Hamt from cid
    pub fn load(store: B, cid: &Cid) -> Result<Self> {
        Self::load_with_bitwidth(store, cid, DEFAULT_BIT_WIDTH)
    }

    pub fn bit_width(&self) -> u32 {
        self.bit_width
    }

    pub fn root(&self) -> &Node {
        &self.root
    }

    /// get a value for k, if not find, would return Error::NotFound
    pub fn find<Output: DeserializeOwned>(&self, k: &str) -> Result<Output> {
        let hash = hash(k);
        let mut hash_bits = HashBits::new(hash.as_ref(), self.bit_width);
        let v = self.root.get(&self.bs, &mut hash_bits, k, |v| {
            cbor_value_to_struct(v.clone()).map_err(Error::IpldCbor)
        })?;
        Ok(v)
    }

    /// set a value for k, if the value is already exist, override it.
    pub fn set<V: Serialize>(&mut self, k: &str, v: V) -> Result<()> {
        let hash = hash(k);
        let mut hash_bits = HashBits::new(hash.as_ref(), self.bit_width);
        let b = struct_to_cbor_value(&v).map_err(Error::IpldCbor)?;
        self.root.set(&self.bs, &mut hash_bits, k, b)
    }

    /// delete for k, if the k is not exist, return Error::NotFound
    pub fn delete(&mut self, k: &str) -> Result<()> {
        let hash = hash(k);
        let mut hash_bits = HashBits::new(hash.as_ref(), self.bit_width);
        self.root.remove(&self.bs, &mut hash_bits, k)
    }

    /// flush all `Ptr` into `Link`, every flush should treat as `Commit`,
    /// means commit current all changes into database, and generate changed cids.
    /// the operation equals to persistence. if store the root cid, then the Hamt is immutable,
    /// could recover all child status from any root cid.
    pub fn flush(&mut self) -> Result<Cid> {
        self.root.flush(&mut self.bs)?;
        self.bs.put(&self.root)
    }

    /// just for test
    #[cfg(test)]
    pub fn check_size(&mut self) -> Result<u64> {
        self.flush()?;
        self.root.check_size(&mut self.bs)
    }
}

impl Item {
    pub fn from_kvs(kvs: Vec<KVT>) -> Self {
        Item::Leaf(kvs.into_iter().collect())
    }

    pub fn from_link(cid: Cid) -> Self {
        Item::Link(cid)
    }

    fn load_item<B>(&mut self, bs: &B) -> Result<()>
    where
        B: CborIpldStore,
    {
        if let Item::Link(cid) = self {
            let node: Node = bs.get(cid)?;
            *self = Item::Ptr(Box::new(node));
        }
        Ok(())
    }

    fn clean_child(&mut self) -> Result<()> {
        match self {
            Item::Ptr(node) => {
                let len = node.items.borrow().len();
                match len {
                    0 => Err(Error::InvalidFormatHAMT),
                    1 => {
                        // this branch means that if current node's child only have one child,
                        // and this child first sub-child is a leaf, then use sub-child replace
                        // current child directly.
                        // due to rust mutable check, when we hold `self` ref, we can't
                        // call mem::swap(self, node.items[0]), so that we just use a `tmp` item to
                        // swap first, and then move the `tmp` item replace `self`
                        let should_move_leaf = {
                            let mut items = node.items.borrow_mut();
                            let leaf = &mut items[0];
                            if let Item::Leaf(_) = leaf {
                                // it's safe, for current child would be release after `*self = leaf`
                                // so that we use a `tmp` Item to replace current sub-child,
                                // and now `tmp` is `sub-child`
                                let mut tmp = Item::Leaf(Default::default());
                                std::mem::swap(&mut tmp, leaf);
                                Some(tmp)
                            } else {
                                // if sub-child is not a leaf, do nothing.
                                None
                            }
                        };
                        // if sub-child is not leaf, this if branch would not hit
                        if let Some(leaf) = should_move_leaf {
                            *self = leaf
                        }
                        Ok(())
                    }
                    x if x <= ARRAY_WIDTH => {
                        // should use clone instead of mem::swap, for this part may be return directly
                        let mut child_vals = KV::default();
                        for child_item in node.items.borrow().iter() {
                            match child_item {
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
                        *self = Item::Leaf(child_vals);
                        Ok(())
                    }
                    _ => Ok(()),
                }
            }
            _ => unreachable!("`clean_child` param must be `Item::Ptr`"),
        }
    }
}

#[cfg(test)]
pub fn test_node(bitfield: &str, items: Vec<Item>) -> Node {
    Node::from_raw(U256::from_dec_str(bitfield).unwrap(), items)
}

impl Node {
    fn new() -> Self {
        Node {
            bitfield: U256::zero(),
            items: Default::default(),
        }
    }

    fn from_raw(bitfield: U256, items: Vec<Item>) -> Self {
        Node {
            bitfield,
            items: RefCell::new(items),
        }
    }

    fn get<'hash, B, F, Output>(
        &self,
        bs: &B,
        hv: &mut HashBits<'hash>,
        k: &str,
        f: F,
    ) -> Result<Output>
    where
        B: CborIpldStore,
        F: Fn(&Value) -> Result<Output>,
    {
        let idx = hv.next().ok_or(Error::MaxDepth)?;
        if self.bitfield.bit(idx as usize) == false {
            return Err(Error::NotFound(k.to_string()));
        }
        let child_index = bit_to_index(&self.bitfield, idx);
        // load_item first
        self.items.borrow_mut()[child_index].load_item(bs)?;

        let items = self.items.borrow();
        let child = &items[child_index];
        match child {
            Item::Link(_) => unreachable!("after `load_item`, should not be Link now"),
            Item::Ptr(node) => node.get(bs, hv, k, f),
            Item::Leaf(kvs) => kvs
                .get(k)
                .ok_or(Error::NotFound(k.to_string()))
                .and_then(|v| f(v)),
        }
    }

    fn set<'hash, B>(&mut self, bs: &B, hv: &mut HashBits<'hash>, k: &str, v: Value) -> Result<()>
    where
        B: CborIpldStore,
    {
        let idx = hv.next().ok_or(Error::MaxDepth)?;
        if self.bitfield.bit(idx as usize) == false {
            return self.insert_child(idx, k, v);
        }
        let item_index = bit_to_index(&self.bitfield, idx);

        let mut items = self.items.borrow_mut();

        let item = &mut items[item_index];
        // try load node from cid
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

        let mut items = self.items.borrow_mut();
        let item = &mut items[item_index];
        // try load node from cid
        item.load_item(bs)?;

        match item {
            Item::Link(_) => unreachable!("after `load_item`, should not be Link now"),
            Item::Ptr(node) => {
                // it's branch, recurse to fetch child
                node.remove(bs, hv, k)?;
                // return directly
                item.clean_child()
            }
            Item::Leaf(kvs) => {
                let _ = kvs.remove(k).ok_or(Error::NotFound(k.to_string()))?;
                if kvs.is_empty() {
                    items.remove(item_index);
                    // set idx pos bit is zero
                    unset_bit(&mut self.bitfield, idx);
                }
                Ok(())
            }
        }
    }

    fn flush<B>(&mut self, bs: &mut B) -> Result<()>
    where
        B: CborIpldStore,
    {
        let mut items = self.items.borrow_mut();
        for item in &mut items[..].iter_mut() {
            if let Item::Ptr(node) = item {
                node.flush(bs)?;
                let cid = bs.put(&**node)?;
                // flush current item
                *item = Item::Link(cid);
            }
        }
        Ok(())
    }

    #[cfg(test)]
    fn check_size<B>(&self, bs: &mut B) -> Result<u64>
    where
        B: CborIpldStore,
    {
        let cid = bs.put(&self)?;
        let node: Node = bs.get(&cid)?;
        let mut total_size = ipld_cbor::dump_object(&node)?.len() as u64;
        for item in self.items.borrow_mut().iter_mut() {
            item.load_item(bs)?;
            if let Item::Ptr(node) = item {
                let child_size = node.check_size(bs)?;
                total_size += child_size;
            }
        }
        Ok(total_size)
    }

    /// insert k,v to this bit position.
    fn insert_child(&mut self, idx: u32, k: &str, v: Value) -> Result<()> {
        let i = bit_to_index(&self.bitfield, idx);
        // set bit for index i
        set_bit(&mut self.bitfield, idx);
        let leaf = Item::from_kvs(vec![(k.to_string(), v)]);
        self.items.borrow_mut().insert(i as usize, leaf);
        Ok(())
    }
}

#[cfg(test)]
#[derive(Debug, Default)]
pub struct HamtStats {
    total_nodes: usize,
    total_kvs: usize,
    counts: BTreeMap<usize, usize>,
}

#[cfg(test)]
pub fn stats<B>(hamt: &Hamt<B>) -> HamtStats
where
    B: CborIpldStore,
{
    let mut st = HamtStats::default();
    stats_rec(&hamt.bs, &hamt.root, &mut st);
    st
}

#[cfg(test)]
fn stats_rec<B>(bs: &B, node: &Node, st: &mut HamtStats)
where
    B: CborIpldStore,
{
    st.total_nodes += 1;

    let mut items = node.items.borrow_mut();
    for p in items.iter_mut() {
        p.load_item(bs).unwrap();
        match p {
            Item::Link(_) => unreachable!(""),
            Item::Ptr(node) => stats_rec(bs, node, st),
            Item::Leaf(kvs) => {
                st.total_kvs += kvs.len();
                *(st.counts.entry(kvs.len()).or_insert(0)) += 1;
            }
        }
    }
}
