// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::collections::BTreeMap;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde::{Deserialize, Serialize};

use ipld_core::{IpldNode, IpldValue};

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
struct MyStruct {
    items: BTreeMap<String, MyStruct>,
    foo: String,
    bar: Vec<u8>,
    baz: Vec<i32>,
}

fn test_struct() -> MyStruct {
    let mut map = BTreeMap::new();
    map.insert(
        "Foo".to_string(),
        MyStruct {
            foo: "Foo".to_string(),
            bar: b"Bar".to_vec(),
            baz: vec![1, 2, 3, 4],
            ..Default::default()
        },
    );
    map.insert(
        "Bar".to_string(),
        MyStruct {
            bar: b"Bar".to_vec(),
            baz: vec![1, 2, 3, 4],
            ..Default::default()
        },
    );

    MyStruct {
        items: map,
        baz: vec![5, 1, 2],
        ..Default::default()
    }
}

fn test_struct_obj() -> IpldValue {
    let obj = test_struct();
    let json = serde_json::to_string(&obj).unwrap();
    serde_json::from_str::<IpldValue>(&json).unwrap()
}

fn bench_from_object(c: &mut Criterion) {
    let obj = test_struct_obj();

    c.bench_function("from_obj", |b| {
        b.iter(|| {
            let _ =
                black_box(IpldNode::from_object(obj.clone(), multihash::Code::Sha2_256).unwrap());
        })
    });
}

fn bench_from_block(c: &mut Criterion) {
    let obj = test_struct_obj();
    let node = IpldNode::from_object(obj, multihash::Code::Sha2_256).unwrap();

    c.bench_function("from_block", |b| {
        b.iter(|| {
            let n = black_box(IpldNode::from_block(&node).unwrap());
            assert_eq!(node, n);
        })
    });
}

fn bench_to_cbor(c: &mut Criterion) {
    let obj = test_struct_obj();
    let node = IpldNode::from_object(obj, multihash::Code::Sha2_256).unwrap();

    c.bench_function("to_cbor", |b| {
        b.iter(|| {
            let _ = black_box(node.to_cbor().unwrap());
        })
    });
}

criterion_group!(benches, bench_from_object, bench_from_block, bench_to_cbor);
criterion_main!(benches);
