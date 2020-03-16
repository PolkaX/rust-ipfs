// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fmt;

use cid::{Cid, IPLD_DAG_CBOR_TAG_CID, RAW_BINARY_MULTIBASE_IDENTITY};
use serde::{de, ser, Deserialize, Serialize};
use serde_cbor::{tags::current_cbor_tag, Value};

use crate::error::IpldCborError;

/// A String Wrapper that implements `Ord` and `PartialOrd`,
/// according to the length of string, in bytes.
#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct SortedStr(String);

impl SortedStr {
    /// Convert to inner string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Ord for SortedStr {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.len() != other.len() {
            self.0.len().cmp(&other.0.len())
        } else {
            self.0.cmp(&other.0)
        }
    }
}

impl PartialOrd for SortedStr {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for SortedStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::borrow::Borrow<str> for SortedStr {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl std::borrow::BorrowMut<str> for SortedStr {
    fn borrow_mut(&mut self) -> &mut str {
        &mut self.0
    }
}

impl AsRef<str> for SortedStr {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsMut<str> for SortedStr {
    fn as_mut(&mut self) -> &mut str {
        &mut self.0
    }
}

impl std::ops::Deref for SortedStr {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for SortedStr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<String> for SortedStr {
    fn from(s: String) -> Self {
        SortedStr(s)
    }
}

impl From<&str> for SortedStr {
    fn from(s: &str) -> Self {
        SortedStr(s.to_string())
    }
}

/// A special CBOR value.
#[derive(PartialEq, Clone, Debug)]
pub enum Obj {
    /// CBOR null value.
    Null,
    /// CBOR bool value.
    Bool(bool),
    /// CBOR integer value.
    Integer(i128),
    /// CBOR float value.
    Float(f64),
    /// CBOR byte string value.
    Bytes(Vec<u8>),
    /// CBOR text string value.
    Text(String),
    /// CBOR array value.
    Array(Vec<Obj>),
    /// CBOR map value.
    Map(BTreeMap<SortedStr, Obj>),
    /// CBOR tag value (tag is 42).
    Cid(Cid),
}

impl ser::Serialize for Obj {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match *self {
            Obj::Null => serializer.serialize_unit(),
            Obj::Bool(v) => serializer.serialize_bool(v),
            Obj::Integer(v) => serializer.serialize_i128(v),
            Obj::Float(v) => serializer.serialize_f64(v),
            Obj::Bytes(ref v) => serializer.serialize_bytes(&v),
            Obj::Text(ref v) => serializer.serialize_str(&v),
            Obj::Array(ref v) => v.serialize(serializer),
            Obj::Map(ref v) => v.serialize(serializer),
            Obj::Cid(ref v) => cid::ipld_dag_cbor::serialize(v, serializer),
        }
    }
}

impl<'de> de::Deserialize<'de> for Obj {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> de::Visitor<'de> for ValueVisitor {
            type Value = Obj;

            fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt.write_str("any valid CBOR value")
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_string(String::from(value))
            }

            #[inline]
            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Obj::Text(value))
            }
            #[inline]
            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_byte_buf(v.to_owned())
            }

            #[inline]
            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Obj::Bytes(v))
            }

            #[inline]
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Obj::Integer(v.into()))
            }

            #[inline]
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Obj::Integer(v.into()))
            }

            #[inline]
            fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Obj::Integer(v))
            }

            #[inline]
            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Obj::Bool(v))
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_unit()
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Obj::Null)
            }

            #[inline]
            fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: de::SeqAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(elem) = visitor.next_element()? {
                    vec.push(elem);
                }

                Ok(Obj::Array(vec))
            }

            #[inline]
            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut values = BTreeMap::new();

                while let Some((key, value)) = visitor.next_entry()? {
                    values.insert(key, value);
                }

                Ok(Obj::Map(values))
            }

            #[inline]
            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Obj::Float(v))
            }

            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                match current_cbor_tag() {
                    Some(IPLD_DAG_CBOR_TAG_CID) | None => {
                        let cid = cid::ipld_dag_cbor::deserialize(deserializer)?;
                        Ok(Obj::Cid(cid))
                    }
                    Some(tag) => Err(de::Error::custom(format!("unexpected tag ({})", tag))),
                }
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

impl TryFrom<serde_cbor::Value> for Obj {
    type Error = IpldCborError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Null => Ok(Obj::Null),
            Value::Bool(b) => Ok(Obj::Bool(b)),
            Value::Integer(i) => Ok(Obj::Integer(i)),
            Value::Float(f) => Ok(Obj::Float(f)),
            Value::Bytes(b) => Ok(Obj::Bytes(b)),
            Value::Text(s) => Ok(Obj::Text(s)),
            Value::Array(arr) => try_from_array(arr),
            Value::Map(map) => try_from_map(map),
            Value::Tag(tag, value) => try_from_tag(tag, *value),
            _ => unreachable!("not impl for the hidden variant"),
        }
    }
}

fn try_from_array(arr: Vec<Value>) -> Result<Obj, IpldCborError> {
    let mut v = Vec::with_capacity(arr.len());
    for i in arr {
        let obj = Obj::try_from(i)?;
        v.push(obj);
    }
    Ok(Obj::Array(v))
}

fn try_from_map(map: BTreeMap<Value, Value>) -> Result<Obj, IpldCborError> {
    let mut m = BTreeMap::new();
    for (k, v) in map {
        if let Value::Text(key) = k {
            m.insert(key.into(), Obj::try_from(v)?);
        } else {
            return Err(IpldCborError::ObjErr("map key must be string".to_string()));
        }
    }
    Ok(Obj::Map(m))
}

fn try_from_tag(tag: u64, value: Value) -> Result<Obj, IpldCborError> {
    if tag != IPLD_DAG_CBOR_TAG_CID {
        return Err(IpldCborError::ObjErr(format!(
            "obj only accept tag [{}] to represent cid",
            IPLD_DAG_CBOR_TAG_CID
        )));
    }

    if let Value::Bytes(ref bytes) = value {
        if bytes.is_empty() || bytes[0] != RAW_BINARY_MULTIBASE_IDENTITY {
            return Err(IpldCborError::ObjErr(format!(
                "raw binary multibase identity [{}] must not be omitted",
                RAW_BINARY_MULTIBASE_IDENTITY,
            )));
        }
        let cid = Cid::try_from(&bytes[1..])?;
        Ok(Obj::Cid(cid))
    } else {
        Err(IpldCborError::ObjErr(format!(
            "tag [{}] value must be bytes",
            IPLD_DAG_CBOR_TAG_CID
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::SortedStr;

    #[test]
    fn test_sorted_str_ord() {
        let sort_s1 = SortedStr::from("abc");
        let sort_s2 = SortedStr::from("bcd");
        let sort_s3 = SortedStr::from("abcd");
        assert!(sort_s1 < sort_s2);
        assert!(sort_s1 < sort_s3);
        assert!(sort_s2 < sort_s3);

        let s1 = sort_s1.into_inner();
        let s2 = sort_s2.into_inner();
        let s3 = sort_s3.into_inner();
        assert!(s1 < s2);
        assert!(s1 < s3);
        assert!(s2 > s3);
    }
}
