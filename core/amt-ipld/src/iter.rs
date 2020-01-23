use cid::Cid;
use serde_cbor::Value;
use std::iter::Zip;
use std::vec;

use crate::blocks::Blocks;
use crate::error::*;
use crate::{FlushedRoot, Node, Root, BITS_PER_SUBKEY, WIDTH};

impl<B> Root<B>
where
    B: Blocks,
{
    /// this function would use in anywhere to traverse the trie, do not need flush first.
    pub fn for_each<F>(&self, f: &mut F) -> Result<()>
    where
        F: FnMut(u64, &Value) -> Result<()>,
    {
        traversing(self.bs.clone(), &self.node, self.height, 0, f)
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

fn traversing<B, F>(bs: B, node: &Node, height: u64, prefix_key: u64, f: &mut F) -> Result<()>
where
    B: Blocks,
    F: FnMut(u64, &Value) -> Result<()>,
{
    let prefix = prefix_key << BITS_PER_SUBKEY;
    for i in 0..WIDTH {
        let current_key = prefix + i as u64;
        if height == 0 {
            if node.get_bit(i) {
                let index = node.index_for_bitpos(i);
                if let Some(v) = node.values.get(index) {
                    f(current_key, v)?;
                } else {
                    unreachable!("bitmap not match value list, the tree is corrupted")
                }
            }
        } else {
            let r = node.load_node(bs.clone(), i, |node| {
                traversing(bs.clone(), node, height - 1, current_key, f)
            });
            match r {
                Ok(_) | Err(AmtIpldError::NoNodeForIndex(_)) => {}
                Err(e) => return Err(e),
            }
        }
    }
    Ok(())
}

impl<B> FlushedRoot<B>
where
    B: Blocks,
{
    /// for `FlushedRoot`, the root must be a flushed tree, thus could load node from cid directly
    pub fn iter(&self) -> Iter<B> {
        let node_ref = &self.root.node;

        let prefix_key_list = (0..WIDTH)
            .map(|prefix_key| (prefix_key, node_ref.get_bit(prefix_key)))
            .filter(|(_, need)| *need)
            .map(|(pref, _)| pref as u64)
            .collect::<Vec<_>>();

        let init = if self.root.height == 0 {
            assert_eq!(prefix_key_list.len(), node_ref.values.len());
            let zip = prefix_key_list
                .into_iter()
                .zip(node_ref.values.clone().into_iter());
            Traversing::Leaf(zip)
        } else {
            assert_eq!(prefix_key_list.len(), node_ref.links.len());
            let zip = prefix_key_list
                .into_iter()
                .zip(node_ref.links.clone().into_iter());
            Traversing::Link(zip)
        };
        Iter {
            size: self.root.count,
            count: 0,
            stack: vec![init],
            bs: self.root.bs.clone(),
        }
    }
}

/// this `Iter` only could be used for FlushedRoot, due to current module use child_cache to store child,
/// and the child ref is under `RefCell`. So that we could only iterate the tree after flushing.
/// if someone do not what iterating the free after flushing, could use `for_each`.
pub struct Iter<B>
where
    B: Blocks,
{
    size: u64,
    count: u64,
    stack: Vec<Traversing>,
    // blocks ref, use for load node from cid
    bs: B,
}

#[derive(Clone)]
enum Traversing {
    Leaf(Zip<vec::IntoIter<u64>, vec::IntoIter<Value>>),
    Link(Zip<vec::IntoIter<u64>, vec::IntoIter<Cid>>),
}

impl<B> Iterator for Iter<B>
where
    B: Blocks,
{
    type Item = (u64, Value);

    fn next(&mut self) -> Option<Self::Item> {
        let last = match self.stack.last_mut() {
            Some(last) => last,
            None => {
                return None;
            }
        };
        match last {
            Traversing::Leaf(ref mut iter) => match iter.next() {
                Some(v) => {
                    self.count += 1;
                    Some(v)
                }
                None => {
                    self.stack.pop();
                    self.next()
                }
            },
            Traversing::Link(ref mut iter) => match iter.next() {
                Some((key, cid)) => {
                    let n: Node = self.bs.get(&cid).ok()?;

                    let prefix_key_list = (0..WIDTH)
                        .map(|prefix_key| (prefix_key, n.get_bit(prefix_key)))
                        .filter(|(_, need)| *need)
                        .map(|(pref, _)| (key << BITS_PER_SUBKEY) + pref as u64)
                        .collect::<Vec<_>>();

                    if n.bitmap != 0 && !n.values.is_empty() {
                        let zip = prefix_key_list.into_iter().zip(n.values.into_iter());
                        self.stack.push(Traversing::Leaf(zip));
                    } else {
                        let zip = prefix_key_list.into_iter().zip(n.links.into_iter());
                        self.stack.push(Traversing::Link(zip));
                    }
                    self.next()
                }
                None => {
                    self.stack.pop();
                    self.next()
                }
            },
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            (self.size - self.count) as usize,
            Some((self.size - self.count) as usize),
        )
    }
}
