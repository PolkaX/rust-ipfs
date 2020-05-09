mod common;

use block_format::{BasicBlock, Block};
use cid::{Cid, Codec};
use rust_ipld_format::coding::{decode, register};
use rust_ipld_format::{Node, Result};

use self::common::EmptyNode;

// coding
fn init() {
    register(Codec::Raw, |_block| {
        let node = EmptyNode::new();
        Ok(Box::new(node))
    });
}

fn decode_fu(_block: &dyn Block) -> Result<Box<dyn Node>> {
    let node = EmptyNode::new();
    Ok(Box::new(node))
}

fn init2() {
    register(Codec::Raw, decode_fu);
}

#[test]
fn test_decode() {
    init();
    let id = Cid::new_v1(Codec::Raw, multihash::Identity::digest(b""));
    let block = BasicBlock::new_with_cid(vec![].into(), id.clone()).unwrap();
    let node = decode(&block).unwrap();
    assert_eq!(node.cid(), &id);
}

#[test]
fn test_init() {
    init2();
}
