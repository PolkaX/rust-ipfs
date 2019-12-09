// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use rust_block_format::{BasicBlock, Block};
use rust_cid::Hash;

#[test]
fn test_blocks_basic() {
    // would not panic
    let empty = vec![];
    BasicBlock::new(empty.into());

    BasicBlock::new(b"Hello world!".as_ref().into());
}

#[test]
fn test_data() {
    let data = b"some data";
    let block = BasicBlock::new(data.as_ref().into());

    assert_eq!(block.raw_data().as_ref(), data.as_ref())
}

#[test]
fn test_hash() {
    let data = b"some other data";
    let block = BasicBlock::new(data.as_ref().into());

    let hash = multihash::encode(Hash::SHA2256, data.as_ref()).unwrap();

    assert_eq!(block.multihash(), hash);
}

#[test]
fn test_cid() {
    let data = b"yet another data";
    let block = BasicBlock::new(data.as_ref().into());
    let cid_ref = block.cid();

    assert_eq!(block.multihash(), cid_ref.multihash());
}
