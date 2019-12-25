// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::fs;

use block_format::{BasicBlock, Block};
use cid::{Cid, Codec};
use either::Either;
use ipld_format::{Node, Resolver};
use maplit::btreemap;
use multihash::Hash;
use rust_ipld_cbor::{json_to_obj, obj_to_json, IpldNode, Obj};
use serde::{Deserialize, Serialize};

#[test]
fn test_from_obj() {
    let node = IpldNode::from_obj(Obj::Text("".to_string()), Hash::SHA2256).unwrap();
    assert_eq!(
        &node.cid().to_string(),
        "bafyreiengp2sbi6ez34a2jctv34bwyjl7yoliteleaswgcwtqzrhmpyt2m"
    );
}

#[test]
fn test_json_roundtrip() {
    let examples = vec![
        (
            "[null]",
            "bafyreigtpkiih7wr7wb7ts6j5aunnotxiff3yqkx33rs4k4xhskprx5tui",
        ),
        (
            "[]",
            "bafyreidwx2fvfdiaox32v2mnn6sxu3j4qoxeqcuenhtgrv5qv6litfnmoe",
        ),
        (
            "{}",
            "bafyreigbtj4x7ip5legnfznufuopl4sg4knzc2cof6duas4b3q2fy6swua",
        ),
        (
            "null",
            "bafyreifqwkmiw256ojf2zws6tzjeonw6bpd5vza4i22ccpcq4hjv2ts7cm",
        ),
        (
            "1",
            "bafyreihtx752fmf3zafbys5dtr4jxohb53yi3qtzfzf6wd5274jwtn5agu",
        ),
        (
            "[1]",
            "bafyreihwrdqkjomfjaoqe5hbpfjzqoxkhptohvoa5u362s6obgpvxcw45q",
        ),
        (
            "true",
            "bafyreibhvppn37ufanewvxvwendgzksh3jpwhk6sxrx2dh3m7s3t5t7noa",
        ),
        (
            r#"{"a":"IPFS"}"#,
            "bafyreihyyz2badz34h5pvcgof4fj3qwwr7mopoucejwbnpzs7soorkrct4",
        ),
        (
            r#"{"a":"IPFS","b":null,"c":[1]}"#,
            "bafyreigg2gcszayx2lywb3edqfoftyvus7gxeanmudqla3e6eh2okei25a",
        ),
        (
            r#"{"a":[]}"#,
            "bafyreian4t6wau4jdqt6nys76dfvsn6g7an4ulbv326yzutdgnrr5cjpui",
        ),
    ];
    for (json, expect_cid) in examples {
        let node = IpldNode::from_json(json, Hash::SHA2256).unwrap();
        assert_eq!(&node.cid().to_string(), expect_cid);
        let s = node.to_json().unwrap();
        assert_eq!(json, &s);
    }
}

#[test]
fn test_cbor_roundtrip() {
    let node = IpldNode::from_cbor(b"`", Hash::SHA2256).unwrap();
    assert_eq!(
        &node.cid().to_string(),
        "bafyreiengp2sbi6ez34a2jctv34bwyjl7yoliteleaswgcwtqzrhmpyt2m"
    );
    let cbor_bytes = node.to_cbor().unwrap();
    assert_eq!(vec![b'`'], cbor_bytes)
}

const TEST_OBJ_ROOT: &str = "tests/test_objects/";

#[test]
fn test_objects() {
    let content = fs::read_to_string(format!("{}expected.json", TEST_OBJ_ROOT)).unwrap();
    let obj = json_to_obj(&content).unwrap();
    if let Obj::Map(map) = obj {
        for (key, value) in map {
            let json_file_name = format!("{}{}.json", TEST_OBJ_ROOT, key);
            let json = fs::read_to_string(json_file_name).unwrap();
            let cbor_file_name = format!("{}{}.cbor", TEST_OBJ_ROOT, key);
            let cbor = fs::read(cbor_file_name).unwrap();

            let node = IpldNode::from_json(&json, Hash::SHA2256).unwrap();
            assert_eq!(node.raw_data(), cbor.as_slice());

            if let Obj::Cid(cid) = value {
                assert_eq!(node.cid(), &cid.into_inner());
            } else {
                unreachable!()
            }
        }
    } else {
        unreachable!()
    }
}

