// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

mod trait_impl;
//mod iter;

use std::cell::RefCell;
use std::ops::DerefMut;

use cid::Cid;
use serde::{de::DeserializeOwned, Serialize};
use serde_cbor::Value;

use self::trait_impl::PartAmt;
use crate::blocks::Blocks;
use crate::error::*;

//const WIDTH: usize = 8; // 2^3
const BITS_PER_SUBKEY: u64 = 3; // 3 is for bit_width 8 (2^3=8)
const SUBKEY_MASK: u64 = 8 - 1; // 2^3 - 1

fn bit_pos(h: u64, s: u64) -> usize {
    ((h >> s) & SUBKEY_MASK) as usize
}

pub struct Amt<B>
where
    B: Blocks,
{
    height: u64,
    count: u64,
    root: Node,

    bs: B,
}

#[derive(Debug, Eq, PartialEq)]
enum Item {
    Link(Cid),
    Ptr(Box<Node>),
}

#[derive(Debug, Eq, PartialEq, Default)]
pub struct Node {
    bitmap: usize,
    branches: RefCell<Vec<Item>>,
    leafs: Vec<Value>,
}

impl<B> Amt<B>
where
    B: Blocks,
{
    pub fn new(bs: B) -> Self {
        Amt::<B> {
            height: 0,
            count: 0,
            root: Node::new(),
            bs,
        }
    }

    pub fn load(cid: &Cid, bs: B) -> Result<Self> {
        let part_root: PartAmt = bs.get(cid)?;
        Ok(Self::from_part(part_root, bs))
    }

    fn from_part(part_root: PartAmt, bs: B) -> Self {
        Amt::<B> {
            height: part_root.0,
            count: part_root.1,
            root: part_root.2,
            bs,
        }
    }

    pub fn from_array<I: Serialize, L: AsRef<[I]>>(arr: L, bs: B) -> Result<Cid> {
        let mut root = Self::new(bs);
        root.batch_set(arr)?;
        root.flush()
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn set<Input: Serialize>(&mut self, k: u64, input: Input) -> Result<()> {
        let v: Value = ipld_cbor::struct_to_cbor_value(&input)?;

        // extend amt tree first
        // if current key large then tree capacity, create a new root, and move current root be
        // sub node of new root, so that the tree is been extended.
        let mut tmp = k >> (BITS_PER_SUBKEY * (self.height + 1));
        while tmp != 0 {
            if !self.root.is_empty() {
                self.root.flush(&mut self.bs, self.height)?;
                let cid = self.bs.put(&self.root)?;
                self.root = Node::new_with_cid(cid);
            }
            tmp >>= BITS_PER_SUBKEY;
            self.height += 1;
        }
        let current_shift = BITS_PER_SUBKEY * self.height;
        let add = self.root.set(&self.bs, self.height, k, v, current_shift)?;
        if add {
            self.count += 1;
        }
        Ok(())
    }

    pub fn batch_set<I: Serialize, L: AsRef<[I]>>(&mut self, vals: L) -> Result<()> {
        for (i, v) in vals.as_ref().iter().enumerate() {
            self.set(i as u64, v)?;
        }
        Ok(())
    }

    pub fn get<Output: DeserializeOwned>(&self, k: u64) -> Result<Output> {
        let test = k >> (BITS_PER_SUBKEY * (self.height + 1));
        if test != 0 {
            // not found
            return Err(AmtIpldError::NotFound(k));
        }
        let current_shift = BITS_PER_SUBKEY * self.height;
        let output = self
            .root
            .get(&self.bs, self.height, k, current_shift, |v| {
                ipld_cbor::cbor_value_to_struct(v.clone()).map_err(AmtIpldError::IpldCbor)
            })?;
        Ok(output)
    }

    pub fn delete(&mut self, k: u64) -> Result<()> {
        let current_shift = BITS_PER_SUBKEY * self.height;
        self.root.delete(&self.bs, self.height, k, current_shift)?;
        self.count -= 1;

        while self.root.bitmap == 1 && self.height > 0 {
            let sub = {
                let mut branches = self.root.branches.borrow_mut();
                branches[0].load_item(&self.bs)?;

                if let Item::Ptr(node) = &mut branches[0] {
                    // the `self.node.branches[;]` would be released with `self.node` releasing,
                    // and self.node would be replaced by `sub` later.
                    // thus, we use an `empty` node replace the `self.node.branches[i]`, passing to outside,
                    // and use the `empty` node replace with self.node.
                    // notice only allow in single thread
                    let mut empty = Node::new();
                    std::mem::swap(node.deref_mut(), &mut empty);
                    empty
                } else {
                    // TODO
                    unreachable!("")
                }
            };

            self.root = sub;
            self.height -= 1;
        }

        Ok(())
    }

    pub fn batch_delete(&mut self, keys: &[u64]) -> Result<()> {
        for k in keys.iter() {
            self.delete(*k)?;
        }
        Ok(())
    }

    pub fn flush(mut self) -> Result<Cid> {
        self.root.flush(&mut self.bs, self.height)?;
        let cid = self.bs.put(&self)?;
        Ok(cid)
    }
}

impl Item {
    fn new_ptr() -> Self {
        Item::Ptr(Box::new(Node::default()))
    }

    fn load_item<B>(&mut self, bs: &B) -> Result<()>
    where
        B: Blocks,
    {
        if let Item::Link(cid) = self {
            let n: Node = bs.get(cid)?;
            *self = Item::Ptr(Box::new(n));
        }
        Ok(())
    }
}

#[inline]
fn set_bit(bitmap: &mut usize, index: usize) {
    let b = 1 << index;
    *bitmap |= b;
}

#[inline]
fn unset_bit(bitmap: &mut usize, index: usize) {
    let b = 1 << index;
    *bitmap &= !b
}

impl Node {
    fn new() -> Self {
        Node::default()
    }

    fn new_with_cid(cid: Cid) -> Self {
        Node {
            bitmap: 1,
            branches: RefCell::new(vec![Item::Link(cid)]),
            leafs: vec![],
        }
    }

    fn new_from_raw(bitmap: usize, branches: Vec<Item>, leafs: Vec<Value>) -> Self {
        Node {
            bitmap,
            branches: RefCell::new(branches),
            leafs,
        }
    }

    #[inline]
    fn get_bit(&self, index: usize) -> bool {
        (1 << index) & self.bitmap != 0
    }

    fn bit_to_index(&self, bit_pos: usize) -> usize {
        let mask = (1 << bit_pos) - 1;
        (mask & self.bitmap).count_ones() as usize
    }

    fn is_empty(&self) -> bool {
        self.bitmap == 0
    }

    fn set<B>(&mut self, bs: &B, height: u64, key: u64, v: Value, shift: u64) -> Result<bool>
    where
        B: Blocks,
    {
        let pos = bit_pos(key, shift);
        let exist = self.get_bit(pos);
        let index = self.bit_to_index(pos);

        // it's leaf node
        if height == 0 {
            if exist {
                self.leafs[index] = v; // must success, or this tree is broken
            } else {
                set_bit(&mut self.bitmap, pos);
                self.leafs.insert(index, v);
            }
            return Ok(!exist);
        }

        let mut branches = self.branches.borrow_mut();

        if exist {
            branches[index].load_item(bs)?;
        } else {
            // create new branch
            branches.insert(index, Item::new_ptr());
            set_bit(&mut self.bitmap, pos);
        }

        if let Item::Ptr(node) = &mut branches[index] {
            node.set(bs, height - 1, key, v, shift - BITS_PER_SUBKEY)
        } else {
            // TODO
            unreachable!("")
        }
    }

    pub fn get<B, F, Output>(
        &self,
        bs: &B,
        height: u64,
        key: u64,
        shift: u64,
        f: F,
    ) -> Result<Output>
    where
        B: Blocks,
        F: Fn(&Value) -> Result<Output>,
    {
        let pos = bit_pos(key, shift);
        if !self.get_bit(pos) {
            return Err(AmtIpldError::NotFound(key));
        }
        let index = self.bit_to_index(pos);

        // touch leaf node, fetch value
        if height == 0 {
            let v_ref = self.leafs.get(index).expect("value list must match bitmap");
            return f(v_ref);
        }

        // load item
        self.branches.borrow_mut()[index].load_item(bs)?;

        let borrow = self.branches.borrow();
        let b = &borrow[index];
        if let Item::Ptr(node) = b {
            node.get(bs, height - 1, key, shift - BITS_PER_SUBKEY, f)
        } else {
            // TODO
            unreachable!("")
        }
    }

    pub fn delete<B>(&mut self, bs: &B, height: u64, key: u64, shift: u64) -> Result<()>
    where
        B: Blocks,
    {
        let pos = bit_pos(key, shift);
        if !self.get_bit(pos) {
            return Err(AmtIpldError::NotFound(key));
        }
        let index = self.bit_to_index(pos);

        if height == 0 {
            unset_bit(&mut self.bitmap, pos);
            self.leafs.remove(index);
            return Ok(());
        }

        let mut branches = self.branches.borrow_mut();
        branches[index].load_item(bs)?;

        if let Item::Ptr(node) = &mut branches[index] {
            node.delete(bs, height - 1, key, shift - BITS_PER_SUBKEY)?;

            if node.is_empty() {
                unset_bit(&mut self.bitmap, pos);
                branches.remove(index);
            }
            Ok(())
        } else {
            // TODO
            unreachable!("")
        }
    }

    pub fn flush<B>(&mut self, bs: &mut B, depth: u64) -> Result<()>
    where
        B: Blocks,
    {
        if depth == 0 {
            // do nothing for leaf
            return Ok(());
        }

        let mut branches = self.branches.borrow_mut();
        for b in branches.iter_mut() {
            b.load_item(bs)?;
            if let Item::Ptr(node) = b {
                node.flush(bs, depth - 1)?;
                let cid = bs.put(node)?;
                // refresh current branch link cid
                *b = Item::Link(cid)
            }
        }
        Ok(())
    }
}

#[cfg(test)]
pub fn create_root<B: Blocks>(height: u64, count: u64, node: Node, bs: B) -> Amt<B> {
    Amt {
        height,
        count,
        root: node,
        bs,
    }
}
