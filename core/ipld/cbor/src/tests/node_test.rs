use super::*;
use std::collections::BTreeMap;

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
        panic!("");
    }
}

#[test]
fn test_basic_marshal() {
    let c = Cid::new_cid_v0(util::hash(b"something")).unwrap();
    let mut m = BTreeMap::new();
    m.insert("name".to_string().into(), Obj::Text("foo".to_string()));
    m.insert("bar".to_string().into(), Obj::Cid(c.clone().into()));
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

    let (lnk, _) = back.resolve_link(&vec!["bar".to_string()]).unwrap();
    assert_eq!(lnk.cid, c);
    assert_eq!(nd.cid(), back.cid())
}

#[test]
fn test_marshal_roundtrip() {
    let c1 = Cid::new_cid_v0(util::hash(b"something1")).unwrap();
    let c2 = Cid::new_cid_v0(util::hash(b"something2")).unwrap();
    let c3 = Cid::new_cid_v0(util::hash(b"something3")).unwrap();

    let mut m = BTreeMap::new();
    m.insert("foo".to_string().into(), Obj::Text("bar".to_string()));
    m.insert("hello".to_string().into(), Obj::Cid(c1.clone().into()));
    m.insert(
        "baz".to_string().into(),
        Obj::Array(vec![Obj::Cid(c1.into()), Obj::Cid(c2.into())]),
    );
    let mut cats_m = BTreeMap::new();
    cats_m.insert("qux".to_string().into(), Obj::Cid(c3.into()));
    m.insert("cats".to_string().into(), Obj::Map(cats_m));
    let obj = Obj::Map(m);

    let nd1 = wrap_obj(obj, MHashEnum::SHA2256).unwrap();
    assert_eq!(
        &nd1.cid().to_string(),
        "bafyreibgx4rjaqolj7c32c7ibxc5tedhisc4d23ihx5t4tgamuvy2hvwjm"
    );
}