#[test]
fn test_canonicalize() {
    let cbor_file_name = format!("{}non-canon.cbor", TEST_OBJ_ROOT);
    let cbor = fs::read(cbor_file_name).unwrap();
    let node1 = IpldNode::from_cbor(&cbor, Hash::SHA2256).unwrap();
    assert_ne!(node1.raw_data(), cbor.as_slice());

    assert_eq!(
        &node1.cid().to_string(),
        "bafyreiawx7ona7oa2ptcoh6vwq4q6bmd7x2ibtkykld327bgb7t73ayrqm"
    );

    let node2 = IpldNode::from_cbor(node1.raw_data(), Hash::SHA2256).unwrap();
    assert_eq!(node1, node2);
}

#[test]
fn test_stable_cid() {
    let cbor_file_name = format!("{}non-canon.cbor", TEST_OBJ_ROOT);
    let cbor = fs::read(cbor_file_name).unwrap();
    let cid = Cid::new_cid_v1(Codec::DagCBOR, util::sha2_256_hash(&cbor)).unwrap();
    let bad_block = BasicBlock::new_with_cid(cbor.into(), cid).unwrap();
    let bad_node = IpldNode::from_block(&bad_block).unwrap();
    assert_eq!(bad_block.cid(), bad_node.cid());
}

#[test]
fn test_canonical_struct_encoding() {
    use hex_literal::hex;

    #[derive(PartialEq, Debug, Serialize, Deserialize)]
    struct Foo {
        zebra: String,
        dog: i32,
        cats: f64,
        whale: String,
        cat: bool,
    }
    let foo1 = Foo {
        zebra: "seven".to_string(),
        dog: 15,
        cats: 1.519,
        whale: "never".to_string(),
        cat: true,
    };
    let cbor = serde_cbor::to_vec(&foo1).unwrap();
    let node = IpldNode::from_cbor(&cbor, Hash::SHA2256).unwrap();
    let expect = hex!("a563636174f563646f670f6463617473fb3ff84dd2f1a9fbe7657768616c65656e65766572657a6562726165736576656e");
    assert_eq!(node.raw_data().as_ref(), &expect[..]);

    let foo2 = serde_cbor::from_slice::<Foo>(&cbor).unwrap();
    assert_eq!(foo1, foo2);
}

#[test]
fn test_tree() {
    let c1 = Cid::new_cid_v0(util::sha2_256_hash(b"something1")).unwrap();
    let c2 = Cid::new_cid_v0(util::sha2_256_hash(b"something2")).unwrap();
    let c3 = Cid::new_cid_v0(util::sha2_256_hash(b"something3")).unwrap();
    let c4 = Cid::new_cid_v0(util::sha2_256_hash(b"something4")).unwrap();
    let obj = Obj::Map(btreemap! {
        "foo".into() => Obj::Cid(c1.into()),
        "baz".into() => Obj::Array(vec![Obj::Cid(c2.into()), Obj::Cid(c3.into()), Obj::Text("c".to_string())]),
        "cats".into() => Obj::Map(btreemap!{
            "qux".into() => Obj::Map(btreemap!{
                "boo".into() => Obj::Integer(1),
                "baa".into() => Obj::Cid(c4.into()),
                "bee".into() => Obj::Integer(3),
                "bii".into() => Obj::Integer(4),
                "buu".into() => Obj::Map(btreemap!{
                    "coat".into() => Obj::Text("rain".to_string()),
                }),
            }),
        }),
    });
    let node = IpldNode::from_obj(obj, Hash::SHA2256).unwrap();
    assert_eq!(
        &node.cid().to_string(),
        "bafyreicp66zmx7grdrnweetu23anx3e5zguda7646iwyothju6nhgqykgq"
    );

    let mut full = vec![
        "foo".to_string(),
        "baz".to_string(),
        "baz/0".to_string(),
        "baz/1".to_string(),
        "baz/2".to_string(),
        "cats".to_string(),
        "cats/qux".to_string(),
        "cats/qux/boo".to_string(),
        "cats/qux/baa".to_string(),
        "cats/qux/bee".to_string(),
        "cats/qux/bii".to_string(),
        "cats/qux/buu".to_string(),
        "cats/qux/buu/coat".to_string(),
    ];
    let mut paths = node.tree("", None);
    paths.sort();
    full.sort();
    assert_eq!(paths, full);

    let mut cats = vec![
        "qux".to_string(),
        "qux/boo".to_string(),
        "qux/baa".to_string(),
        "qux/bee".to_string(),
        "qux/bii".to_string(),
        "qux/buu".to_string(),
        "qux/buu/coat".to_string(),
    ];
    let mut paths = node.tree("cats", None);
    paths.sort();
    cats.sort();
    assert_eq!(paths, cats);

    let mut toplevel = vec!["foo".to_string(), "baz".to_string(), "cats".to_string()];
    let mut paths = node.tree("", Some(1));
    paths.sort();
    toplevel.sort();
    assert_eq!(paths, toplevel);

    assert_eq!(node.tree("", Some(0)), Vec::<String>::new());
}

