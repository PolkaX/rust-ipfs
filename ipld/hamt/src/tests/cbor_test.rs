// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use super::*;

use bytes::Bytes;
use cid::Cid;
use ipld_cbor::struct_to_cbor_value;

#[test]
fn test_kv() {
    let b: Bytes = vec![1_u8, 2, 3].into();
    let v = struct_to_cbor_value(&b).unwrap();
    let kv: KVT = ("123".to_string(), v);

    let r = serde_cbor::to_vec(&kv).unwrap();
    println!("{:?}", r);

    let result = vec![130_u8, 99, 49, 50, 51, 67, 1, 2, 3];
    assert_eq!(r, result);

    let kv2: KVT = serde_cbor::from_slice(&r).unwrap();
    assert_eq!(kv, kv2);
}

#[test]
fn test_pointer_and_node() {
    let b: Bytes = vec![1_u8, 2, 3].into();
    let v = struct_to_cbor_value(&b).unwrap();
    let kv: KVT = ("123".to_string(), v.clone());
    let kv2: KVT = ("124".to_string(), v);
    let pointer = Item::from_kvs(vec![kv, kv2]);
    let r = serde_cbor::to_vec(&pointer).unwrap();
    println!("{:?}", r);
    assert_eq!(
        r,
        vec![161, 97, 49, 130, 130, 99, 49, 50, 51, 67, 1, 2, 3, 130, 99, 49, 50, 52, 67, 1, 2, 3]
    );

    let p2: Item = serde_cbor::from_slice(&r).unwrap();
    assert_eq!(p2, pointer);

    let cid = Cid::new_v0(multihash::Sha2_256::digest(b"something")).unwrap();
    let pointer2 = Item::from_link(cid);
    let r = serde_cbor::to_vec(&pointer2).unwrap();
    println!("{:?}", r);
    assert_eq!(
        r,
        vec![
            161, 97, 48, 216, 42, 88, 35, 0, 18, 32, 63, 201, 182, 137, 69, 157, 115, 143, 140,
            136, 163, 164, 138, 169, 227, 53, 66, 1, 107, 122, 64, 82, 224, 1, 170, 165, 54, 252,
            167, 72, 19, 203
        ]
    );

    // bitfield is 0
    let node = test_node("0", vec![pointer.clone(), pointer2.clone()]);
    let r = serde_cbor::to_vec(&node).unwrap();
    println!("{:?}", r);
    assert_eq!(
        r,
        vec![
            130, 64, 130, 161, 97, 49, 130, 130, 99, 49, 50, 51, 67, 1, 2, 3, 130, 99, 49, 50, 52,
            67, 1, 2, 3, 161, 97, 48, 216, 42, 88, 35, 0, 18, 32, 63, 201, 182, 137, 69, 157, 115,
            143, 140, 136, 163, 164, 138, 169, 227, 53, 66, 1, 107, 122, 64, 82, 224, 1, 170, 165,
            54, 252, 167, 72, 19, 203
        ]
    );
    let node: Node = serde_cbor::from_slice(&r).unwrap();
    println!("{:?}", node);

    // bitfield is 9999
    let node = test_node("9999", vec![pointer.clone(), pointer2.clone()]);
    let r = serde_cbor::to_vec(&node).unwrap();
    println!("{:?}", r);
    assert_eq!(
        r,
        vec![
            130, 66, 39, 15, 130, 161, 97, 49, 130, 130, 99, 49, 50, 51, 67, 1, 2, 3, 130, 99, 49,
            50, 52, 67, 1, 2, 3, 161, 97, 48, 216, 42, 88, 35, 0, 18, 32, 63, 201, 182, 137, 69,
            157, 115, 143, 140, 136, 163, 164, 138, 169, 227, 53, 66, 1, 107, 122, 64, 82, 224, 1,
            170, 165, 54, 252, 167, 72, 19, 203
        ]
    );
    let node: Node = serde_cbor::from_slice(&r).unwrap();
    println!("{:?}", node);

    // bitfield is 0x12345678
    let node = test_node("305419896", vec![pointer.clone(), pointer2.clone()]);
    let r = serde_cbor::to_vec(&node).unwrap();
    println!("{:?}", r);
    assert_eq!(
        r,
        vec![
            130, 68, 18, 52, 86, 120, 130, 161, 97, 49, 130, 130, 99, 49, 50, 51, 67, 1, 2, 3, 130,
            99, 49, 50, 52, 67, 1, 2, 3, 161, 97, 48, 216, 42, 88, 35, 0, 18, 32, 63, 201, 182,
            137, 69, 157, 115, 143, 140, 136, 163, 164, 138, 169, 227, 53, 66, 1, 107, 122, 64, 82,
            224, 1, 170, 165, 54, 252, 167, 72, 19, 203
        ]
    );
    let node: Node = serde_cbor::from_slice(&r).unwrap();
    println!("{:?}", node);

    let node = test_node(
        "11579208923731619542357098500868790785326998466564056403945758400791312",
        vec![pointer, pointer2],
    );
    let r = serde_cbor::to_vec(&node).unwrap();
    let node: Node = serde_cbor::from_slice(&r).unwrap();
    println!("{:?}", node);
}
