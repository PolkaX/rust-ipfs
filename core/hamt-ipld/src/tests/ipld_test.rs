use super::*;

use crate::node::set_bit;
use archery::RcK;

#[test]
fn test_roundtrip() {
    let cs = new_cbor_store();
    let mut n = Node::<_, RcK>::new(cs.clone());
    set_bit(n.get_mut_bitfield(), 5);
    set_bit(n.get_mut_bitfield(), 7);
    set_bit(n.get_mut_bitfield(), 18);

    let v = vec![0x83_u8, 0x01, 0x02, 0x03];
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
