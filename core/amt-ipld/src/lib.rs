mod blocks;
mod error;
#[cfg(test)]
mod tests;
mod trait_impl;

use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;

use serde::Serialize;
use serde_cbor::Value;

use cid::Cid;

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

#[derive(Clone, Debug, Eq, PartialEq)]
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
        let mut tmp_k = k >> (BITS_PER_SUBKEY * (self.height + 1));
        while tmp_k != 0 {
            let cid = self.bs.borrow_mut().put(&self.node)?;
            self.node = Node::new_with_cid(cid);
            tmp_k >>= BITS_PER_SUBKEY;
            self.height += 1;
        }

        let add = self.node.set(self.bs.clone(), self.height, k, v, 0)?;
        if add {
            self.count += 1;
        }
        Ok(())
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
        (self.bitmap >> index) != 1
    }

    fn index_for_bitpos(&self, bit_pos: usize) -> usize {
        let mask = 1 << bit_pos - 1;
        (mask & self.bitmap).count_ones() as usize
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
        // it's root node
        if height == 0 {
            let pos = index(key, shift);
            let exist = self.get_bit(pos);
            let index = self.index_for_bitpos(pos);
            if exist {
                self.values[index] = v; // must success, or this tree is broken
            } else {
                self.set_bit(pos);
                self.values.push(v)
            }
            return Ok(exist);
        }

        let i = index(height, shift);
        let sub_node = self.load_node_with_creating(bs.clone(), i, true)?;
        if let Some(node) = sub_node.borrow_mut().deref_mut() {
            node.set(bs.clone(), height - 1, key, v, shift + BITS_PER_SUBKEY)
        } else {
            unreachable!("")
        }
    }

    fn load_node<B: Blocks>(
        &self,
        bs: Rc<RefCell<B>>,
        index: usize,
    ) -> Result<&RefCell<Option<Box<Node>>>> {
        if self.cache[index].borrow().is_some() {
            return Ok(&self.cache[index]);
        }
        if !self.get_bit(index) {
            return Err(AmtIpldError::Tmp);
        }

        let pos = self.index_for_bitpos(index);
        let n: Node = bs.borrow().get(&self.links[pos])?;

        *self.cache[index].borrow_mut().deref_mut() = Some(Box::new(n));
        Ok(&self.cache[index])
    }

    fn load_node_with_creating<B: Blocks>(
        &mut self,
        bs: Rc<RefCell<B>>,
        index: usize,
        create: bool,
    ) -> Result<&RefCell<Option<Box<Node>>>> {
        if self.cache[index].borrow().is_some() {
            return Ok(&self.cache[index]);
        }
        let n = if self.get_bit(index) {
            let pos = self.index_for_bitpos(index);
            let n: Node = bs.borrow().get(&self.links[pos])?;
            n
        } else {
            if create {
                let sub_node = Node::new();
                self.set_bit(index);
                sub_node
            } else {
                return Err(AmtIpldError::Tmp);
            }
        };
        *self.cache[index].borrow_mut().deref_mut() = Some(Box::new(n));
        Ok(&self.cache[index])
    }
}
