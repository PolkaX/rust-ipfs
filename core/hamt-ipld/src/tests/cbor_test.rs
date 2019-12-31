use super::*;

use archery::RcK;
use cid::Cid;

#[test]
fn test_kv() {
    let kv = KV::new("123".to_string(), vec![1_u8, 2, 3]);

    let r = serde_cbor::to_vec(&kv).unwrap();
    println!("{:?}", r);

    let result = vec![130_u8, 99, 49, 50, 51, 67, 1, 2, 3];
    assert_eq!(r, result);

    let kv2: KV = serde_cbor::from_slice(&r).unwrap();
    assert_eq!(kv, kv2);
}

#[test]
fn test_pointer_and_node() {
    let kv = KV::new("123".to_string(), vec![1_u8, 2, 3]);
    let pointer = Pointer::from_kvs(vec![kv.clone(), kv]);
    let r = serde_cbor::to_vec(&pointer).unwrap();
    println!("{:?}", r);
    assert_eq!(
        r,
        vec![161, 97, 49, 130, 130, 99, 49, 50, 51, 67, 1, 2, 3, 130, 99, 49, 50, 51, 67, 1, 2, 3]
    );

    let p2: Pointer<_, RcK> = serde_cbor::from_slice(&r).unwrap();
    assert_eq!(p2, pointer);

    let cid = Cid::new_cid_v0(util::sha2_256_hash(b"something")).unwrap();
    let pointer2 = Pointer::<_, RcK>::from_link(cid);
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

    let store = new_cbor_store();
    let bit_width = DEFAULT_BIT_WIDTH;
    // bitfield is 0
    let node = Node::test_init(store.clone(), 0, vec![pointer.clone(), pointer2.clone()], 0);
    let r = serde_cbor::to_vec(&node).unwrap();
    println!("{:?}", r);
    assert_eq!(
        r,
        vec![
            130, 64, 130, 161, 97, 49, 130, 130, 99, 49, 50, 51, 67, 1, 2, 3, 130, 99, 49, 50, 51,
            67, 1, 2, 3, 161, 97, 48, 216, 42, 88, 35, 0, 18, 32, 63, 201, 182, 137, 69, 157, 115,
            143, 140, 136, 163, 164, 138, 169, 227, 53, 66, 1, 107, 122, 64, 82, 224, 1, 170, 165,
            54, 252, 167, 72, 19, 203
        ]
    );
    let node: PartNode<_, RcK> = serde_cbor::from_slice(&r).unwrap();
    let node = node.into_node(store.clone(), bit_width);
    println!("{:?}", node);

    // bitfield is 9999
    let node = Node::test_init(
        store.clone(),
        9999,
        vec![pointer.clone(), pointer2.clone()],
        0,
    );
    let r = serde_cbor::to_vec(&node).unwrap();
    println!("{:?}", r);
    assert_eq!(
        r,
        vec![
            130, 66, 39, 15, 130, 161, 97, 49, 130, 130, 99, 49, 50, 51, 67, 1, 2, 3, 130, 99, 49,
            50, 51, 67, 1, 2, 3, 161, 97, 48, 216, 42, 88, 35, 0, 18, 32, 63, 201, 182, 137, 69,
            157, 115, 143, 140, 136, 163, 164, 138, 169, 227, 53, 66, 1, 107, 122, 64, 82, 224, 1,
            170, 165, 54, 252, 167, 72, 19, 203
        ]
    );
    let node: PartNode<_, RcK> = serde_cbor::from_slice(&r).unwrap();
    let node = node.into_node(store.clone(), bit_width);
    println!("{:?}", node);

    // bitfield is 0x12345678
    let node = Node::test_init(
        store.clone(),
        305419896,
        vec![pointer.clone(), pointer2.clone()],
        0,
    );
    let r = serde_cbor::to_vec(&node).unwrap();
    println!("{:?}", r);
    assert_eq!(
        r,
        vec![
            130, 68, 18, 52, 86, 120, 130, 161, 97, 49, 130, 130, 99, 49, 50, 51, 67, 1, 2, 3, 130,
            99, 49, 50, 51, 67, 1, 2, 3, 161, 97, 48, 216, 42, 88, 35, 0, 18, 32, 63, 201, 182,
            137, 69, 157, 115, 143, 140, 136, 163, 164, 138, 169, 227, 53, 66, 1, 107, 122, 64, 82,
            224, 1, 170, 165, 54, 252, 167, 72, 19, 203
        ]
    );
    let node: PartNode<_, RcK> = serde_cbor::from_slice(&r).unwrap();
    let node = node.into_node(store.clone(), bit_width);
    println!("{:?}", node);
}
