/// notice, in GO version, thought it support `big.Int` cbor serialize/deserialize,
/// but in fact it only support unsigned int.
/// ```go
/// var BigIntAtlasEntry = atlas.BuildEntry(big.Int{}).Transform().
///	TransformMarshal(atlas.MakeMarshalTransformFunc(
///		func(i big.Int) ([]byte, error) {
///         /* notice `i.Bytes()` just return absolute value of x as a big-endian byte slice */
///			return i.Bytes(), nil
///		})).
///	TransformUnmarshal(atlas.MakeUnmarshalTransformFunc(
///		func(x []byte) (big.Int, error) {
///         /* when recover from slice, `big.NewInt(0)` neg is true, thus, whatever set positive
///         or negative number in `big.Int`, when recover from slice, would all be positive */
///			return *big.NewInt(0).SetBytes(x), nil
///		})).
///	Complete()
/// ```
/// Thus, we just provide `num::BigUint` to stand for `big.NewInt` in rust. if someone need real
/// `num::BigInt`, it should implement by himself.
use num_bigint::BigUint;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Error, Formatter};

#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct CborBigUint(pub BigUint);

impl Display for CborBigUint {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.0.fmt(f)
    }
}

impl Serialize for CborBigUint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Returns the byte representation of the `BigUint` in big-endian byte order
        let v = self.0.to_bytes_be();
        serializer.serialize_bytes(&v)
    }
}

impl<'de> Deserialize<'de> for CborBigUint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = Vec::<u8>::deserialize(deserializer)?;
        Ok(CborBigUint(BigUint::from_bytes_be(&v)))
    }
}
