// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::collections::{BTreeMap, HashMap};
use std::convert::TryFrom;
use std::fs;

use block_format::{BasicBlock, Block};
use cid::{Cid, Codec};
use either::Either;
use ipld_format::Resolver;
use multihash::Hash;
use rust_ipld_cbor::{json_to_obj, obj_to_json, IpldNode, Obj, SortedStr, CborCid};
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
    let node = IpldNode::from_cbor("`".as_bytes(), Hash::SHA2256).unwrap();
    assert_eq!(
        &node.cid().to_string(),
        "bafyreiengp2sbi6ez34a2jctv34bwyjl7yoliteleaswgcwtqzrhmpyt2m"
    );
    let cbor_bytes = node.to_cbor().unwrap();
    assert_eq!(vec![b'`'], cbor_bytes)
}

const TEST_OBJ_ROOT: &'static str = "tests/test_objects/";

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

    #[derive(Debug, Serialize, Deserialize)]
    struct Foo {
        zebra: String,
        dog: i32,
        cats: f64,
        whale: String,
        cat: bool,
    }
    let foo = Foo {
        zebra: "seven".to_string(),
        dog: 15,
        cats: 1.519,
        whale: "never".to_string(),
        cat: true,
    };
    let value = serde_cbor::value::to_value(foo).unwrap();
    let obj = Obj::try_from(value).unwrap();
    let node = IpldNode::from_obj(obj, Hash::SHA2256).unwrap();

    let expect = hex!("a563636174f563646f670f6463617473fb3ff84dd2f1a9fbe7657768616c65656e65766572657a6562726165736576656e");
    assert_eq!(node.raw_data(), expect.as_ref());
}

/*
#[test]
fn test_basic_marshal() {
    let c = Cid::new_cid_v0(util::sha2_256_hash(b"something")).unwrap();
    let mut m = BTreeMap::new();
    m.insert("name".to_string().into(), Obj::Text("foo".to_string()));
    m.insert("bar".to_string().into(), Obj::Cid(c.clone().into()));
    let obj = Obj::Map(m);

    let raw = dump_object(&obj).unwrap();

    let nd = wrap_obj(obj, Hash::SHA2256).unwrap();
    assert_eq!(
        &nd.cid().to_string(),
        "bafyreib4hmpkwa7zyzoxmpwykof6k7akxnvmsn23oiubsey4e2tf6gqlui"
    );

    let back = decode(nd.raw_data(), Hash::SHA2256).unwrap();
    assert_eq!(
        &back.cid().to_string(),
        "bafyreib4hmpkwa7zyzoxmpwykof6k7akxnvmsn23oiubsey4e2tf6gqlui"
    );

    let (lnk, _) = back.resolve_link(&vec!["bar"]).unwrap();
    assert_eq!(lnk.cid, c);
    assert_eq!(nd.cid(), back.cid())
}

#[test]
fn test_marshal_roundtrip() {
    let c1 = Cid::new_cid_v0(util::sha2_256_hash(b"something1")).unwrap();
    let c2 = Cid::new_cid_v0(util::sha2_256_hash(b"something2")).unwrap();
    let c3 = Cid::new_cid_v0(util::sha2_256_hash(b"something3")).unwrap();

    let mut m = BTreeMap::new();
    m.insert("foo".to_string().into(), Obj::Text("bar".to_string()));
    m.insert("hello".to_string().into(), Obj::Cid(c1.clone().into()));
    m.insert(
        "baz".to_string().into(),
        Obj::Array(vec![Obj::Cid(c1.into()), Obj::Cid(c2.clone().into())]),
    );
    let mut cats_m = BTreeMap::new();
    cats_m.insert("qux".to_string().into(), Obj::Cid(c3.into()));
    m.insert("cats".to_string().into(), Obj::Map(cats_m));
    let obj = Obj::Map(m);

    let nd1 = wrap_obj(obj, Hash::SHA2256).unwrap();
    assert_eq!(
        &nd1.cid().to_string(),
        "bafyreibgx4rjaqolj7c32c7ibxc5tedhisc4d23ihx5t4tgamuvy2hvwjm"
    );

    assert_eq!(nd1.links().len(), 4_usize);

    let nd2 = decode(nd1.raw_data(), Hash::SHA2256).unwrap();
    assert_eq!(nd1.cid(), nd2.cid());

    let (link, rest) = nd2.resolve_link(&vec!["baz", "1", "bop"]).unwrap();
    assert_eq!(link.cid, c2);

    assert_eq!(rest.len(), 1);
    assert_eq!(rest[0], "bop");

    let output = to_json(&nd1).unwrap();
    println!("{:}", output);
    let raw = r#"
    {"baz":[{"/":"Qma75NN8GaM99ioqsNUF9Ho816SonoGsVrvSnqz9uL4LPF"},{"/":"QmUE28rcN99es8ntD4T3sBScfyrADkF3q8qzM1gEW82oMh"}],"cats":{"qux":{"/":"QmSsM8Xm1g5WtfwkBvnttZafpDhaW6jkXeSFccECUnx7hg"}},"foo":"bar","hello":{"/":"Qma75NN8GaM99ioqsNUF9Ho816SonoGsVrvSnqz9uL4LPF"}}
    "#;
    let v: serde_json::Value = serde_json::from_slice(raw.as_ref()).unwrap();
    let v2: serde_json::Value = serde_json::from_slice(output.as_ref()).unwrap();
    assert_eq!(v, v2);
}
*/

