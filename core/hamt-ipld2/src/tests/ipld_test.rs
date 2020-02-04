// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use archery::RcK;
use ipld_cbor::struct_to_cbor_value;

use super::*;
use crate::node::set_bit;

#[test]
fn test_roundtrip() {
    let cs = new_cbor_store();
    let mut n = Node::<_, RcK>::new(cs.clone());
    set_bit(n.get_mut_bitfield(), 5);
    set_bit(n.get_mut_bitfield(), 7);
    set_bit(n.get_mut_bitfield(), 18);

    let v = struct_to_cbor_value(&vec![0x83_u8, 0x01, 0x02, 0x03]).unwrap();
    let kv = KV::new("foo".to_string(), v);
    let p = Pointer::from_kvs(vec![kv]);
    n.get_mut_pointers().push(p);

    let cid = cs.put(n).unwrap();
    let n2: PartNode<MockBlocks, RcK> = cs.get(&cid).unwrap();
    let n2 = n2.into_node(cs.clone(), DEFAULT_BIT_WIDTH);

    let c2 = cs.put(n2).unwrap();
    assert_eq!(cid, c2);
}

#[test]
fn test_basic_bytes_loading() {
    let b = b"cats and dogs are taking over".to_vec();
    let o = ipld_cbor::dump_object::<Vec<u8>>(b.as_ref()).unwrap();
    let s: Vec<u8> = ipld_cbor::decode_into(&o).unwrap();
    assert_eq!(b, s);
}

#[cfg(not(feature = "test-hash"))]
#[test]
fn test_kv() {
    use ipld_cbor::Obj;
    use std::collections::BTreeMap;

    let cs = new_cbor_store();
    let mut thingy1 = HashMap::new();
    thingy1.insert("cat".to_string(), "dog".to_string());
    let c1 = cs.put(thingy1).unwrap();

    let c = Obj::Cid(c1);
    let mut hash = BTreeMap::new();
    hash.insert("one".into(), c);
    hash.insert("foo".into(), Obj::Text("bar".to_string()));
    let thingy2 = Obj::Map(hash);

    let b = ipld_cbor::dump_object(&thingy2).unwrap();
    println!("{:?}", b);

    let mut node = NodeRc::new(cs);
    node.set("cat", thingy2).unwrap();

    let b = ipld_cbor::dump_object(&node).unwrap();
    println!("{:?} {}", b, b.len());

    assert_eq!(
        b,
        vec![
            130, 88, 30, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 129, 161, 97, 49, 129, 130, 99, 99, 97, 116, 162, 99, 102, 111, 111, 99,
            98, 97, 114, 99, 111, 110, 101, 216, 42, 88, 39, 0, 1, 113, 160, 228, 2, 32, 236, 82,
            22, 81, 235, 94, 233, 82, 25, 143, 252, 235, 234, 106, 54, 182, 180, 21, 78, 124, 8,
            62, 200, 123, 158, 248, 162, 126, 171, 22, 88, 64
        ]
    )
}
