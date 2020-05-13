// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fmt;

use cid::Cid;
use minicbor::{data::Type, decode, encode, Decoder, Encoder};
use serde::{
    de,
    ser::{self, SerializeMap, SerializeSeq},
    Deserialize, Serialize,
};

/// A String Wrapper that implements `Ord` and `PartialOrd`,
/// according to the length of string, in bytes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

// Implement CBOR serialization for SortedStr.
impl encode::Encode for SortedStr {
    fn encode<W: encode::Write>(&self, e: &mut Encoder<W>) -> Result<(), encode::Error<W::Error>> {
        e.str(&self.0)?.ok()
    }
}

// Implement CBOR deserialization for SortedStr.
impl<'b> decode::Decode<'b> for SortedStr {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, decode::Error> {
        let s = d.str()?.to_owned();
        Ok(SortedStr(s))
    }
}

/// The IPLD value.
#[derive(Clone, Debug, PartialEq)]
pub enum IpldValue {
    /// null value.
    Null,
    /// bool value.
    Bool(bool),
    /// integer value.
    Integer(i128),
    /// float value.
    Float(f64),
    /// UTF-8 text string value.
    String(String),
    /// byte string value.
    Bytes(Vec<u8>),
    /// list value.
    List(Vec<IpldValue>),
    /// map value.
    Map(BTreeMap<SortedStr, IpldValue>),
    /// link value.
    Link(Cid),
}

// Implement CBOR serialization for IpldValue.
impl encode::Encode for IpldValue {
    fn encode<W: encode::Write>(&self, e: &mut Encoder<W>) -> Result<(), encode::Error<W::Error>> {
        match self {
            IpldValue::Null => e.null()?.ok(),
            IpldValue::Bool(bool) => e.bool(*bool)?.ok(),
            IpldValue::Integer(i128) => e.i64(*i128 as i64)?.ok(),
            IpldValue::Float(f64) => e.f64(*f64)?.ok(),
            IpldValue::Bytes(bytes) => e.bytes(bytes)?.ok(),
            IpldValue::String(string) => e.str(string)?.ok(),
            IpldValue::List(list) => {
                let e = e.array(list.len() as u64)?;
                for value in list {
                    e.encode(value)?;
                }
                e.ok()
            }
            IpldValue::Map(map) => {
                let e = e.map(map.len() as u64)?;
                for (key, value) in map {
                    e.encode(key)?.encode(value)?;
                }
                e.ok()
            }
            IpldValue::Link(cid) => e.encode(cid)?.ok(),
        }
    }
}

// Implement CBOR deserialization for IpldValue.
impl<'b> decode::Decode<'b> for IpldValue {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, decode::Error> {
        match d.datatype()? {
            Type::Null => {
                d.skip()?;
                Ok(IpldValue::Null)
            }
            Type::Bool => Ok(IpldValue::Bool(d.bool()?)),
            Type::U8 => Ok(IpldValue::Integer(i128::from(d.u8()?))),
            Type::U16 => Ok(IpldValue::Integer(i128::from(d.u16()?))),
            Type::U32 => Ok(IpldValue::Integer(i128::from(d.u32()?))),
            Type::U64 => Ok(IpldValue::Integer(i128::from(d.u32()?))),
            Type::I8 => Ok(IpldValue::Integer(i128::from(d.i8()?))),
            Type::I16 => Ok(IpldValue::Integer(i128::from(d.i16()?))),
            Type::I32 => Ok(IpldValue::Integer(i128::from(d.i32()?))),
            Type::I64 => Ok(IpldValue::Integer(i128::from(d.i64()?))),
            Type::F16 => Ok(IpldValue::Float(f64::from(d.f16()?))),
            Type::F32 => Ok(IpldValue::Float(f64::from(d.f32()?))),
            Type::F64 => Ok(IpldValue::Float(d.f64()?)),
            Type::Bytes => Ok(IpldValue::Bytes(d.bytes()?.to_vec())),
            Type::String => Ok(IpldValue::String(d.str()?.to_owned())),
            Type::Array => {
                let array_len = d.array()?.expect("array is definite");
                let mut array = Vec::with_capacity(array_len as usize);
                for _ in 0..array_len {
                    let obj = d.decode::<IpldValue>()?;
                    array.push(obj);
                }
                Ok(IpldValue::List(array))
            }
            Type::Map => {
                let map_len = d.map()?.expect("map is definite");
                let mut map = BTreeMap::new();
                for _ in 0..map_len {
                    let k = d.decode::<SortedStr>()?;
                    let v = d.decode::<IpldValue>()?;
                    map.insert(k, v);
                }
                Ok(IpldValue::Map(map))
            }
            Type::Tag => {
                let cid = d.decode::<Cid>()?;
                Ok(IpldValue::Link(cid))
            }
            Type::Break | Type::Unknown(_) | Type::Undefined | Type::Simple => {
                Err(decode::Error::Message("unexpected type"))
            }
        }
    }
}

