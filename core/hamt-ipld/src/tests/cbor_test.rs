use super::*;
use crate::node::Pointer;

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
fn test_pointer() {
    let kv = KV::new("123".to_string(), vec![1_u8, 2, 3]);
    let pointer = Pointer::from_kvs(vec![kv.clone(), kv]);
    let r = serde_cbor::to_vec(&pointer).unwrap();
    println!("{:?}", r);
    assert_eq!(
        r,
        vec![161, 97, 49, 130, 130, 99, 49, 50, 51, 67, 1, 2, 3, 130, 99, 49, 50, 51, 67, 1, 2, 3]
    );

    let p2: Pointer = serde_cbor::from_slice(&r).unwrap();
    assert_eq!(p2, pointer);

    let cid = Cid::new_cid_v0(util::sha2_256_hash(b"something")).unwrap();
    let pointer2 = Pointer::from_link(cid);
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
}