/*
#[test]
fn test_tree() {
    let c1 = Cid::new_cid_v0(util::sha2_256_hash(b"something1")).unwrap();
    let c2 = Cid::new_cid_v0(util::sha2_256_hash(b"something2")).unwrap();
    let c3 = Cid::new_cid_v0(util::sha2_256_hash(b"something3")).unwrap();
    let c4 = Cid::new_cid_v0(util::sha2_256_hash(b"something4")).unwrap();

    let mut obj_m = BTreeMap::new();
    obj_m.insert("foo".into(), Obj::Cid(c1.into()));
    obj_m.insert(
        "baz".into(),
        Obj::Array(vec![
            Obj::Cid(c2.into()),
            Obj::Cid(c3.into()),
            Obj::Text("c".to_string()),
        ]),
    );
    let mut cats_m = BTreeMap::new();
    {
        let mut qux_m = BTreeMap::new();
        {
            qux_m.insert("boo".into(), Obj::Integer(1));
            qux_m.insert("baa".into(), Obj::Cid(c4.into()));
            qux_m.insert("bee".into(), Obj::Integer(3));
            qux_m.insert("bii".into(), Obj::Integer(4));

            let mut buu_m = BTreeMap::new();
            buu_m.insert("coat".into(), Obj::Text("rain".to_string()));
            qux_m.insert("buu".into(), Obj::Map(buu_m));
        }
        cats_m.insert("qux".into(), Obj::Map(qux_m));
    }
    obj_m.insert("cats".into(), Obj::Map(cats_m));
    let obj = Obj::Map(obj_m);

    let nd = wrap_obj(obj, Hash::SHA2256).unwrap();

    assert_eq!(
        &nd.cid().to_string(),
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
    let mut paths = nd.tree("", None);
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
    let mut paths = nd.tree("cats", None);
    paths.sort();
    cats.sort();
    assert_eq!(paths, cats);

    let mut toplevel = vec!["foo".to_string(), "baz".to_string(), "cats".to_string()];
    let mut paths = nd.tree("", Some(1));
    paths.sort();
    toplevel.sort();
    assert_eq!(paths, toplevel);

    let v: Vec<String> = vec![];
    assert_eq!(nd.tree("", Some(0)), v);
}
*/

/*
#[test]
fn test_resolved_val_is_jsonable() {
    let data = r#"{
        "foo": {
            "bar": 1,
            "baz": 2
        }
    }"#;

    let n = from_json(data, Hash::SHA2256).unwrap();
    assert_eq!(
        &n.cid().to_string(),
        "bafyreiahcy6ewqmabbh7lcjhxrillpf72zlu3vqcovckanvj2fwdtenvbe"
    );
    let (val, _) = n.resolve(&vec!["foo"]).unwrap();
    match val {
        Either::Left(_) => panic!(""),
        Either::Right(obj) => {
            let s = obj_to_json(obj).unwrap();
            assert_eq!(&s, r#"{"bar":1,"baz":2}"#)
        }
    }
}
*/
