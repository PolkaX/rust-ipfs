// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::collections::BTreeMap;
use std::time::Instant;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use super::*;
use crate::node::stats;

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
    let mut begin_node = Hamt::new_with_bitwidth(cs.clone(), bit_width);
    for (k, v) in all.iter() {
        begin_node.set(k, v.clone()).unwrap();
    }
    println!("start flush");
    let now = Instant::now();
    let cid = begin_node.flush().unwrap();
    println!("flush took: {}", now.elapsed().as_nanos());

    let mut node = Hamt::load_with_bitwidth(cs.clone(), &cid, bit_width).unwrap();

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
    let cid2 = begin_node.flush().unwrap();
    let mut node2 = Hamt::load(cs, &cid2).unwrap();
    node_equal(&mut node, &mut node2);
}

#[cfg(feature = "test-hash")]
#[test]
fn test_canonical_structure() {
    let k1 = ["K"];
    let k2 = ["B"];
    add_and_remove_keys(DEFAULT_BIT_WIDTH, &k1, &k2);
    let k1 = ["K0", "K1", "KAA1", "KAA2", "KAA3"];
    let k2 = ["KAA4"];
    add_and_remove_keys(DEFAULT_BIT_WIDTH, &k1, &k2);
}

#[cfg(feature = "test-hash")]
#[test]
fn test_canonical_structure_alternate_bit_width() {
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
fn test_overflow() {
    let keys = [
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA0",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA1",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA2",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA3",
    ];
    let cs = new_cbor_store();
    let mut node = Hamt::new(cs);
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
    let mut begin_node = Hamt::new(cs.clone());
    let val = b"cat dog bear".to_vec();
    let key = "foo";
    begin_node.set(key, val.clone()).unwrap();

    for _ in 0..1000 {
        let k = rand_string();
        begin_node.set(&k, rand_value()).unwrap()
    }

    let cid = begin_node.flush().unwrap();

    let node = Hamt::load(cs, &cid).unwrap();
    let v: Vec<u8> = node.find(key).unwrap();
    assert_eq!(v, val);
}

#[test]
fn test_set_get() {
    let mut map: HashMap<String, Vec<u8>> = HashMap::new();
    for _ in 0..100_000 {
        map.insert(rand_string(), rand_value());
    }

    let cs = new_cbor_store();
    let mut begin_node = Hamt::new(cs.clone());
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
    let cid = begin_node.flush().unwrap();
    println!("flush took: {}", now.elapsed().as_millis());

    let mut node = Hamt::load_with_bitwidth(cs, &cid, begin_node.bit_width()).unwrap();

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
    let mut n = Hamt::new(cs.clone());
    let c = n.flush().unwrap();
    let mut on = Hamt::load(cs, &c).unwrap();
    on.set("foo", b"bar".to_vec()).unwrap();
}

#[test]
fn test_value_linking() {
    use ipld_cbor::Obj;

    let mut cs = new_cbor_store();
    let mut thingy1 = HashMap::new();
    thingy1.insert("cat".to_string(), "dog".to_string());
    let c1 = cs.put(thingy1).unwrap();

    let c = Obj::Cid(c1);
    let mut hash = BTreeMap::new();
    hash.insert("one".into(), c);
    hash.insert("foo".into(), Obj::Text("bar".to_string()));
    let thingy2 = Obj::Map(hash);

    let mut n = Hamt::new(cs);
    n.set("cat", thingy2).unwrap();
    let tcid = n.flush().unwrap();

    assert_eq!(
        &tcid.to_string(),
        "bafy2bzacedrwmwbquhdfs2ivq4dhejgyo2jxhlw2xdrrjwcarbimpxuvmx3e4"
    );
}
