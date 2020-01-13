// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::collections::HashMap;

use rust_cid::{new_prefix_v0, new_prefix_v1, Cid, Codec, Hash, Prefix, Version};

#[test]
fn basic_marshalling() {
    let h = multihash::encode(multihash::Hash::SHA2256, b"beep boop").unwrap();

    let cid = Cid::new(Version::V1, Codec::DagProtobuf, h);

    let data = cid.to_bytes();
    let out = Cid::from(data).unwrap();

    assert_eq!(cid, out);

    let s = cid.to_string();
    let out2 = Cid::from(&s[..]).unwrap();

    assert_eq!(cid, out2);
}

#[test]
fn test_read_cids_from_buffer() {
    let cid_strs = vec![
        (
            multibase::Base::Base32Lower,
            "bafkreie5qrjvaw64n4tjm6hbnm7fnqvcssfed4whsjqxzslbd3jwhsk3mm",
        ),
        (
            multibase::Base::Base58Btc,
            "Qmf5Qzp6nGBku7CEn2UQx4mgN8TW69YUok36DrGa6NN893",
        ),
        (
            multibase::Base::Base58Btc,
            "zb2rhZi1JR4eNc2jBGaRYJKYM8JEB4ovenym8L1CmFsRAytkz",
        ),
    ];
    for (base, cs) in cid_strs {
        let cid = cs.parse::<Cid>().unwrap();
        assert_eq!(cid.to_string_by_base(base), cs)
    }
}

#[test]
fn empty_string() {
    assert!(Cid::from("").is_err());
}

#[test]
fn v0_handling() {
    let old = "QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n";
    let cid = Cid::from(old).unwrap();

    assert_eq!(cid.version(), Version::V0);
    assert_eq!(cid.to_string(), old);
}

#[test]
fn from_str() {
    let cid: Cid = "QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n"
        .parse()
        .unwrap();
    assert_eq!(cid.version(), Version::V0);

    let bad = "QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zIII".parse::<Cid>();
    assert!(bad.is_err());
}

#[test]
fn v0_error() {
    let bad = "QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zIII";
    assert!(Cid::from(bad).is_err());
}

#[test]
fn prefix_roundtrip() {
    let data = b"awesome test content";
    let h = multihash::encode(multihash::Hash::SHA2256, data).unwrap();

    let cid = Cid::new(Version::V1, Codec::DagProtobuf, h);
    let prefix = cid.prefix();

    let cid2 = Cid::new_from_prefix(&prefix, data).unwrap();

    assert_eq!(cid, cid2);

    let prefix_bytes = prefix.as_bytes();
    let prefix2 = Prefix::new_from_bytes(&prefix_bytes).unwrap();

    assert_eq!(prefix, prefix2);
}

#[test]
fn from() {
    let the_hash = "QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n";

    let cases = vec![
        format!("/ipfs/{}", &the_hash),
        format!("https://ipfs.io/ipfs/{}", &the_hash),
        format!("http://localhost:8080/ipfs/{}", &the_hash),
    ];

    for case in cases {
        let cid = Cid::from(case).unwrap();
        assert_eq!(cid.version(), Version::V0);
        assert_eq!(cid.to_string(), the_hash);
    }
}

#[test]
//#[allow(clippy::mutable_key_type)]
fn test_hash() {
    let data: Vec<u8> = vec![1, 2, 3];
    let prefix = Prefix {
        version: Version::V0,
        codec: Codec::DagProtobuf,
        mh_type: multihash::Hash::SHA2256,
        mh_len: 32,
    };
    let mut map = HashMap::new();
    let cid = Cid::new_from_prefix(&prefix, &data).unwrap();
    map.insert(cid.clone(), data.clone());
    assert_eq!(&data, map.get(&cid).unwrap());
}

#[test]
fn test_new_prefix_v0() {
    let data = b"this is some test content";
    let prefix = new_prefix_v0(Hash::SHA2256);

    // Construct c1
    let c1 = prefix.sum(data.as_ref()).unwrap();
    if c1.prefix() != prefix {
        panic!("prefix not preserved")
    }

    // Construct c2
    let mhash = multihash::encode(Hash::SHA2256, data).unwrap();
    let c2 = Cid::new_cid_v0(mhash).unwrap();
    assert_eq!(c1, c2);
    assert_eq!(c1.prefix(), c2.prefix());
}

#[test]
fn test_invalid_v0_prefix() {
    let tests = vec![
        Prefix {
            mh_type: Hash::SHA2256,
            mh_len: 31,
            version: Version::V0,
            codec: Codec::DagProtobuf,
        },
        Prefix {
            mh_type: Hash::SHA2256,
            mh_len: 33,
            version: Version::V0,
            codec: Codec::DagProtobuf,
        },
        Prefix {
            mh_type: Hash::SHA2512,
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
fn test_new_prefix_v1() {
    let data = b"this is some test content";
    let prefix = new_prefix_v1(Codec::DagCBOR, Hash::SHA2256);

    // Construct c1
    let c1 = prefix.sum(data.as_ref()).unwrap();
    if c1.prefix() != prefix {
        panic!("prefix not preserved")
    }

    // Construct c2
    let mhash = multihash::encode(Hash::SHA2256, data).unwrap();
    let c2 = Cid::new_cid_v1(Codec::DagCBOR, mhash).unwrap();
    assert_eq!(c1, c2);
    assert_eq!(c1.prefix(), c2.prefix());
}

#[test]
fn test_prefix_roundtrip() {
    let data = b"this is some test content";
    let mhash = multihash::encode(Hash::SHA2256, data.as_ref()).unwrap();
    let c1 = Cid::new_cid_v1(Codec::DagCBOR, mhash).unwrap();

    let prefix1 = c1.prefix();
    let c2 = prefix1.sum(data.as_ref()).unwrap();
    assert_eq!(c1, c2);

    let prefix2 = Prefix::new_from_bytes(&prefix1.as_bytes()).unwrap();
    assert_eq!(prefix1, prefix2)
}
