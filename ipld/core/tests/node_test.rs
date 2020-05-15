// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::fs;

use cid::{Cid, Codec, IntoExt};
use either::Either;
use maplit::btreemap;
use multihash::Code;

use block_format::{BasicBlock, Block};
use ipld_core::{IpldNode, IpldValue};
use ipld_format::{Node, Resolver};

#[test]
fn test_non_object() {
    let value = IpldValue::String("".into());
    let node = IpldNode::wrap_object(&value, Code::Sha2_256.into()).unwrap();
    assert_eq!(
        node.cid().to_string(),
        "bafyreiengp2sbi6ez34a2jctv34bwyjl7yoliteleaswgcwtqzrhmpyt2m"
    );
    let back = IpldNode::from_cbor(node.raw_data(), Code::Sha2_256.into()).unwrap();
    assert_eq!(
        back.cid().to_string(),
        "bafyreiengp2sbi6ez34a2jctv34bwyjl7yoliteleaswgcwtqzrhmpyt2m"
    );
}

#[test]
fn test_from_cbor() {
    let cases = vec![
        IpldValue::Map(btreemap! {
            "name".into() => IpldValue::String("foo".to_string()),
        }),
        IpldValue::String("foobar".to_string()),
    ];
    for value in cases {
        let node = IpldNode::wrap_object(&value, Code::Sha2_256.into()).unwrap();
        let de = minicbor::decode::<IpldValue>(node.raw_data()).unwrap();
        assert_eq!(de, value);
    }
}

#[test]
fn test_basic_marshal() {
    let cid = Cid::new_v0(Code::Sha2_256.digest(b"something").into_ext()).unwrap();
    let value = IpldValue::Map(btreemap! {
        "name".into() => IpldValue::String("foo".to_string()),
        "bar".into() => IpldValue::Link(cid.clone()),
    });
    let node = IpldNode::wrap_object(&value, Code::Sha2_256.into()).unwrap();
    assert_eq!(
        node.cid().to_string(),
        "bafyreib4hmpkwa7zyzoxmpwykof6k7akxnvmsn23oiubsey4e2tf6gqlui"
    );

    let back = IpldNode::from_cbor(node.raw_data(), Code::Sha2_256.into()).unwrap();
    assert_eq!(
        back.cid().to_string(),
        "bafyreib4hmpkwa7zyzoxmpwykof6k7akxnvmsn23oiubsey4e2tf6gqlui"
    );

    let (link, _) = back.resolve_link(&["bar"]).unwrap();
    assert_eq!(link.cid, cid);
}

#[test]
fn test_marshal_roundtrip() {
    let c1 = Cid::new_v0(Code::Sha2_256.digest(b"something1").into_ext()).unwrap();
    let c2 = Cid::new_v0(Code::Sha2_256.digest(b"something2").into_ext()).unwrap();
    let c3 = Cid::new_v0(Code::Sha2_256.digest(b"something3").into_ext()).unwrap();
    let value = IpldValue::Map(btreemap! {
        "foo".into() => IpldValue::String("bar".to_string()),
        "hello".into() => IpldValue::Link(c1.clone()),
        "baz".into() => IpldValue::List(vec![IpldValue::Link(c1), IpldValue::Link(c2.clone())]),
        "cats".into() => IpldValue::Map(btreemap!{
            "qux".into() => IpldValue::Link(c3),
        }),
    });

    let node1 = IpldNode::wrap_object(&value, Code::Sha2_256.into()).unwrap();
    assert_eq!(
        node1.cid().to_string(),
        "bafyreibgx4rjaqolj7c32c7ibxc5tedhisc4d23ihx5t4tgamuvy2hvwjm"
    );
    assert_eq!(node1.links().len(), 4);
    assert_eq!(
        node1.to_json().unwrap(),
        "{\
            \"baz\":[\
                {\"/\":\"Qma75NN8GaM99ioqsNUF9Ho816SonoGsVrvSnqz9uL4LPF\"},\
                {\"/\":\"QmUE28rcN99es8ntD4T3sBScfyrADkF3q8qzM1gEW82oMh\"}\
            ],\
            \"foo\":\"bar\",\
            \"cats\":{\
                \"qux\":{\"/\":\"QmSsM8Xm1g5WtfwkBvnttZafpDhaW6jkXeSFccECUnx7hg\"}\
            },\
            \"hello\":{\"/\":\"Qma75NN8GaM99ioqsNUF9Ho816SonoGsVrvSnqz9uL4LPF\"}\
        }"
    );

    let node2 = IpldNode::from_cbor(node1.raw_data(), Code::Sha2_256.into()).unwrap();
    assert_eq!(node1.cid(), node2.cid());

    let (link, rest) = node2.resolve_link(&["baz", "1", "bop"]).unwrap();
    assert_eq!(link.cid, c2);
    assert_eq!(rest.len(), 1);
    assert_eq!(rest[0], "bop");
}

