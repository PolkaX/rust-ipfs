pub mod entry;
mod trait_impl;

use archery::{RcK, SharedPointer, SharedPointerKind};

use crate::error::*;
use crate::hash::{hash, HashBits};

use self::entry::{PContent, Pointer, KV};

const ARRAY_WIDTH: usize = 3;
const DEFAULT_BIT_WIDTH: usize = 8;

pub type NodeP<P: SharedPointerKind> = SharedPointer<Node<P>, P>;

#[derive(Debug)]
pub struct Node<P = RcK>
where
    P: SharedPointerKind,
{
    // we use u64 here, for normally a branch of node would not over 64, 64 branch's wide is so large, if larger then 64, panic
    bitfield: u64,
    pointers: Vec<Pointer<P>>,

    /// for fetching and storing children
    //    store: &'a mut CborIpldStor<B>,
    bit_width: u32,
}

#[inline]
fn bit(input: u64, n: u32) -> u64 {
    input & (1 << n as u64)
}

#[inline]
fn set_bit(input: &mut u64, n: u32) {
    *input |= 1 << n as u64
}

#[inline]
fn unset_bit(input: &mut u64, n: u32) {
    *input &= !(1 << n as u64)
}

/// index for bit position in this bitmap
#[inline]
pub fn index_for_bitpos(bitmap: u64, bit_pos: u32) -> u32 {
    let mask = (1_u64 << bit_pos as u64) - 1;
    (bitmap & mask).count_ones()
}

impl<P> Node<P>
where
    P: SharedPointerKind,
{
    #[cfg(test)]
    pub fn test_init(bitfield: u64, pointers: Vec<Pointer<P>>, bit_width: u32) -> Self {
        Node {
            bitfield,
            pointers,
            bit_width,
        }
    }

    fn modify_value<'hash>(
        &mut self,
        hv: &mut HashBits<'hash>,
        k: &str,
        v: Option<Vec<u8>>,
    ) -> Result<()> {
        // TODO
        let idx = hv.next(self.bit_width).ok_or(Error::Tmp)?;
        // bitmap do not have this bit, it's a new key for this bit position.
        if bit(self.bitfield, idx) != 1 {
            return self.insert_child(idx, k, v);
        }

        let cindex = index_for_bitpos(self.bitfield, idx);
        let child = self.pointers.get_mut(cindex as usize).ok_or(Error::Tmp)?; // todo

        match child.data {
            PContent::Link(_) => {
                let mut child_node_p = child.load_child(self.bit_width)?;

                let need_delete = v.is_none();
                SharedPointer::make_mut(&mut child_node_p).modify_value(hv, k, v)?;

                if need_delete {
                    return self.clean_child(child_node_p, cindex);
                }
                Ok(())
            }
            PContent::KVs(ref mut kvs) => {
                // when need to remove this key/value pair
                if v.is_none() {
                    let old_len = kvs.len();
                    // remove pair when key equal to k
                    kvs.retain(|entry| entry.key != k);

                    let result = if kvs.len() == 0 {
                        // no pair left, remove this child node
                        self.remove_child(idx)
                    } else if old_len == kvs.len() {
                        // no pair could be removed
                        Err(Error::Tmp)
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
                    let mut sub = Node::<P> {
                        bitfield: 0,
                        pointers: vec![],
                        bit_width: self.bit_width,
                    };
                    let mut hash_copy = hv.clone();
                    sub.modify_value(&mut hash_copy, k, v)?;

                    for p in kvs.iter() {
                        let new_hash = hash(p.key.as_bytes());
                        let mut ch_hv =
                            HashBits::new_with_consumed(new_hash.as_ref(), hv.consumed());
                        sub.modify_value(&mut ch_hv, p.key.as_str(), Some(p.value.to_vec()))?;
                    }

                    // TODO
                    // let c = self.store.put(sub)
                    // let pointer = Pointer::from_link(c);
                    // return self.set_child(cindex, pointer)
                    return Ok(());
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
    fn insert_child(&mut self, idx: u32, k: &str, v: Option<Vec<u8>>) -> Result<()> {
        // in insert, the value must exist, `None` represent delete this key.
        let v = v.ok_or(Error::Tmp)?; // todo

        let i = index_for_bitpos(self.bitfield, idx);
        // set bit for index i
        set_bit(&mut self.bitfield, i);

        // net pointer
        let p = Pointer::from_kvs(vec![KV::new(k.to_string(), v)]);
        self.pointers.push(p);
        Ok(())
    }

    fn clean_child(&mut self, child_node: NodeP<P>, idx: u32) -> Result<()> {
        let len = child_node.pointers.len();
        match len {
            0 => {
                Err(Error::Tmp) // TODO "incorrectly formed HAMT"
            }
            1 => {
                // TODO: only do this if its a value, cant do this for shards unless pairs requirements are met.
                let p = child_node.pointers.get(0).ok_or(Error::Tmp)?; // TODO
                if let PContent::Link(ref _cid) = p.data {
                    // don't know why... todo
                    return Ok(());
                }

                self.set_child(idx, (*p).clone())
            }
            x if x <= ARRAY_WIDTH => {
                let mut chvals = vec![];
                for p in child_node.pointers.iter() {
                    match p.data {
                        PContent::Link(_) => return Ok(()),
                        PContent::KVs(ref kvs) => {
                            for sp in kvs {
                                if chvals.len() == ARRAY_WIDTH {
                                    return Ok(());
                                }
                                chvals.push(sp.clone());
                            }
                        }
                    }
                }
                self.set_child(idx, Pointer::from_kvs(chvals))
            }
            _ => Ok(()),
        }
    }

    fn remove_child(&mut self, idx: u32) -> Result<()> {
        if idx as usize >= self.pointers.len() {
            return Err(Error::Tmp); // TODO
        }
        self.pointers.remove(idx as usize);
        // set idx pos bit is zero
        unset_bit(&mut self.bitfield, idx);
        Ok(())
    }

    fn set_child(&mut self, idx: u32, p: Pointer<P>) -> Result<()> {
        let v = self.pointers.get_mut(idx as usize).ok_or(Error::Tmp)?;
        *v = p;
        Ok(())
    }
}
