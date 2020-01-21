use cid::Cid;
use serde_cbor::Value;
use std::slice;

use crate::blocks::Blocks;
use crate::error::AmtIpldError;
use crate::{Node, Root, BITS_PER_SUBKEY, WIDTH};

impl<B> Root<B>
where
    B: Blocks,
{
    /// this function would use in anywhere to traverse the trie, do not need flush first.
    pub fn for_each<F>(&self, f: &mut F)
    where
        F: FnMut(u64, &Value),
    {
        traversing(self.bs.clone(), &self.node, self.height, 0, f);
    }
}

fn traversing<B, F>(bs: B, node: &Node, height: u64, prefix_key: u64, f: &mut F)
where
    B: Blocks,
    F: FnMut(u64, &Value),
{
    let prefix = prefix_key << BITS_PER_SUBKEY;
    for i in 0..WIDTH {
        let current_key = prefix + i as u64;
        if height == 0 {
            if node.get_bit(i) {
                let index = node.index_for_bitpos(i);
                if let Some(v) = node.values.get(index) {
                    f(current_key, v)
                } else {
                    unreachable!("bitmap not match value list, the tree is corrupted")
                }
            }
        } else {
            let _ = node.load_node(bs.clone(), i, |node| {
                traversing(bs.clone(), node, height - 1, current_key, f);
                Ok(())
            });
        }
    }
}

struct Iter<'a, B>
where
    B: Blocks,
{
    size: usize,
    count: usize,
    stack: Vec<Traversing<'a>>,

    bs: B,
}

#[derive(Clone)]
enum Traversing<'a> {
    Leaf(slice::Iter<'a, Value>),
    Link(&'a Node, usize),
}

//impl<'a, B> Iterator for Iter<'a, B>
//    where
//        B: Blocks,
//{
//    type Item = &'a Value;
//
//    fn next(&mut self) -> Option<Self::Item> {
//        let last = match self.stack.pop() {
//            Some(last) => last,
//            None => {
//                return None;
//            }
//        };
//        match last {
//            Traversing::Leaf(mut iter) => {
//                match iter.next() {
//                    Some(ref v) => {
//                        self.count += 1;
//                        self.stack.push(Traversing::Leaf(iter));
//                        Some(v)
//                    }
//                    None => {
//                        self.next()
//                    }
//                }
//            },
//            Traversing::Link(mut node, mut pos) => {
//                if pos == WIDTH {
//                    // current node cache has searched finish, do next
//                    return self.next()
//                }
//                if node.values.len() != 0 && node.bitmap != 0 {
//                    self.stack.push(Traversing::Leaf(node.values.iter()));
//                    self.next()
//                } else {
//                    // try get or load cache for pos, if cache not exist, push current node and plus pos
//                    let r = node.load_node(self.bs.clone(), pos, |node| {
//                        self.stack.push(Traversing::Link(node, 0));
//                        self.next();
//                        Ok(())
//                    });
//                    match r {
//                        Ok(_) => {
//                            self.next()
//                        }
//                        Err(AmtIpldError::NoNodeForIndex(_)) => {
//                            self.stack.push(Traversing::Link(node, pos + 1));
//                            self.next()
//                        }
//                        Err(_) => unreachable!("should not reach this branch, otherwise the tree is corrupted")
//                    }
//                }
//            }
//        }
//    }
//}