// Implement JSON serialization for IpldValue.
impl ser::Serialize for IpldValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match self {
            IpldValue::Null => serializer.serialize_none(),
            IpldValue::Bool(bool) => serializer.serialize_bool(*bool),
            IpldValue::Integer(i128) => {
                if *i128 > 0 {
                    serializer.serialize_u64(*i128 as u64)
                } else {
                    serializer.serialize_i64(*i128 as i64)
                }
            }
            IpldValue::Float(f64) => serializer.serialize_f64(*f64),
            IpldValue::String(string) => serializer.serialize_str(string),
            IpldValue::Bytes(bytes) => serializer.serialize_bytes(bytes),
            IpldValue::List(list) => {
                let mut seq = serializer.serialize_seq(Some(list.len()))?;
                for element in list {
                    seq.serialize_element(element)?;
                }
                seq.end()
            }
            IpldValue::Map(map) => {
                let mut m = serializer.serialize_map(Some(map.len()))?;
                for (k, v) in map {
                    m.serialize_entry(k, v)?;
                }
                m.end()
            }
            IpldValue::Link(link) => link.serialize(serializer),
        }
    }
}

// Implement JSON deserialization for IpldValue.
impl<'de> de::Deserialize<'de> for IpldValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_any(JsonVisitor)
    }
}

struct JsonVisitor;
impl<'de> de::Visitor<'de> for JsonVisitor {
    type Value = IpldValue;

    fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str("any valid JSON value")
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
        Ok(IpldValue::String(value))
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
        Ok(IpldValue::Bytes(v))
    }

    #[inline]
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(IpldValue::Integer(v.into()))
    }

    #[inline]
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(IpldValue::Integer(v.into()))
    }

    #[inline]
    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(IpldValue::Integer(v))
    }

    #[inline]
    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(IpldValue::Bool(v))
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
        Ok(IpldValue::Null)
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

        Ok(IpldValue::List(vec))
    }

    #[inline]
    fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
    where
        V: de::MapAccess<'de>,
    {
        const LINK_KEY: &str = "/";

        let mut values = Vec::<(SortedStr, IpldValue)>::new();

        while let Some((key, value)) = visitor.next_entry()? {
            values.push((key, value));
        }

        // JSON Object represents IPLD Link if it is `{ "/": "...." }`
        if let Some((key, IpldValue::String(value))) = values.first() {
            if key.as_str() == LINK_KEY && values.len() == 1 {
                let cid = Cid::try_from(value.as_str()).map_err(de::Error::custom)?;
                return Ok(IpldValue::Link(cid));
            }
        }

        let values = values
            .into_iter()
            .collect::<BTreeMap<SortedStr, IpldValue>>();
        Ok(IpldValue::Map(values))
    }

    #[inline]
    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(IpldValue::Float(v))
    }
}

#[cfg(test)]
mod tests {
    use super::{IpldValue, SortedStr};

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

    #[test]
    fn test_ipld_value_cbor_and_json() {
        const TEST_OBJ_ROOT: &str = "tests/test_objects/";

        let content = std::fs::read_to_string(format!("{}expected.json", TEST_OBJ_ROOT)).unwrap();
        let value = serde_json::from_str::<IpldValue>(&content).unwrap();
        if let IpldValue::Map(map) = value {
            for (key, _value) in map {
                println!("key: {}", key);
                let json_file_name = format!("{}{}.json", TEST_OBJ_ROOT, key);
                let json = std::fs::read_to_string(json_file_name).unwrap();
                // println!("json: {:?}", json);
                let json_value = serde_json::from_str::<IpldValue>(&json).unwrap();
                // println!("value from json: {:?}", json_value);
                let cbor_file_name = format!("{}{}.cbor", TEST_OBJ_ROOT, key);
                let cbor = std::fs::read(cbor_file_name).unwrap();
                // println!("cbor: {:?}", cbor);
                let cbor_value = minicbor::decode::<IpldValue>(&cbor).unwrap();
                // println!("value from cbor: {:?}", json_value);
                assert_eq!(json_value, cbor_value);
            }
        }
    }
}
