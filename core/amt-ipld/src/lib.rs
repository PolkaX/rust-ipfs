mod blocks;
mod error;
#[cfg(test)]
mod tests;
mod trait_impl;

use std::cell::RefCell;
use std::ops::{Deref, DerefMut};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_cbor::Value;

use cid::{zero_cid, Cid};

use crate::blocks::Blocks;
use crate::error::*;
use crate::internal::RefCellTrick;
use crate::trait_impl::PartRoot;
use std::borrow::{Borrow, BorrowMut};
use std::fmt::Debug;

const WIDTH: usize = 8; // 2^3
const BITS_PER_SUBKEY: u64 = 3;
const SUBKEY_MASK: u64 = 8 - 1; // 2^3 - 1

fn index(h: u64, s: u64) -> usize {
    ((h >> s) & SUBKEY_MASK) as usize
}

fn mask(h: u64, s: u64) -> usize {
    1 << index(h, s)
}

pub struct Root<B, NodeRef>
where
    B: Blocks,
    NodeRef: NodeRefLike<Target=Node<NodeRef>> + Debug,
{
    height: u64,
    count: u64,
    node: Node<NodeRef>,

    bs: B,
}

#[derive(Clone, Debug)]
pub struct Node<NodeRef: NodeRefLike<Target=Node<NodeRef>> + Debug> {
    bitmap: usize,
    links: Vec<Cid>,
    values: Vec<Value>,

    // cache
    cache: [NodeRef; WIDTH],
}

pub trait NodeRefLike: Debug + Default {
    type Target;
    fn load_node<B, F, R>(
        &self,
        parent: &Self::Target,
        index: usize,
        bs: Rc<RefCell<B>>,
        f: F,
    ) -> Result<R>
    where
        B: Blocks,
        F: Fn(&Self::Target) -> Result<R>,
        Self: Sized;

    fn load_node_with_creating<B, F, R>(
        &mut self,
        parent: &mut Self::Target,
        index: usize,
        bs: Rc<RefCell<B>>,
        create: bool,
        f: F,
    ) -> Result<R>
    where
        B: Blocks,
        F: FnMut(&mut Self::Target) -> Result<R>,
        Self: Sized;
}

pub mod internal {
    use super::Node;
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub struct RefCellTrick(pub RefCell<Option<Box<Node<RefCellTrick>>>>);
    // TODO RwLockTrick
}

impl NodeRefLike for internal::RefCellTrick {
    type Target = Node<internal::RefCellTrick>;

    fn load_node<B, F, R>(
        &self,
        parent: &Self::Target,
        index: usize,
        bs: Rc<RefCell<B>>,
        mut f: F,
    ) -> Result<R>
    where
        B: Blocks,
        F: Fn(&Self::Target) -> Result<R>,
    {
        if let Some(node) = self.0.borrow().deref() {
            return f(node);
        }
        if !parent.get_bit(index) {
            return Err(AmtIpldError::Tmp);
        }
        let pos = parent.index_for_bitpos(index);
        let n: Node<Self> = bs.deref().borrow().get(&parent.links[pos])?;
        let r = f(&n);
        *self.0.borrow_mut().deref_mut() = Some(Box::new(n));
        r
    }
    
    fn load_node_with_creating<B, F, R>(
        &mut self,
        parent: &mut Self::Target,
        index: usize,
        bs: Rc<RefCell<B>>,
        create: bool,
        mut f: F,
    ) -> Result<R>
    where
        B: Blocks,
        F: FnMut(&mut Self::Target) -> Result<R>,
    {
        if let Some(node) = self.0.borrow_mut().deref_mut() {
            return f(node);
        }
        let mut n = if parent.get_bit(index) {
            let pos = parent.index_for_bitpos(index);
            let n: Node<Self> = bs.deref().borrow().get(&parent.links[pos])?;
            n
        } else {
            if create {
                let sub_node = Node::new();
                parent.set_bit(index);
                sub_node
            } else {
                return Err(AmtIpldError::Tmp);
            }
        };
        let r = f(&mut n);
        *self.0.borrow_mut().deref_mut() = Some(Box::new(n));
        r
    }
}

/// for thread unsafe
pub type RefCellNode = Node<internal::RefCellTrick>;
pub type RefCellRoot<B> = Root<B, internal::RefCellTrick>;

#[cfg(test)]
pub fn create_node(bitmap: usize, links: Vec<Cid>, values: Vec<Value>) -> RefCellNode {
    Node {
        bitmap,
        links,
        values,
        cache: Default::default(),
    }
}

#[cfg(test)]
pub fn create_root<B: Blocks>(
    height: u64,
    count: u64,
    node: RefCellNode,
    bs: Rc<RefCell<B>>,
) -> RefCellRoot<B> {
    Root {
        height,
        count,
        node,
        bs,
    }
}

impl<B, NodeRef> Root<B, NodeRef>
where
    B: Blocks,
    NodeRef: NodeRefLike<Target=Node<NodeRef>> + Debug,
{
    pub fn new(bs: B) -> Self {
        Root::<B, NodeRef> {
            height: 0,
            count: 0,
            node: Node::<NodeRef>::new(),
            bs: Rc::new(RefCell::new(bs)),
        }
    }

    pub fn from_partroot(part_root: PartRoot<NodeRef>, bs: Rc<RefCell<B>>) -> Self {
        Root::<B, NodeRef> {
            height: part_root.0,
            count: part_root.1,
            node: part_root.2,
            bs,
        }
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn set<Input: Serialize>(&mut self, k: u64, input: Input) -> Result<()> {
        let v: Value = ipld_cbor::struct_to_cbor_value(&input)?;

        // extend amt tree first
        // if current key large then tree capacity, create a new root, and move current root be
        // sub node of new root, so that the tree is been extended.
        let mut tmp_k = k >> (BITS_PER_SUBKEY * (self.height + 1));
        while tmp_k != 0 {
            let cid = self.bs.deref().borrow_mut().put(&self.node)?;
            self.node = Node::new_with_cid(cid);
            tmp_k >>= BITS_PER_SUBKEY;
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
        let cid = self.bs.put(&self)?;
        Ok(cid)
    }
}

impl<NodeRef> Node<NodeRef>
where
    NodeRef: NodeRefLike<Target=Self> + Debug,
{
    pub fn new() -> Self {
        Node::<NodeRef> {
            bitmap: 0,
            links: vec![],
            values: vec![],
            cache: Default::default(),
        }
    }

    pub fn new_with_cid(cid: Cid) -> Self {
        Node::<NodeRef> {
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

    pub fn set<B>(&mut self, bs: B, height: u64, key: u64, v: Value, shift: u64) -> Result<bool>
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

        let i = index(height, shift);
        self.cache[i].load_node_with_creating(self, i, bs.clone(), true, |node| {
            node.set(bs.clone(), height - 1, key, v, shift + BITS_PER_SUBKEY)
        })
        //        let sub_node = self.load_node_with_creating(bs.clone(), i, true)?;
        //        if let Some(node) = sub_node.borrow_mut().deref_mut() {
        //            node.set(bs.clone(), height - 1, key, v, shift + BITS_PER_SUBKEY)
        //        } else {
        //            unreachable!("")
        //        }
        //        Ok(true)
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
                let sub_node = Node {
                    bitmap: 0,
                    links: vec![],
                    values: vec![],
                    cache: Default::default(),
                };
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