#[test]
fn test_resolved_val_is_jsonable() {
    let json = r#"{
        "foo": {
            "bar": 1,
            "baz": 2
        }
    }"#;

    let node = IpldNode::from_json(json, Hash::SHA2256).unwrap();
    assert_eq!(
        &node.cid().to_string(),
        "bafyreiahcy6ewqmabbh7lcjhxrillpf72zlu3vqcovckanvj2fwdtenvbe"
    );

    if let (Either::Right(obj), _) = node.resolve(&["foo"]).unwrap() {
        assert_eq!(&obj_to_json(obj).unwrap(), r#"{"bar":1,"baz":2}"#);
    } else {
        unreachable!()
    }
}

#[test]
fn test_basic_marshal() {
    let cid = Cid::new_cid_v0(util::sha2_256_hash(b"something")).unwrap();
    let obj = Obj::Map(btreemap! {
        "name".into() => Obj::Text("foo".to_string()),
        "bar".into() => Obj::Cid(cid.clone().into()),
    });
    let node = IpldNode::from_obj(obj, Hash::SHA2256).unwrap();
    assert_eq!(
        &node.cid().to_string(),
        "bafyreib4hmpkwa7zyzoxmpwykof6k7akxnvmsn23oiubsey4e2tf6gqlui"
    );

    let back = IpldNode::from_cbor(node.raw_data(), Hash::SHA2256).unwrap();
    assert_eq!(
        &back.cid().to_string(),
        "bafyreib4hmpkwa7zyzoxmpwykof6k7akxnvmsn23oiubsey4e2tf6gqlui"
    );

    let (link, _) = back.resolve_link(&["bar"]).unwrap();
    assert_eq!(link.cid, cid);
}

#[test]
fn test_marshal_roundtrip() {
    let c1 = Cid::new_cid_v0(util::sha2_256_hash(b"something1")).unwrap();
    let c2 = Cid::new_cid_v0(util::sha2_256_hash(b"something2")).unwrap();
    let c3 = Cid::new_cid_v0(util::sha2_256_hash(b"something3")).unwrap();
    let obj = Obj::Map(btreemap! {
        "foo".into() => Obj::Text("bar".to_string()),
        "hello".into() => Obj::Cid(c1.clone().into()),
        "baz".into() => Obj::Array(vec![Obj::Cid(c1.into()), Obj::Cid(c2.clone().into())]),
        "cats".into() => Obj::Map(btreemap!{
            "qux".into() => Obj::Cid(c3.into()),
        }),
    });
    let node1 = IpldNode::from_obj(obj, Hash::SHA2256).unwrap();
    assert_eq!(
        &node1.cid().to_string(),
        "bafyreibgx4rjaqolj7c32c7ibxc5tedhisc4d23ihx5t4tgamuvy2hvwjm"
    );
    assert_eq!(node1.links().len(), 4);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&node1.to_json().unwrap()).unwrap(),
        serde_json::json!({
            "baz":[
                {"/":"Qma75NN8GaM99ioqsNUF9Ho816SonoGsVrvSnqz9uL4LPF"},
                {"/":"QmUE28rcN99es8ntD4T3sBScfyrADkF3q8qzM1gEW82oMh"}
            ],
            "cats":{
                "qux":{"/":"QmSsM8Xm1g5WtfwkBvnttZafpDhaW6jkXeSFccECUnx7hg"}
            },
            "foo":"bar",
            "hello":{"/":"Qma75NN8GaM99ioqsNUF9Ho816SonoGsVrvSnqz9uL4LPF"}
        }),
    );

    let node2 = IpldNode::from_cbor(node1.raw_data(), Hash::SHA2256).unwrap();
    assert_eq!(node1.cid(), node2.cid());
    let (link, rest) = node2.resolve_link(&["baz", "1", "bop"]).unwrap();
    assert_eq!(link.cid, c2);
    assert_eq!(rest.len(), 1);
    assert_eq!(rest[0], "bop");
}
