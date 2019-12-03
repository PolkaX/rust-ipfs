use super::*;
use crate::coding::{decode, register};

// coding
fn init() {
    register(Codec::Raw, |b| {
        let node = EmptyNode::new();
        Ok(Box::new(node))
    });
}

fn decode_fu(block: &dyn Block) -> Result<Box<dyn Node>> {
    let node = EmptyNode::new();
    Ok(Box::new(node))
}

fn init2() {
    register(Codec::Raw, decode_fu);
}

#[test]
fn test_decode() {
    init();
    let p = Prefix {
        version: Version::V1,
        codec: Codec::Raw,
        mh_type: MHashEnum::Identity,
        mh_len: 0,
    };
    let id = p.sum(b"").unwrap();
    let block = BasicBlock::new_with_cid(vec![].into(), id.clone()).unwrap();
    let node = decode(&block).unwrap();
    assert_eq!(node.cid(), &id);
}

#[test]
fn test_init() {
    init2();
}
