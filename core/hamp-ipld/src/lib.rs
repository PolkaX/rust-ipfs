mod error;
mod ipld;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

use crate::ipld::{Blocks, CborIpldStor};
use cid::Cid;
use ipld_cbor::CborBigUint;

const DEFAULT_BIT_WIDTH: usize = 8;

//#[derive(Serialize, Deserialize, Debug)]
//pub struct Node<B: Blocks> {
//    bitfield: CborBigUint,
//    pointers: Vec<Pointer>,
//
//    store: CborIpldStor<B>,
//    bit_width: usize,
//}

//impl<B: Blocks> Node<B> {
//    pub fn new<F>(cs: CborIpldStor<B>, options: &[F]) -> Node<B>
//    where
//        F: Fn(&Node<B>),
//    {
//        let nd = Node {
//            bitfield: CborBigUint(1_u64.into()),
//            pointers: vec![],
//            store: cs,
//            bit_width: DEFAULT_BIT_WIDTH,
//        };
//        for f in options {
//            f(&nd);
//        }
//        nd
//    }
//
////    pub fn find(k: &str) -> Result<()> {
//
//    }
//
//    fn get_value()
//}

#[derive(Debug)]
pub struct KV {
    key: String,
    value: Vec<u8>,
}

#[derive(Debug)]
pub struct Pointer {
    kvs: Vec<KV>,
    link: Cid,
}
