// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::collections::BTreeMap;
use std::ops::Deref;
use std::time::Instant;

use archery::SharedPointerKind;
use matches::matches;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use super::*;

fn rand_string() -> String {
    let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(18).collect();
    rand_string
}

fn rand_value() -> Vec<u8> {
    let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(30).collect();
    rand_string.into_bytes()
}

#[cfg(feature = "test-hash")]
fn add_and_remove_keys(bit_width: u32, keys: &[&str], extra_keys: &[&str]) {
    let all: Vec<(&str, Vec<u8>)> = keys.iter().map(|k| (*k, rand_value())).collect();

    let cs = new_cbor_store();
    let mut begin_node = NodeRc::new_with_bitwidth(cs.clone(), bit_width);
    for (k, v) in all.iter() {
        begin_node.set(k, v.clone()).unwrap();
    }
    println!("start flush");
    let now = Instant::now();
    begin_node.flush().unwrap();
    println!("flush took: {}", now.elapsed().as_micros());

    let cid = cs.put(&begin_node).unwrap();
    let part_node: PartNodeRc<_> = cs.get(&cid).unwrap();
    let node = part_node.into_node(cs.clone(), bit_width);

    for (k, v) in all {
        let v2: Vec<u8> = node.find(k).unwrap();
        assert_eq!(v, v2);
    }

    // create second hamt by adding and deleting the extra keys
    for k in extra_keys.iter() {
        begin_node.set(k, rand_value()).unwrap();
    }
    for k in extra_keys.iter() {
        begin_node.delete(k).unwrap();
    }
    begin_node.flush().unwrap();
    let cid2 = cs.put(begin_node).unwrap();
    let part_node: PartNodeRc<_> = cs.get(&cid2).unwrap();
    let node2 = part_node.into_node(cs, bit_width);
    assert_eq!(node, node2);
}

#[cfg(feature = "test-hash")]
#[test]
fn test_hash_canonical_structure() {
    let k1 = ["K"];
    let k2 = ["B"];
    add_and_remove_keys(DEFAULT_BIT_WIDTH, &k1, &k2);
    let k1 = ["K0", "K1", "KAA1", "KAA2", "KAA3"];
    let k2 = ["KAA4"];
    add_and_remove_keys(DEFAULT_BIT_WIDTH, &k1, &k2);
}

#[cfg(feature = "test-hash")]
#[test]
fn test_hash_canonical_structure_alternate_bit_width() {
    add_and_remove_keys(7, ["K"].as_ref(), ["B"].as_ref());
    add_and_remove_keys(
        7,
        ["K0", "K1", "KAA1", "KAA2", "KAA3"].as_ref(),
        ["KAA4"].as_ref(),
    );
    add_and_remove_keys(6, ["K"].as_ref(), ["B"].as_ref());
    add_and_remove_keys(
        6,
        ["K0", "K1", "KAA1", "KAA2", "KAA3"].as_ref(),
        ["KAA4"].as_ref(),
    );
    add_and_remove_keys(5, ["K"].as_ref(), ["B"].as_ref());
    add_and_remove_keys(
        5,
        ["K0", "K1", "KAA1", "KAA2", "KAA3"].as_ref(),
        ["KAA4"].as_ref(),
    );
}

#[cfg(feature = "test-hash")]
#[test]
fn test_hash_overflow() {
    let keys = [
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA0",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA1",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA2",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA3",
    ];
    let cs = new_cbor_store();
    let mut node = NodeRc::new(cs);
    for k in &keys[..3] {
        node.set(k, b"foobar".to_vec()).unwrap();
    }

    let res = node.set(keys[3], b"foobar".to_vec());
    assert!(matches!(res, Err(Error::MaxDepth)));
    // Try forcing the depth beyond 32
    node.set(&keys[3][1..], b"foobar".to_vec()).unwrap();
}

#[test]
fn test_basic() {
    let cs = new_cbor_store();
    let mut begin_node = NodeRc::new(cs.clone());
    let val = b"cat dog bear".to_vec();
    let key = "foo";
    begin_node.set(key, val.clone()).unwrap();

    for _ in 0..1000 {
        let k = rand_string();
        begin_node.set(&k, rand_value()).unwrap()
    }

    begin_node.flush().unwrap();

    let cid = cs.put(&begin_node).unwrap();
    let node = NodeRc::load_node(cs, cid).unwrap();
    let v: Vec<u8> = node.find(key).unwrap();
    assert_eq!(v, val);
}

