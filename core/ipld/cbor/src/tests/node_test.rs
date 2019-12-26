use super::*;
use crate::obj::SortedStr;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fs;

#[test]
fn test_non_obj() {
    let nd = wrap_obj(Obj::Text("".to_string()), MHashEnum::SHA2256).unwrap();
    assert_eq!(
        &nd.cid().to_string(),
        "bafyreiengp2sbi6ez34a2jctv34bwyjl7yoliteleaswgcwtqzrhmpyt2m"
    );

    // decode from cbor bytes to a node
    let back = decode(nd.raw_data(), MHashEnum::SHA2256).unwrap();
    assert_eq!(
        &back.cid().to_string(),
        "bafyreiengp2sbi6ez34a2jctv34bwyjl7yoliteleaswgcwtqzrhmpyt2m"
    );
}

#[test]
fn test_decode_into() {
    let mut m = BTreeMap::new();
    m.insert("name".to_string().into(), Obj::Text("foo".to_string()));
    let obj = Obj::Map(m);
    let nd = wrap_obj(obj, MHashEnum::SHA2256).unwrap();
    // `serde_cbor::from_slice` equal to `DecodeInto`
    let o: Obj = serde_cbor::from_slice(nd.raw_data()).unwrap();
    assert_eq!(o, nd.obj);
}

#[test]
fn test_decode_into_non_obj() {
    let nd = wrap_obj(Obj::Text("foobar".to_string()), MHashEnum::SHA2256).unwrap();
    let o: Obj = serde_cbor::from_slice(nd.raw_data()).unwrap();
    if let Obj::Text(ref s) = o {
        assert_eq!(s, "foobar");
    } else {
        unreachable!();
    }
}

