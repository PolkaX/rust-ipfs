// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fmt;

use cid::{Cid, IPLD_DAG_CBOR_TAG_CID, RAW_BINARY_MULTIBASE_IDENTITY};
use minicbor::{
    data::{Tag, Type},
    decode, encode, Decoder, Encoder,
};
use serde::{de, ser};

/// The IPLD value.
#[derive(Clone, PartialOrd, PartialEq, Debug)]
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
    Map(BTreeMap<String, IpldValue>),
    /// link value.
    Link(Cid),
}

// Implement CBOR serialization for IpldValue.
impl encode::Encode for IpldValue {
    fn encode<W: encode::Write>(&self, e: &mut Encoder<W>) -> Result<(), encode::Error<W::Error>> {
        match self {
            IpldValue::Null => e.null()?.ok(),
            IpldValue::Bool(v) => e.bool(*v)?.ok(),
            IpldValue::Integer(v) => e.i64(*v as i64)?.ok(),
            IpldValue::Float(v) => e.f64(*v)?.ok(),
            IpldValue::Bytes(v) => e.bytes(v)?.ok(),
            IpldValue::String(v) => e.str(v)?.ok(),
            IpldValue::List(v) => {
                let mut e = e.array(v.len())?;
                for obj in v {
                    e = e.encode(obj)?;
                }
                e.ok()
            }
            IpldValue::Map(v) => {
                let mut e = e.map(v.len())?;
                for (k, v) in v {
                    e = e.str(k)?.encode(v)?;
                }
                e.ok()
            }
            IpldValue::Link(v) => e.encode(v)?.ok(),
        }
    }
}

// Implement CBOR deserialization for IpldValue.
impl<'b> decode::Decode<'b> for IpldValue {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, decode::Error> {
        match d.datatype()? {
            Type::Null => Ok(IpldValue::Null),
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
                    let k = d.str()?.to_owned();
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
            IpldValue::List(list) => list.serialize(serializer),
            IpldValue::Map(map) => map.serialize(serializer),
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

    // Convert IpldValue::Integer to IpldValue::Float for matching golang version
    #[inline]
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        // Ok(IpldValue::Integer(v.into()))
        Ok(IpldValue::Float(v as f64))
    }

    #[inline]
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        // Ok(IpldValue::Integer(v.into()))
        Ok(IpldValue::Float(v as f64))
    }

    #[inline]
    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        // Ok(IpldValue::Integer(v))
        Ok(IpldValue::Float(v as f64))
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

        let mut values = Vec::<(String, IpldValue)>::new();

        while let Some((key, value)) = visitor.next_entry()? {
            values.push((key, value));
        }

        // JSON Object represents IPLD Link if it is `{ "/": "...." }`
        if let Some((key, IpldValue::String(value))) = values.first() {
            if key == LINK_KEY && values.len() == 1 {
                let cid = Cid::try_from(value.as_str()).map_err(de::Error::custom)?;
                return Ok(IpldValue::Link(cid));
            }
        }

        let values = values.into_iter().collect::<BTreeMap<String, IpldValue>>();
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