#[derive(Debug, Default)]
struct HamtStats {
    total_nodes: usize,
    total_kvs: usize,
    counts: HashMap<usize, usize>,
}

fn stats<B, P>(node: &Node<B, P>) -> HamtStats
where
    B: Blocks,
    P: SharedPointerKind,
{
    let mut st = HamtStats::default();
    stats_rec(node, &mut st);
    st
}

fn stats_rec<B, P>(node: &Node<B, P>, st: &mut HamtStats)
where
    B: Blocks,
    P: SharedPointerKind,
{
    st.total_nodes += 1;
    for p in node.get_pointers().iter() {
        match p.data {
            PContent::Link(_) => {
                let child_node = p.load_child(node.get_store(), node.get_bitwidth()).unwrap();
                let node = child_node.read().unwrap();
                if let Some(n) = node.deref() {
                    stats_rec(n, st)
                } else {
                    unreachable!("node cache must be `Some()` here")
                }
            }
            PContent::KVs(ref kvs) => {
                st.total_kvs += kvs.len();
                *(st.counts.entry(kvs.len()).or_insert(0)) += 1;
            }
        }
    }
}

#[test]
fn test_set_get() {
    let mut map: HashMap<String, Vec<u8>> = HashMap::new();
    for _ in 0..100_000 {
        map.insert(rand_string(), rand_value());
    }

    let cs = new_cbor_store();
    let mut begin_node = NodeRc::new(cs.clone());
    for (k, v) in map.iter() {
        begin_node.set(k, v.clone()).unwrap();
    }

    let size = begin_node.check_size().unwrap();
    let map_size = map
        .iter()
        .fold(0, |last, item| last + (item.0.len() + item.1.len()));
    println!(
        "Total size is: {}, size of keys+vals: {}, overhead: {:.2}",
        size,
        map_size,
        size as f64 / map_size as f64
    );
    println!("stats:{:?}", stats(&begin_node));

    println!("start flush");
    let now = Instant::now();
    begin_node.flush().unwrap();
    println!("flush took: {}", now.elapsed().as_millis());

    let cid = cs.put(&begin_node).unwrap();
    let part_node: PartNodeRc<_> = cs.get(&cid).unwrap();
    let mut node = part_node.into_node(cs, begin_node.get_bitwidth());

    let now = Instant::now();
    for (k, v) in map.iter() {
        let map_v: Vec<u8> = node.find(k).unwrap();
        assert_eq!(map_v, *v);
    }
    println!("finds took: {}", now.elapsed().as_millis());

    for _ in 0..100 {
        let r = rand_string();
        let result = node.find::<Vec<u8>>(&r);
        assert!(matches!(result, Err(Error::NotFound(_))));
    }

    map.iter_mut().for_each(|(k, v)| {
        let new_v = rand_value();
        *v = new_v.clone();
        node.set(k, new_v).unwrap();
    });

    map.iter().for_each(|(k, v)| {
        let node_v: Vec<u8> = node.find(k).unwrap();
        assert_eq!(node_v, *v);
    });

    for _ in 0..100 {
        let r = rand_string();
        let result = node.delete(&r);
        assert!(matches!(result, Err(Error::NotFound(_))));
    }

    for (k, _) in map {
        node.delete(&k).unwrap();
        let result = node.find::<Vec<u8>>(&k);
        assert!(matches!(result, Err(Error::NotFound(_))));
    }
}

#[test]
fn test_reload_empty() {
    let cs = new_cbor_store();
    let n = NodeRc::new(cs.clone());
    let c = cs.put(&n).unwrap();
    let mut on = NodeRc::load_node(cs, c).unwrap();
    on.set("foo", b"bar".to_vec()).unwrap();
}

fn nodes_equal<B, P>(store: CborIpldStor<B>, n1: &mut Node<B, P>, n2: &mut Node<B, P>) -> bool
where
    B: Blocks,
    P: SharedPointerKind,
{
    n1.flush().unwrap();
    let n1_cid = store.put(n1).unwrap();
    n2.flush().unwrap();
    let n2_cid = store.put(n2).unwrap();
    n1_cid == n2_cid
}