#[test]
fn test_tree() {
    let c1 = Cid::new_v0(Code::Sha2_256.digest(b"something1").into_ext()).unwrap();
    let c2 = Cid::new_v0(Code::Sha2_256.digest(b"something2").into_ext()).unwrap();
    let c3 = Cid::new_v0(Code::Sha2_256.digest(b"something3").into_ext()).unwrap();
    let c4 = Cid::new_v0(Code::Sha2_256.digest(b"something4").into_ext()).unwrap();
    let obj = IpldValue::Map(btreemap! {
        "foo".into() => IpldValue::Link(c1),
        "baz".into() => IpldValue::List(vec![IpldValue::Link(c2), IpldValue::Link(c3), IpldValue::String("c".to_string())]),
        "cats".into() => IpldValue::Map(btreemap!{
            "qux".into() => IpldValue::Map(btreemap!{
                "boo".into() => IpldValue::Integer(1),
                "baa".into() => IpldValue::Link(c4),
                "bee".into() => IpldValue::Integer(3),
                "bii".into() => IpldValue::Integer(4),
                "buu".into() => IpldValue::Map(btreemap!{
                    "coat".into() => IpldValue::String("rain".to_string()),
                }),
            }),
        }),
    });
    let node = IpldNode::wrap_object(&obj, Code::Sha2_256.into()).unwrap();
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
            "bar": 1.0,
            "baz": 2.0
        }
    }"#;

    let node = IpldNode::from_json(json, Code::Sha2_256.into()).unwrap();
    assert_eq!(
        &node.cid().to_string(),
        "bafyreiahcy6ewqmabbh7lcjhxrillpf72zlu3vqcovckanvj2fwdtenvbe"
    );

    if let (Either::Right(obj), _) = node.resolve(&["foo"]).unwrap() {
        assert_eq!(
            serde_json::to_string(&obj).unwrap(),
            r#"{"bar":1.0,"baz":2.0}"#
        );
    } else {
        unreachable!()
    }
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
            "1.0",
            "bafyreihtx752fmf3zafbys5dtr4jxohb53yi3qtzfzf6wd5274jwtn5agu",
        ),
        (
            "[1.0]",
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
            r#"{"a":"IPFS","b":null,"c":[1.0]}"#,
            "bafyreigg2gcszayx2lywb3edqfoftyvus7gxeanmudqla3e6eh2okei25a",
        ),
        (
            r#"{"a":[]}"#,
            "bafyreian4t6wau4jdqt6nys76dfvsn6g7an4ulbv326yzutdgnrr5cjpui",
        ),
    ];
    for (json, expect_cid) in examples {
        let node = IpldNode::from_json(json, Code::Sha2_256.into()).unwrap();
        assert_eq!(node.cid().to_string(), expect_cid);
        let ser = node.to_json().unwrap();
        assert_eq!(ser, json);
    }
}

#[test]
fn test_cbor_roundtrip() {
    let node = IpldNode::from_cbor(b"`", Code::Sha2_256.into()).unwrap();
    assert_eq!(
        node.cid().to_string(),
        "bafyreiengp2sbi6ez34a2jctv34bwyjl7yoliteleaswgcwtqzrhmpyt2m"
    );
    let cbor = node.to_cbor().unwrap();
    assert_eq!(vec![b'`'], cbor)
}

const TEST_OBJ_ROOT: &str = "tests/test_objects/";

