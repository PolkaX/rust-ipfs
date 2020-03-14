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

use std::borrow::{Borrow, BorrowMut};
use std::fmt;
use std::ops::{Deref, DerefMut};

pub use num_bigint::{self, BigInt, BigUint, Sign};
pub use num_traits::{self, One, Zero};

use num_traits::{Signed, ToPrimitive};
use serde::de::Error;

/// A CBOR implementation of big int.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Hash, Default)]
pub struct CborBigUint(pub BigUint);

/// A CBOR implementation of big int.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Hash, Default)]
pub struct CborBigInt(pub BigInt);

const SIZE_UNITS: [&str; 8] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB", "ZiB"];

impl CborBigInt {
    /// print current bigint in size mod, like "0 B", "1.95 KiB" and "5 MiB", etc...
    pub fn size_str(&self) -> String {
        let mut i = 0;
        let sign = self.0.sign();
        let mut integer = self.0.abs();
        let unit = BigInt::from(1024);
        let mut decimal = Zero::zero();
        while integer >= unit && i + 1 < SIZE_UNITS.len() {
            decimal = integer.clone() & BigInt::from(1023);
            integer >>= 10;
            i += 1;
        }
        if decimal.is_zero() {
            if sign == Sign::Minus {
                format!("-{} {}", integer, SIZE_UNITS[i])
            } else {
                format!("{} {}", integer, SIZE_UNITS[i])
            }
        } else {
            let integer = integer.to_f64().unwrap();
            let part = decimal.to_f64().unwrap();
            let out = part / 1024_f64 + integer;
            if sign == Sign::Minus {
                format!("-{:0.3} {}", out, SIZE_UNITS[i])
            } else {
                format!("{:0.3} {}", out, SIZE_UNITS[i])
            }
        }
    }
}

impl fmt::Display for CborBigUint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for CborBigInt {
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

impl Deref for CborBigUint {
    type Target = BigUint;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CborBigUint {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<BigUint> for CborBigUint {
    fn as_ref(&self) -> &BigUint {
        &self.0
    }
}

impl AsMut<BigUint> for CborBigUint {
    fn as_mut(&mut self) -> &mut BigUint {
        &mut self.0
    }
}

impl Borrow<BigUint> for CborBigUint {
    fn borrow(&self) -> &BigUint {
        self.0.borrow()
    }
}

impl BorrowMut<BigUint> for CborBigUint {
    fn borrow_mut(&mut self) -> &mut BigUint {
        self.0.borrow_mut()
    }
}

impl serde::Serialize for CborBigInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Returns the byte representation of the `BigInt` in big-endian byte order
        let (sign, mut v) = self.0.to_bytes_be();
        let v = match sign {
            Sign::Plus => {
                let mut buf = Vec::with_capacity(1 + v.len());
                buf.push(0);
                buf.extend(v.iter());
                buf
            }
            Sign::Minus => {
                let mut buf = Vec::with_capacity(1 + v.len());
                buf.push(1);
                buf.extend(v.iter());
                buf
            }
            Sign::NoSign => {
                v.clear();
                v
            }
        };
        let value = serde_bytes::Bytes::new(&v);
        serializer.serialize_bytes(&value)
    }
}

impl<'de> serde::Deserialize<'de> for CborBigInt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = serde_bytes::ByteBuf::deserialize(deserializer)?;
        let v = v.into_vec();
        if v.is_empty() {
            return Ok(CborBigInt(0_u64.into()));
        }
        let sign = match &v[0] {
            0 => Sign::Plus,
            1 => Sign::Minus,
            _ => {
                return Err(D::Error::custom(format!(
                    "big int prefix should be either 0 or 1, got {}",
                    v[0]
                )))
            }
        };
        Ok(CborBigInt(BigInt::from_bytes_be(sign, &v[1..])))
    }
}

impl Deref for CborBigInt {
    type Target = BigInt;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CborBigInt {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<BigInt> for CborBigInt {
    fn as_ref(&self) -> &BigInt {
        &self.0
    }
}

impl AsMut<BigInt> for CborBigInt {
    fn as_mut(&mut self) -> &mut BigInt {
        &mut self.0
    }
}

impl Borrow<BigInt> for CborBigInt {
    fn borrow(&self) -> &BigInt {
        self.0.borrow()
    }
}

impl BorrowMut<BigInt> for CborBigInt {
    fn borrow_mut(&mut self) -> &mut BigInt {
        self.0.borrow_mut()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::str::FromStr;

    use block_format::Block;
    use cid::Cid;
    use num_bigint::BigInt;
    use serde::{Deserialize, Serialize};

    use super::{CborBigInt, CborBigUint};
    use crate::node::IpldNode;
    use crate::obj::{Obj, SortedStr};

    #[test]
    fn test_cid_and_bigint() {
        #[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
        struct Foo {
            big_int: CborBigUint,
            #[serde(with = "cid::ipld_dag_cbor")]
            cid: Cid,
        }

        let node = IpldNode::from_object(Obj::Null, multihash::Code::Sha2_256).unwrap();
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

    #[test]
    fn test_bigint_serialization_roundtrip() {
        let test_values = [
            ("0", vec![64_u8]),
            ("1", vec![66, 0, 1]),
            ("10", vec![66, 0, 10]),
            ("-10", vec![66, 1, 10]),
            ("9999", vec![67, 0, 39, 15]),
            (
                "12345678901234567891234567890123456789012345678901234567890",
                vec![
                    88, 26, 0, 1, 247, 126, 230, 227, 172, 105, 112, 56, 202, 66, 148, 9, 33, 254,
                    186, 53, 220, 190, 84, 150, 206, 63, 10, 210,
                ],
            ),
        ];
        for (s, src) in test_values.iter() {
            let expect = CborBigInt(BigInt::from_str(s).unwrap());
            let bytes = serde_cbor::to_vec(&expect).unwrap();
            assert_eq!(bytes, *src);
            let out: CborBigInt = serde_cbor::from_slice(&bytes).unwrap();
            assert_eq!(out, expect);
        }
    }

    #[test]
    fn test_size_str() {
        let cases = [
            (0_u128, "0 B"),
            (1, "1 B"),
            (1024, "1 KiB"),
            (2000, "1.953 KiB"),
            (5 << 20, "5 MiB"),
            (11 << 60, "11 EiB"),
        ];

        for (num, expect) in cases.iter() {
            let c = CborBigInt((*num).into());
            let s = c.size_str();
            println!("{:}", s);
            assert_eq!(s.as_str(), *expect);
        }

        let o = CborBigInt((-2000).into());
        assert_eq!(o.size_str().as_str(), "-1.953 KiB");
    }

    #[test]
    fn test_size_str_big() {
        let mut c = CborBigInt(50000.into());
        c.0 <<= 70;
        assert_eq!(c.size_str().as_str(), "50000 ZiB");
    }
}
