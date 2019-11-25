use std::collections::HashMap;

use rust_cid::{new_cid_v0, new_cid_v1, Cid, Codec, Error, MHashEnum, Prefix, Version};

#[test]
fn basic_marshalling() {
    let h = multihash::encode(multihash::Hash::SHA2256, b"beep boop").unwrap();

    let cid = Cid::new(Codec::DagProtobuf, Version::V1, h.as_bytes());

    let data = cid.to_bytes();
    let out = Cid::from(data).unwrap();

    assert_eq!(cid, out);

    let s = cid.to_string();
    let out2 = Cid::from(&s[..]).unwrap();

    assert_eq!(cid, out2);
}

#[test]
fn empty_string() {
    assert_eq!(Cid::from(""), Err(Error::InputTooShort));
}

#[test]
fn v0_handling() {
    let old = "QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n";
    let cid = Cid::from(old).unwrap();

    assert_eq!(cid.version, Version::V0);
    assert_eq!(cid.to_string(), old);
}

#[test]
fn from_str() {
    let cid: Cid = "QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n"
        .parse()
        .unwrap();
    assert_eq!(cid.version, Version::V0);

    let bad = "QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zIII".parse::<Cid>();
    assert_eq!(
        bad,
        Err(Error::ParsingError(format!(
            "Multibase, reason: Invalid base string"
        )))
    );
}

#[test]
fn v0_error() {
    let bad = "QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zIII";
    assert_eq!(
        Cid::from(bad),
        Err(Error::ParsingError(format!(
            "Multibase, reason: Invalid base string"
        )))
    );
}

#[test]
fn prefix_roundtrip() {
    let data = b"awesome test content";
    let h = multihash::encode(multihash::Hash::SHA2256, data).unwrap();

    let cid = Cid::new(Codec::DagProtobuf, Version::V1, h.as_bytes());
    let prefix = cid.prefix();

    let cid2 = Cid::new_from_prefix(&prefix, data);

    assert_eq!(cid, cid2);

    let prefix_bytes = prefix.as_bytes();
    let prefix2 = Prefix::new_from_bytes(&prefix_bytes).unwrap();

    assert_eq!(prefix, prefix2);
}

#[test]
fn from() {
    let the_hash = "QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n";

    let cases = vec![
        format!("/ipfs/{:}", &the_hash),
        format!("https://ipfs.io/ipfs/{:}", &the_hash),
        format!("http://localhost:8080/ipfs/{:}", &the_hash),
    ];

    for case in cases {
        let cid = Cid::from(case).unwrap();
        assert_eq!(cid.version, Version::V0);
        assert_eq!(cid.to_string(), the_hash);
    }
}

#[test]
fn test_hash() {
    let data: Vec<u8> = vec![1, 2, 3];
    let prefix = Prefix {
        version: Version::V0,
        codec: Codec::DagProtobuf,
        mh_type: multihash::Hash::SHA2256,
        mh_len: 32,
    };
    let mut map = HashMap::new();
    let cid = Cid::new_from_prefix(&prefix, &data);
    map.insert(cid.clone(), data.clone());
    assert_eq!(&data, map.get(&cid).unwrap());
}

fn new_prefix_v0(hash: MHashEnum) -> Prefix {
    Prefix {
        version: Version::V0,
        codec: Codec::DagProtobuf,
        mh_type: hash,
        mh_len: hash.size() as usize,
    }
}

fn new_prefix_v1(codec: Codec, hash: MHashEnum) -> Prefix {
    Prefix {
        version: Version::V1,
        codec,
        mh_type: hash,
        mh_len: hash.size() as usize,
    }
}

#[test]
fn test_new_prefix_v0() {
    let data = b"this is some test content";
    let prefix = new_prefix_v0(MHashEnum::SHA2256);

    // Construct c1
    let c1 = prefix.sum(data.as_ref()).unwrap();
    if c1.prefix() != prefix {
        panic!("prefix not preserved")
    }

    // Construct c2
    let mhash = multihash::encode(MHashEnum::SHA2256, data).unwrap();
    let c2 = new_cid_v0(mhash).unwrap();
    assert_eq!(c1, c2);
    assert_eq!(c1.prefix(), c2.prefix());
}

#[test]
fn test_invalid_v0_prefix() {
    let tests = vec![
        Prefix {
            mh_type: MHashEnum::SHA2256,
            mh_len: 31,
            version: Version::V0,
            codec: Codec::DagProtobuf,
        },
        Prefix {
            mh_type: MHashEnum::SHA2256,
            mh_len: 33,
            version: Version::V0,
            codec: Codec::DagProtobuf,
        },
        Prefix {
            mh_type: MHashEnum::SHA2512,
            mh_len: 32,
            version: Version::V0,
            codec: Codec::DagProtobuf,
        },
    ];
    for p in tests {
        let r = p.sum(b"testdata");
        assert!(r.is_err());
    }
}

#[test]
fn test_prefix_roundtrip() {
    let data = b"this is some test content";
    let mhash = multihash::encode(MHashEnum::SHA2256, data.as_ref()).unwrap();
    let c1 = new_cid_v1(Codec::DagCBOR, mhash).unwrap();

    let prefix = c1.prefix();
    let c2 = prefix.sum(data.as_ref()).unwrap();
    assert_eq!(c1, c2);

    let prefix_bytes = prefix.as_bytes();

    let prefix2 = Prefix::new_from_bytes(prefix_bytes.as_slice()).unwrap();

    assert_eq!(prefix, prefix2)
}