#[test]
fn test_objects() {
    let content = fs::read_to_string(format!("{}expected.json", TEST_OBJ_ROOT)).unwrap();
    let obj = serde_json::from_str::<IpldValue>(&content).unwrap();
    if let IpldValue::Map(map) = obj {
        for (key, value) in map {
            let json_file_name = format!("{}{}.json", TEST_OBJ_ROOT, key);
            let json = fs::read_to_string(json_file_name).unwrap();
            let cbor_file_name = format!("{}{}.cbor", TEST_OBJ_ROOT, key);
            let cbor = fs::read(cbor_file_name).unwrap();

            let node = IpldNode::from_json(&json, Code::Sha2_256.into()).unwrap();
            assert_eq!(node.raw_data(), cbor.as_slice());

            if let IpldValue::Link(cid) = value {
                assert_eq!(node.cid(), &cid);
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
    let node1 = IpldNode::from_cbor(&cbor, Code::Sha2_256.into()).unwrap();
    assert_ne!(node1.raw_data(), cbor.as_slice());

    assert_eq!(
        node1.cid().to_string(),
        "bafyreiawx7ona7oa2ptcoh6vwq4q6bmd7x2ibtkykld327bgb7t73ayrqm"
    );

    let node2 = IpldNode::from_cbor(node1.raw_data(), Code::Sha2_256.into()).unwrap();
    assert_eq!(node1, node2);
}

#[test]
fn test_stable_cid() {
    let cbor_file_name = format!("{}non-canon.cbor", TEST_OBJ_ROOT);
    let cbor = fs::read(cbor_file_name).unwrap();
    let cid = Cid::new_v1(Codec::DagCBOR, Code::Sha2_256.digest(&cbor).into_ext());
    let bad_block = BasicBlock::new_with_cid(cbor.into(), cid).unwrap();
    let bad_node = IpldNode::from_block(&bad_block).unwrap();
    assert_eq!(bad_block.cid(), bad_node.cid());
}

#[test]
fn test_canonical_struct_encoding() {
    use minicbor::{decode, encode, Decoder, Encoder};

    #[derive(PartialEq, Debug)]
    struct Foo {
        zebra: String,
        dog: i32,
        cats: f64,
        whale: String,
        cat: bool,
    }

    // Implement CBOR serialization for Foo.
    impl encode::Encode for Foo {
        fn encode<W: encode::Write>(
            &self,
            e: &mut Encoder<W>,
        ) -> Result<(), encode::Error<W::Error>> {
            e.map(5)?
                .str("zebra")?
                .str(&self.zebra)?
                .str("dog")?
                .i32(self.dog)?
                .str("cats")?
                .f64(self.cats)?
                .str("whale")?
                .str(&self.whale)?
                .str("cat")?
                .bool(self.cat)?
                .ok()
        }
    }

    // Implement CBOR deserialization for Foo.
    impl<'b> decode::Decode<'b> for Foo {
        fn decode(d: &mut Decoder<'b>) -> Result<Self, decode::Error> {
            let map_len = d.map()?;
            assert_eq!(map_len, Some(5));
            Ok(Foo {
                zebra: {
                    let _ = d.str()?;
                    d.str()?.to_owned()
                },
                dog: {
                    let _ = d.str()?;
                    d.i32()?
                },
                cats: {
                    let _ = d.str()?;
                    d.f64()?
                },
                whale: {
                    let _ = d.str()?;
                    d.str()?.to_owned()
                },
                cat: {
                    let _ = d.str()?;
                    d.bool()?
                },
            })
        }
    }

    let foo1 = Foo {
        zebra: "seven".to_string(),
        dog: 15,
        cats: 1.519,
        whale: "never".to_string(),
        cat: true,
    };
    let cbor = minicbor::to_vec(&foo1).unwrap();
    println!("CBOR: {:?}", cbor);
    let node = IpldNode::from_cbor(&cbor, Code::Sha2_256.into()).unwrap();
    let expect = hex::decode("a563636174f563646f670f6463617473fb3ff84dd2f1a9fbe7657768616c65656e65766572657a6562726165736576656e").unwrap();
    assert_eq!(node.raw_data().as_ref(), expect.as_slice());

    let foo2 = minicbor::decode::<Foo>(&cbor).unwrap();
    assert_eq!(foo1, foo2);
}
