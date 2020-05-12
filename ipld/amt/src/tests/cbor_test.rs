use super::*;

#[test]
fn node_test() {
    let cid = Cid::new_v0(multihash::Sha2_256::digest(b"something")).unwrap();
    let b = Item::Link(cid);
    let node = Node::new_from_raw(1, vec![b], vec![]);
    let v = serde_cbor::to_vec(&node).unwrap();
    println!("{:?}", v);
    assert_eq!(
        v,
        vec![
            131, 65, 1, 129, 216, 42, 88, 35, 0, 18, 32, 63, 201, 182, 137, 69, 157, 115, 143, 140,
            136, 163, 164, 138, 169, 227, 53, 66, 1, 107, 122, 64, 82, 224, 1, 170, 165, 54, 252,
            167, 72, 19, 203, 128
        ]
    );
    let n: Node = serde_cbor::from_slice(&v).unwrap();
    assert_eq!(node, n);

    let node = Node::new_from_raw(255, vec![], vec![Value::Text("123".to_string())]);
    let v = serde_cbor::to_vec(&node).unwrap();
    println!("{:?}", v);
    assert_eq!(v, vec![131, 65, 255, 128, 129, 99, 49, 50, 51]);
    let n: Node = serde_cbor::from_slice(&v).unwrap();
    assert_eq!(node, n);
}

#[test]
fn root_test() {
    let cid = Cid::new_v0(multihash::Sha2_256::digest(b"something")).unwrap();
    let b = Item::Link(cid);
    let node = Node::new_from_raw(1, vec![b], vec![]);
    let db = db();
    let root = create_root(4, 100, node, db.clone());
    let v = serde_cbor::to_vec(&root).unwrap();
    println!("{:?}", v);

    assert_eq!(
        v,
        vec![
            131, 4, 24, 100, 131, 65, 1, 129, 216, 42, 88, 35, 0, 18, 32, 63, 201, 182, 137, 69,
            157, 115, 143, 140, 136, 163, 164, 138, 169, 227, 53, 66, 1, 107, 122, 64, 82, 224, 1,
            170, 165, 54, 252, 167, 72, 19, 203, 128
        ]
    );

    let pr: PartAmt = serde_cbor::from_slice(&v).unwrap();
    let r = Amt::from_part(pr, db);
    assert_eq!(root, r);
}
