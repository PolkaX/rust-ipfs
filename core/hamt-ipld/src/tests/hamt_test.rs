use super::*;

fn rand_value() -> Vec<u8> {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(30).collect();
    rand_string.into_bytes()
}

fn add_and_remove_keys(bit_width: u32, keys: &[&str], extra_keys: &[&str]) {
    use std::time::Instant;

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
    let node2 = part_node.into_node(cs.clone(), bit_width);
    assert_eq!(node, node2);
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
    let mut node = NodeRc::new(cs);
    for k in &keys[..3] {
        node.set(k, b"foobar".to_vec()).unwrap();
    }

    let r = node.set(keys[3], b"foobar".to_vec());
    assert!(r.is_err(), true);
    // Try forcing the depth beyond 32
    node.set(&keys[3][1..], b"foobar".to_vec()).unwrap();
}

#[test]
fn test_basic() {
    let cs = new_cbor_store();
    let begin_node = NodeRc::new(cs);
}
