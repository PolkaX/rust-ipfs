// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

// notice, in GO version, thought it support `big.Int` cbor serialize/deserialize,
// but in fact it only support unsigned int.
// ```go
// var BigIntAtlasEntry = atlas.BuildEntry(big.Int{}).Transform().
//	TransformMarshal(atlas.MakeMarshalTransformFunc(
//		func(i big.Int) ([]byte, error) {
//         /* notice `i.Bytes()` just return absolute value of x as a big-endian byte slice */
//			return i.Bytes(), nil
//		})).
//	TransformUnmarshal(atlas.MakeUnmarshalTransformFunc(
//		func(x []byte) (big.Int, error) {
//         /* when recover from slice, `big.NewInt(0)` neg is true, thus, whatever set positive
//         or negative number in `big.Int`, when recover from slice, would all be positive */
//			return *big.NewInt(0).SetBytes(x), nil
//		})).
//	Complete()
// ```
// Thus, we just provide `num::BigUint` to stand for `big.NewInt` in rust. if someone need real
// `num::BigInt`, it should implement by himself.

use std::fmt;

use num_bigint::BigUint;

/// A CBOR implementation of big int.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Hash)]
pub struct CborBigUint(pub BigUint);

impl fmt::Display for CborBigUint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl serde::Serialize for CborBigUint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Returns the byte representation of the `BigUint` in big-endian byte order
        let v = self.0.to_bytes_be();
        let value = serde_bytes::Bytes::new(&v);
        serializer.serialize_bytes(&value)
    }
}

impl<'de> serde::Deserialize<'de> for CborBigUint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = serde_bytes::ByteBuf::deserialize(deserializer)?;
        Ok(CborBigUint(BigUint::from_bytes_be(&v)))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde::{Deserialize, Serialize};

    use cid::{AsCidRef, Cid};
    use multihash::Hash;

    use super::CborBigUint;
    use crate::node::IpldNode;
    use crate::obj::{Obj, SortedStr};

    #[test]
    fn test_cid_and_bigint() {
        #[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
        struct Foo {
            big_int: CborBigUint,
            cid: Cid,
        }

        let node = IpldNode::from_object(Obj::Null, Hash::SHA2256).unwrap();
        let cid = node.cid().clone();
        let foo1 = Foo {
            big_int: CborBigUint(1_u64.into()),
            cid,
        };

        let bytes = serde_cbor::to_vec(&foo1).unwrap();
        let foo2 = serde_cbor::from_slice::<Foo>(&bytes).unwrap();
        assert_eq!(foo1, foo2);
    }

    #[test]
    fn test_bigint_roundtrip() {
        #[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
        struct TestMe {
            hello: CborBigUint,
            world: CborBigUint,
            hi: i32,
        }
        let me1 = TestMe {
            hello: CborBigUint(100_u64.into()),
            world: CborBigUint(99_u64.into()),
            hi: 0,
        };
        let bytes = serde_cbor::to_vec(&me1).unwrap();
        let me2 = serde_cbor::from_slice::<TestMe>(&bytes).unwrap();
        assert_eq!(me1, me2);

        type M = BTreeMap<SortedStr, TestMe>;
        let mut m1 = M::new();
        m1.insert(
            "hello".into(),
            TestMe {
                hello: CborBigUint(10_u64.into()),
                world: CborBigUint(101_u64.into()),
                hi: 1,
            },
        );
        m1.insert(
            "world".into(),
            TestMe {
                hello: CborBigUint(9_u64.into()),
                world: CborBigUint(901_u64.into()),
                hi: 3,
            },
        );
        let bytes = serde_cbor::to_vec(&m1).unwrap();
        let m2: M = serde_cbor::from_slice(&bytes).unwrap();
        assert_eq!(m1, m2);
    }
}