#[test]
fn test_copy() {
    let cs = new_cbor_store();
    let mut n = NodeRc::new(cs.clone());
    let mut nc = n.clone();
    assert_eq!(nodes_equal(cs.clone(), &mut n, &mut nc), true);

    n.set("key", vec![1_u8]).unwrap();
    assert_eq!(nodes_equal(cs.clone(), &mut n, &mut nc), false);

    let mut nc = n.clone();
    nc.set("key2", vec![2_u8]).unwrap();
    assert_eq!(nodes_equal(cs.clone(), &mut n, &mut nc), false);

    let mut n = nc.clone();
    assert_eq!(nodes_equal(cs, &mut n, &mut nc), true);
}

#[test]
fn test_deep_copy() {
    let cs = new_cbor_store();
    let mut n = NodeRc::new(cs.clone());
    let mut nc = n.deep_copy();
    assert_eq!(nodes_equal(cs.clone(), &mut n, &mut nc), true);

    n.set("key", vec![1_u8]).unwrap();
    assert_eq!(nodes_equal(cs.clone(), &mut n, &mut nc), false);

    let mut nc = n.deep_copy();
    nc.set("key2", vec![2_u8]).unwrap();
    assert_eq!(nodes_equal(cs.clone(), &mut n, &mut nc), false);

    let mut n = nc.deep_copy();
    assert_eq!(nodes_equal(cs, &mut n, &mut nc), true);
}

#[test]
fn test_copy_copies_nil_slices() {
    let cs = new_cbor_store();
    let mut n = NodeRc::new(cs);
    let p = Pointer::from_kvs(vec![]);
    n.get_mut_pointers().push(p);

    if let PContent::KVs(kvs) = &n.get_pointers()[0].data {
        assert_eq!(kvs.len(), 0);
    } else {
        panic!("")
    }

    let nc = n.clone();
    if let PContent::KVs(kvs) = &nc.get_pointers()[0].data {
        assert_eq!(kvs.len(), 0);
    } else {
        panic!("")
    }

    let ncc = n.deep_copy();
    if let PContent::KVs(kvs) = &ncc.get_pointers()[0].data {
        assert_eq!(kvs.len(), 0);
    } else {
        panic!("")
    }
}

#[test]
fn test_copy_without_flush() {
    let cs = new_cbor_store();
    let mut n = NodeRc::new(cs);

    let count = 200_u8;
    for i in 0..count {
        let key = format!("key{}", i);
        n.set(&key, vec![i]).unwrap();
    }
    n.flush().unwrap();

    for i in 0..count {
        let key = format!("key{}", i);
        // override
        n.set(&key, (count as u32 + i as u32).to_be_bytes().to_vec())
            .unwrap();
    }

    let nc = n.clone();

    for i in 0..count {
        let key = format!("key{}", i);
        let val: Vec<u8> = n.find(&key).unwrap();

        let val_copy: Vec<u8> = nc.find(&key).unwrap();
        assert_eq!(val, val_copy);
    }

    let nc = n.deep_copy();

    for i in 0..count {
        let key = format!("key{}", i);
        let val: Vec<u8> = n.find(&key).unwrap();

        let val_copy: Vec<u8> = nc.find(&key).unwrap();
        assert_eq!(val, val_copy);
    }
}

#[test]
fn test_value_linking() {
    use ipld_cbor::{Node as _, Obj};

    let cs = new_cbor_store();
    let mut thingy1 = HashMap::new();
    thingy1.insert("cat".to_string(), "dog".to_string());
    let c1 = cs.put(thingy1).unwrap();

    let c = Obj::Cid(c1.clone());
    let mut hash = BTreeMap::new();
    hash.insert("one".into(), c);
    hash.insert("foo".into(), Obj::Text("bar".to_string()));
    let thingy2 = Obj::Map(hash);

    let mut n = NodeRc::new(cs.clone());
    n.set("cat", thingy2).unwrap();
    let tcid = cs.put(n).unwrap();

    let blk = cs.get_block(&tcid).unwrap();
    println!("{:?}", blk.raw_data().to_vec());
    let ipld_node = ipld_cbor::IpldNode::from_block(&*blk).unwrap();

    println!("thingy1:{}", c1.to_string());
    println!("{:?}", ipld_node.links()[0]);
}
