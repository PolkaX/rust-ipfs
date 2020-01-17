mod blocks;
mod error;
#[cfg(test)]
mod tests;
mod trait_impl;

use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_cbor::Value;

use cid::{zero_cid, Cid};

use crate::blocks::Blocks;
use crate::error::*;
use crate::trait_impl::PartRoot;

const WIDTH: usize = 8; // 2^3
const BITS_PER_SUBKEY: u64 = 3;
const SUBKEY_MASK: u64 = 8 - 1; // 2^3 - 1

fn index(h: u64, s: u64) -> usize {
    ((h >> s) & SUBKEY_MASK) as usize
}

fn mask(h: u64, s: u64) -> usize {
    1 << index(h, s)
}

pub struct Root<B>
where
    B: Blocks,
{
    height: u64,
    count: u64,
    node: Node,

    bs: Rc<RefCell<B>>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Node {
    bitmap: usize,
    links: Vec<Cid>,
    values: Vec<Value>,

    // cache
    cache: [RefCell<Option<Box<Node>>>; WIDTH],
}

#[cfg(test)]
pub fn create_node(bitmap: usize, links: Vec<Cid>, values: Vec<Value>) -> Node {
    Node {
        bitmap,
        links,
        values,
        cache: Default::default(),
    }
}

#[cfg(test)]
pub fn create_root<B: Blocks>(height: u64, count: u64, node: Node, bs: Rc<RefCell<B>>) -> Root<B> {
    Root {
        height,
        count,
        node,
        bs,
    }
}

impl<B> Root<B>
where
    B: Blocks,
{
    pub fn new(bs: B) -> Self {
        Root::<B> {
            height: 0,
            count: 0,
            node: Node::new(),
            bs: Rc::new(RefCell::new(bs)),
        }
    }

    pub fn from_partroot(part_root: PartRoot, bs: Rc<RefCell<B>>) -> Self {
        Root::<B> {
            height: part_root.0,
            count: part_root.1,
            node: part_root.2,
            bs,
        }
    }

    pub fn set<Input: Serialize>(&mut self, k: u64, input: Input) -> Result<()> {
        let v: Value = ipld_cbor::struct_to_cbor_value(&input)?;

        // extend amt tree first
        // if current key large then tree capacity, create a new root, and move current root be
        // sub node of new root, so that the tree is been extended.
        let mut tmp = k >> (BITS_PER_SUBKEY * (self.height + 1));
        while tmp != 0 {
            if !self.node.empty() {
                self.node.flush(self.bs.clone(), self.height)?;
                let cid = self.bs.borrow_mut().put(&self.node)?;
                self.node = Node::new_with_cid(cid);
            }
            tmp >>= BITS_PER_SUBKEY;
            self.height += 1;
        }
        let current_shift = BITS_PER_SUBKEY * self.height;
        let add = self
            .node
            .set(self.bs.clone(), self.height, k, v, current_shift)?;
        if add {
            self.count += 1;
        }
        Ok(())
    }

    pub fn get<Output: DeserializeOwned>(&self, k: u64) -> Result<Output> {
        let test = k >> (BITS_PER_SUBKEY * (self.height + 1));
        if test != 0 {
            // not found
            return Err(AmtIpldError::Tmp);
        }
        let current_shift = BITS_PER_SUBKEY * self.height;
        let v = self
            .node
            .get(self.bs.clone(), self.height, k, current_shift)?;
        let output = ipld_cbor::cbor_value_to_struct(v)?;
        Ok(output)
    }

    pub fn flush(&mut self) -> Result<Cid> {
        self.node.flush(self.bs.clone(), self.height)?;
        let cid = self.bs.borrow_mut().put(&self)?;
        Ok(cid)
    }
}

impl Node {
    pub fn new() -> Self {
        Node {
            bitmap: 0,
            links: vec![],
            values: vec![],
            cache: Default::default(),
        }
    }

    pub fn new_with_cid(cid: Cid) -> Self {
        Node {
            bitmap: 1,
            links: vec![cid],
            values: vec![],
            cache: Default::default(),
        }
    }

    fn set_bit(&mut self, index: usize) {
        let b = 1 << index;
        self.bitmap |= b;
    }

    fn get_bit(&self, index: usize) -> bool {
        (1 << index) & self.bitmap != 0
    }

    fn index_for_bitpos(&self, bit_pos: usize) -> usize {
        let mask = (1 << bit_pos) - 1;
        (mask & self.bitmap).count_ones() as usize
    }

    pub fn empty(&self) -> bool {
        self.bitmap == 0
    }

    pub fn set<B>(
        &mut self,
        bs: Rc<RefCell<B>>,
        height: u64,
        key: u64,
        v: Value,
        shift: u64,
    ) -> Result<bool>
    where
        B: Blocks,
    {
        // it's leaf node
        if height == 0 {
            let pos = index(key, shift);
            let exist = self.get_bit(pos);
            let index = self.index_for_bitpos(pos);
            if exist {
                self.values[index] = v; // must success, or this tree is broken
            } else {
                self.set_bit(pos);
                self.values.insert(index, v);
            }
            return Ok(!exist);
        }

        let i = index(key, shift);
        self.load_node_with_creating(bs.clone(), i, true, |node| {
            node.set(bs.clone(), height - 1, key, v, shift - BITS_PER_SUBKEY)
        })
    }

    pub fn get<B>(&self, bs: Rc<RefCell<B>>, height: u64, key: u64, shift: u64) -> Result<Value>
    where
        B: Blocks,
    {
        let i = index(height, shift);
        if !self.get_bit(i) {
            return Err(AmtIpldError::Tmp);
        }

        // touch leaf node, fetch value
        if height == 0 {
            let pos = self.index_for_bitpos(i);
            let v_ref = self.values.get(pos).expect("value list must match bitmap");
            return Ok(v_ref.clone());
        }

        self.load_node(bs.clone(), i, |sub_node| {
            sub_node.get(bs.clone(), height - 1, key, shift - BITS_PER_SUBKEY)
        })
    }

    pub fn flush<B>(&mut self, bs: Rc<RefCell<B>>, depth: u64) -> Result<()>
    where
        B: Blocks,
    {
        if depth == 0 {
            // do nothing for leaf
            return Ok(());
        }

        for i in 0..WIDTH {
            let cid_option = self.try_get_cache(i, |sub_node| -> Result<Cid> {
                sub_node.flush(bs.clone(), depth - 1)?;
                let db = bs.clone();
                let cid = db.borrow_mut().put(sub_node)?;
                Ok(cid)
            })?;

            if let Some(cid) = cid_option {
                // refresh link cid from cache
                let link_index = self.index_for_bitpos(i);
                let old = self
                    .links
                    .get_mut(link_index)
                    .expect("link must exist in flush");
                *old = cid;
            }
        }
        Ok(())
    }

    fn try_get_cache<F, R>(&self, index: usize, mut f: F) -> Result<Option<R>>
    where
        F: FnMut(&mut Self) -> Result<R>,
    {
        if let Some(node) = self.cache[index].borrow_mut().deref_mut() {
            return f(node).map(Some);
        }
        Ok(None)
    }

    fn load_node<B: Blocks, F, R>(&self, bs: Rc<RefCell<B>>, pos: usize, f: F) -> Result<R>
    where
        F: Fn(&Self) -> Result<R>,
    {
        if let Some(node) = self.cache[pos].borrow().deref() {
            return f(node);
        }
        if !self.get_bit(pos) {
            return Err(AmtIpldError::Tmp);
        }

        let pos = self.index_for_bitpos(pos);
        let n: Node = bs.borrow().get(&self.links[pos])?;
        let r = f(&n);
        *self.cache[pos].borrow_mut().deref_mut() = Some(Box::new(n));
        r
    }

    fn load_node_with_creating<B: Blocks, F, R>(
        &mut self,
        bs: Rc<RefCell<B>>,
        pos: usize,
        create: bool,
        f: F,
    ) -> Result<R>
    where
        F: FnOnce(&mut Self) -> Result<R>,
    {
        if let Some(n) = self.cache[pos].borrow_mut().deref_mut() {
            return f(n);
        }
        let index = self.index_for_bitpos(pos);
        let mut n = if self.get_bit(pos) {
            let n: Node = bs.borrow().get(&self.links[index])?;
            n
        } else {
            if create {
                let sub_node = Node::new();
                self.set_bit(pos);
                let mock = zero_cid();
                self.links.insert(index, mock);
                sub_node
            } else {
                return Err(AmtIpldError::Tmp);
            }
        };
        let r = f(&mut n);
        *self.cache[pos].borrow_mut().deref_mut() = Some(Box::new(n));
        r
    }
}