#[test]
fn test_basic_marshal() {
    let c = Cid::new_cid_v0(util::sha2_256_hash(b"something")).unwrap();
    let mut m = BTreeMap::new();
    m.insert("name".to_string().into(), Obj::Text("foo".to_string()));
    m.insert("bar".to_string().into(), Obj::Cid(c.clone()));
    let obj = Obj::Map(m);

    let raw = dump_object(&obj).unwrap();

    let nd = wrap_obj(obj, MHashEnum::SHA2256).unwrap();
    assert_eq!(
        &nd.cid().to_string(),
        "bafyreib4hmpkwa7zyzoxmpwykof6k7akxnvmsn23oiubsey4e2tf6gqlui"
    );

    let back = decode(nd.raw_data(), MHashEnum::SHA2256).unwrap();
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
    m.insert("hello".to_string().into(), Obj::Cid(c1.clone()));
    m.insert(
        "baz".to_string().into(),
        Obj::Array(vec![Obj::Cid(c1), Obj::Cid(c2.clone())]),
    );
    let mut cats_m = BTreeMap::new();
    cats_m.insert("qux".to_string().into(), Obj::Cid(c3));
    m.insert("cats".to_string().into(), Obj::Map(cats_m));
    let obj = Obj::Map(m);

    let nd1 = wrap_obj(obj, MHashEnum::SHA2256).unwrap();
    assert_eq!(
        &nd1.cid().to_string(),
        "bafyreibgx4rjaqolj7c32c7ibxc5tedhisc4d23ihx5t4tgamuvy2hvwjm"
    );

    assert_eq!(nd1.links().len(), 4_usize);

    let nd2 = decode(nd1.raw_data(), MHashEnum::SHA2256).unwrap();
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

#[test]
fn test_tree() {
    let c1 = Cid::new_cid_v0(util::sha2_256_hash(b"something1")).unwrap();
    let c2 = Cid::new_cid_v0(util::sha2_256_hash(b"something2")).unwrap();
    let c3 = Cid::new_cid_v0(util::sha2_256_hash(b"something3")).unwrap();
    let c4 = Cid::new_cid_v0(util::sha2_256_hash(b"something4")).unwrap();

    let mut obj_m = BTreeMap::new();
    obj_m.insert("foo".into(), Obj::Cid(c1));
    obj_m.insert(
        "baz".into(),
        Obj::Array(vec![Obj::Cid(c2), Obj::Cid(c3), Obj::Text("c".to_string())]),
    );
    let mut cats_m = BTreeMap::new();
    {
        let mut qux_m = BTreeMap::new();
        {
            qux_m.insert("boo".into(), Obj::Integer(1));
            qux_m.insert("baa".into(), Obj::Cid(c4));
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

    let nd = wrap_obj(obj, MHashEnum::SHA2256).unwrap();

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

#[test]
fn test_parsing() {
    // skip
}

#[test]
fn test_from_json() {
    let data = r#"{
        "something": {"/":"bafkreifvxooyaffa7gy5mhrb46lnpdom34jvf4r42mubf5efbodyvzeujq"},
        "cats": "not cats",
        "cheese": [
                {"/":"bafkreifvxooyaffa7gy5mhrb46lnpdom34jvf4r42mubf5efbodyvzeujq"},
                {"/":"bafkreifvxooyaffa7gy5mhrb46lnpdom34jvf4r42mubf5efbodyvzeujq"},
                {"/":"bafkreifvxooyaffa7gy5mhrb46lnpdom34jvf4r42mubf5efbodyvzeujq"},
                {"/":"bafkreifvxooyaffa7gy5mhrb46lnpdom34jvf4r42mubf5efbodyvzeujq"}
        ]
    }"#;
    let n = from_json(data, MHashEnum::SHA2256).unwrap();
    assert_eq!(
        &n.cid().to_string(),
        "bafyreicnokmhmrnlp2wjhyk2haep4tqxiptwfrp2rrs7rzq7uk766chqvq"
    );
    match n.obj {
        Obj::Map(m) => {
            let v = m.get::<str>("something").unwrap();
            match v {
                Obj::Cid(cid) => {
                    assert_eq!(
                        &cid.to_string(),
                        "bafkreifvxooyaffa7gy5mhrb46lnpdom34jvf4r42mubf5efbodyvzeujq"
                    );
                }
                _ => panic!("must be cid"),
            }
        }
        _ => panic!("must be map"),
    }
}

#[test]
fn test_resolved_val_is_jsonable() {
    let data = r#"{
		"foo": {
			"bar": 1,
			"baz": 2
		}
	}"#;

    let n = from_json(data, MHashEnum::SHA2256).unwrap();
    assert_eq!(
        &n.cid().to_string(),
        "bafyreiahcy6ewqmabbh7lcjhxrillpf72zlu3vqcovckanvj2fwdtenvbe"
    );
    let (val, _) = n.resolve(&vec!["foo"]).unwrap();
    match val {
        Either::Left(l) => panic!(""),
        Either::Right(obj) => {
            let s = obj_to_json(obj).unwrap();
            assert_eq!(&s, r#"{"bar":1,"baz":2}"#)
        }
    }
}

#[test]
fn test_examples() {
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
    for (json, expcid) in examples {
        let n = from_json(json, MHashEnum::SHA2256).unwrap();
        assert_eq!(&n.cid().to_string(), expcid);

        let cbor_raw = n.raw_data();
        let node = decode(cbor_raw, MHashEnum::SHA2256).unwrap();

        let s = to_json(&node).unwrap();

        assert_eq!(json, &s);
    }
}

const TEST_OBJ_ROOT: &'static str = "src/tests/test_objects/";

#[test]
fn test_objects() {
    let content = fs::read_to_string(format!("{}{}", TEST_OBJ_ROOT, "expected.json")).unwrap();
    let obj = json_to_obj(&content).unwrap();
    match obj {
        Obj::Map(m) => {
            for (k, v) in m {
                let json = fs::read_to_string(format!("{}{}.json", TEST_OBJ_ROOT, k.0)).unwrap();
                let expected = fs::read(format!("{}{}.cbor", TEST_OBJ_ROOT, k.0)).unwrap();

                let nd = from_json(&json, MHashEnum::SHA2256).unwrap();
                assert_eq!(nd.raw_data(), expected.as_slice());
                match v {
                    Obj::Cid(cid) => {
                        assert_eq!(nd.cid(), &cid);
                    }
                    _ => unreachable!(),
                }
            }
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_canonicalize() {
    let b = fs::read(format!("{}non-canon.cbor", TEST_OBJ_ROOT)).unwrap();
    let nd1 = decode(&b, MHashEnum::SHA2256).unwrap();

    assert_ne!(nd1.raw_data().as_ref(), b.as_slice());
    assert_eq!(
        &nd1.cid().to_string(),
        "bafyreiawx7ona7oa2ptcoh6vwq4q6bmd7x2ibtkykld327bgb7t73ayrqm"
    );

    let nd2 = decode(nd1.raw_data(), MHashEnum::SHA2256).unwrap();
    assert_eq!(nd1, nd2);
}

#[test]
fn test_stable_cid() {
    let b = fs::read(format!("{}non-canon.cbor", TEST_OBJ_ROOT)).unwrap();
    let hash = util::sha2_256_hash(&b);
    let c = Cid::new_cid_v1(Codec::DagCBOR, hash).unwrap();
    let bad_block = BasicBlock::new_with_cid(b.into(), c).unwrap();
    let bad_node = decode_block(&bad_block).unwrap();
    assert_eq!(bad_block.cid(), bad_node.cid());
}

#[cfg(feature = "bigint")]
#[test]
fn test_cid_and_bigint() {
    #[derive(Serialize, Deserialize, Debug)]
    struct Foo {
        b: CborBigUint,
        a: Cid,
    }
    let nd = wrap_obj(Obj::Text("".to_string()), MHashEnum::SHA2256).unwrap();
    let c = nd.cid().clone();

    let foo = Foo {
        a: c,
        b: CborBigUint(1_u64.into()),
    };

    let value = struct_to_cbor_value(&foo).unwrap();
    println!("{:?}", value);

    let obj = Obj::try_from(value).unwrap();
    wrap_obj(obj, MHashEnum::SHA2256).unwrap();
}

#[test]
fn test_empty_cid() {
    #[derive(Serialize, Deserialize, Debug)]
    struct Foo {
        a: Cid,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Bar {
        #[serde(skip)]
        a: Option<Cid>,
    }

    let foo = Foo {
        a: Cid::new_cid_v0(util::sha2_256_hash(b"")).unwrap(),
    };
    let bar = Bar { a: None };

    let value = struct_to_cbor_value(&foo).unwrap();
    println!("{:?}", value);

    let obj = Obj::try_from(value).unwrap();
    wrap_obj(obj, MHashEnum::SHA2256).unwrap();

    let value = struct_to_cbor_value(&bar).unwrap();
    println!("{:?}", value);
    let obj = Obj::try_from(value).unwrap();
    let node = wrap_obj(obj, MHashEnum::SHA2256).unwrap();
    assert_eq!(
        &node.cid().to_string(),
        "bafyreigbtj4x7ip5legnfznufuopl4sg4knzc2cof6duas4b3q2fy6swua"
    )
}

#[test]
fn test_canonical_struct_encoding() {
    use hex_literal::hex;

    #[derive(Serialize, Deserialize, Debug)]
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
    let v = serde_cbor::value::to_value(foo).unwrap();
    let obj = Obj::try_from(v).unwrap();
    println!("{:?}", obj);
    let m = wrap_obj(obj, MHashEnum::SHA2256).unwrap();

    let expraw = hex!("a563636174f563646f670f6463617473fb3ff84dd2f1a9fbe7657768616c65656e65766572657a6562726165736576656e");
    assert_eq!(m.raw_data(), expraw.as_ref());
}

#[cfg(feature = "bigint")]
#[test]
fn test_bigint_roundtrip() {
    #[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
    struct TestMe {
        hello: CborBigUint,
        world: CborBigUint,
        hi: i32,
    }
    let me = TestMe {
        hello: CborBigUint(100_u64.into()),
        world: CborBigUint(99_u64.into()),
        hi: 0,
    };
    let v = dump_object(&me).unwrap();
    let obj: TestMe = decode_into(&v).unwrap();
    assert_eq!(me, obj);

    type M = BTreeMap<SortedStr, TestMe>;
    let mut m = M::new();
    m.insert(
        "hello".into(),
        TestMe {
            hello: CborBigUint(10_u64.into()),
            world: CborBigUint(101_u64.into()),
            hi: 1,
        },
    );
    m.insert(
        "world".into(),
        TestMe {
            hello: CborBigUint(9_u64.into()),
            world: CborBigUint(901_u64.into()),
            hi: 3,
        },
    );
    let bytes = dump_object(&m).unwrap();
    let m2: M = decode_into(&bytes).unwrap();
    assert_eq!(m, m2);
}
