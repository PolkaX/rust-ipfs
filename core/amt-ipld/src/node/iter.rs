// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.
use std::vec;

use serde_cbor::Value;

use super::{Amt, Item, Node, BITS_PER_SUBKEY, WIDTH};
use crate::blocks::Blocks;
use crate::error::*;

impl<B> Amt<B>
where
    B: Blocks,
{
    /// this function would use in anywhere to traverse the trie, do not need flush first.
    pub fn for_each<F>(&self, f: &mut F) -> Result<()>
    where
        F: FnMut(u64, &Value) -> Result<()>,
    {
        traversing(&self.bs, &self.root, self.height, 0, f)
    }

    /// Subtract removes all elements of 'or' from 'self'
    pub fn subtract(&mut self, or: Self) -> Result<()> {
        or.for_each(&mut |key, _| {
            // if not find, do not handle error
            match self.delete(key) {
                Ok(_) | Err(AmtIpldError::NotFound(_)) => Ok(()),
                Err(e) => Err(e),
            }
        })
    }
}

fn traversing<B, F>(bs: &B, node: &Node, height: u64, prefix_key: u64, f: &mut F) -> Result<()>
where
    B: Blocks,
    F: FnMut(u64, &Value) -> Result<()>,
{
    let prefix = prefix_key << BITS_PER_SUBKEY;
    if height == 0 {
        for i in 0..WIDTH {
            if node.get_bit(i) {
                let current_key = prefix + i as u64;
                let index = node.bit_to_index(i);
                f(current_key, &node.leafs[index])?;
            }
        }
        return Ok(());
    }

    let mut branches = node.branches.borrow_mut();

    for i in 0..WIDTH {
        if node.get_bit(i) {
            let current_key = prefix + i as u64;
            let index = node.bit_to_index(i);
            branches[index].load_item(bs)?;

            if let Item::Ptr(node) = &branches[index] {
                traversing(bs, node, height - 1, current_key, f)?;
            } else {
                unreachable!("after `load_item`, Item must be `Ptr`")
            }
        }
    }
    Ok(())
}

impl<B> Amt<B>
where
    B: Blocks,
{
    /// `iter()` is equal to `for_each` now(could use before `flush()`).
    /// but `iter()` would cast more resource
    pub fn iter(&self) -> Iter<B> {
        let node_ref = &self.root;

        let prefix_key_list = (0..WIDTH)
            .map(|prefix_key| (prefix_key, node_ref.get_bit(prefix_key)))
            .filter(|(_, need)| *need)
            .map(|(pref, _)| pref as u64)
            .collect::<Vec<_>>();

        let init_node = &self.root as *const Node;

        let init = if self.height == 0 {
            Traversing::Leaf(0, (prefix_key_list, init_node))
        } else {
            Traversing::Branch(prefix_key_list, init_node)
        };
        Iter {
            size: self.count,
            count: 0,
            stack: vec![init],
            bs: &self.bs,
        }
    }
}

/// this `Iter` only could be used for FlushedRoot, due to current module use child_cache to store child,
/// and the child ref is under `RefCell`. So that we could only iterate the tree after flushing.
/// if someone do not what iterating the free after flushing, could use `for_each`.
pub struct Iter<'a, B>
where
    B: Blocks,
{
    size: u64,
    count: u64,
    stack: Vec<Traversing>,
    // blocks ref, use for load node from cid
    bs: &'a B,
}

enum Traversing {
    Leaf(usize, (Vec<u64>, *const Node)),
    Branch(Vec<u64>, *const Node),
}

impl<'a, B> Iterator for Iter<'a, B>
where
    B: Blocks,
{
    type Item = (u64, &'a Value);

    /// it's safe to use unsafe here, for except root node, every node is in heap,
    /// and be refered from root node. thus we use unsafe to avoid lifetime check
    /// and mutable check.
    /// notice iterator would load node for cid, thus after iter, all tree is in
    /// `Ptr` mode, in other word, is being expanded
    fn next(&mut self) -> Option<Self::Item> {
        let last = match self.stack.pop() {
            Some(last) => last,
            None => {
                return None;
            }
        };
        match last {
            Traversing::Leaf(pos, (keys, leaf_node)) => {
                let r = unsafe { (*leaf_node).leafs.get(pos).map(|v| (keys[pos], v)) };
                match r {
                    Some(v) => {
                        let pos = pos + 1;
                        self.stack.push(Traversing::Leaf(pos, (keys, leaf_node)));
                        self.count += 1;
                        Some(v)
                    }
                    None => self.next(),
                }
            }
            Traversing::Branch(keys, node_ref) => {
                unsafe {
                    let node = &(*node_ref);
                    let mut children = vec![];
                    for (b, key) in node.branches.borrow_mut().iter_mut().zip(keys.into_iter()) {
                        b.load_item(self.bs).ok()?;
                        if let Item::Ptr(child_node) = b {
                            let prefix_key_list = (0..WIDTH)
                                .map(|prefix_key| (prefix_key, child_node.get_bit(prefix_key)))
                                .filter(|(_, need)| *need)
                                .map(|(pref, _)| (key << BITS_PER_SUBKEY) + pref as u64)
                                .collect::<Vec<_>>();

                            let node_ptr = child_node.as_ref() as *const Node;
                            if !child_node.leafs.is_empty() {
                                children.push(Traversing::Leaf(0, (prefix_key_list, node_ptr)));
                            } else {
                                children.push(Traversing::Branch(prefix_key_list, node_ptr));
                            }
                        }
                    }
                    self.stack.extend(children.into_iter().rev());
                }
                self.next()
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let hint = (self.size - self.count) as usize;
        (hint, Some(hint))
    }
}
