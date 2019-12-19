// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fmt;
use std::ops::{Deref, DerefMut};

use cid::{Cid, ToCid};
use serde::{de, Deserialize, Serialize};
use serde_cbor::Value;

use crate::error::{CborError, Result};
use crate::localcid::CborCid;

#[derive(Clone, Debug, PartialEq)]
//#[serde(untagged)]
pub enum Obj {
    Null,
    Bool(bool),
    Integer(i128),
    Float(f64),
    Bytes(Vec<u8>),
    Text(String),
    Array(Vec<Obj>),
    Map(BTreeMap<SortedStr, Obj>),
    Cid(CborCid),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct SortedStr(pub String);

impl Borrow<str> for SortedStr {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for SortedStr {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for SortedStr {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SortedStr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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

impl serde::Serialize for Obj {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
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
            Obj::Cid(ref v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for Obj {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> de::Visitor<'de> for ValueVisitor {
            type Value = Obj;

            fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt.write_str("any valid CBOR value")
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> ::std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_string(String::from(value))
            }

            #[inline]
            fn visit_string<E>(self, value: String) -> ::std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Obj::Text(value))
            }
            #[inline]
            fn visit_bytes<E>(self, v: &[u8]) -> ::std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_byte_buf(v.to_owned())
            }

            #[inline]
            fn visit_byte_buf<E>(self, v: Vec<u8>) -> ::std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Obj::Bytes(v))
            }

            #[inline]
            fn visit_u64<E>(self, v: u64) -> ::std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Obj::Integer(v.into()))
            }

            #[inline]
            fn visit_i64<E>(self, v: i64) -> ::std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Obj::Integer(v.into()))
            }

            #[inline]
            fn visit_i128<E>(self, v: i128) -> ::std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Obj::Integer(v))
            }

            #[inline]
            fn visit_bool<E>(self, v: bool) -> ::std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Obj::Bool(v))
            }

            #[inline]
            fn visit_none<E>(self) -> ::std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_unit()
            }

            #[inline]
            fn visit_unit<E>(self) -> ::std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Obj::Null)
            }

            #[inline]
            fn visit_seq<V>(self, mut visitor: V) -> ::std::result::Result<Self::Value, V::Error>
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
            fn visit_map<V>(self, mut visitor: V) -> ::std::result::Result<Self::Value, V::Error>
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
            fn visit_f64<E>(self, v: f64) -> ::std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Obj::Float(v))
            }

            fn visit_newtype_struct<D>(
                self,
                deserializer: D,
            ) -> ::std::result::Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let cid = CborCid::deserialize(deserializer)?;
                Ok(Obj::Cid(cid))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

impl TryFrom<serde_cbor::Value> for Obj {
    type Error = ();

    fn try_from(value: Value) -> std::result::Result<Self, Self::Error> {
        match value {
            Value::Null => Ok(Obj::Null),
            Value::Bool(b) => Ok(Obj::Bool(b)),
            Value::Integer(i) => Ok(Obj::Integer(i)),
            Value::Float(f) => Ok(Obj::Float(f)),
            Value::Bytes(b) => Ok(Obj::Bytes(b)),
            Value::Text(s) => Ok(Obj::Text(s)),
            Value::Array(arr) => {
                let mut v = vec![];
                for i in arr {
                    let obj = Obj::try_from(i)?;
                    v.push(obj);
                }
                Ok(Obj::Array(v))
            }
            Value::Map(m) => {
                let mut new_m = BTreeMap::new();
                for (k, v) in m {
                    if let Value::Text(key) = k {
                        new_m.insert(key.into(), Obj::try_from(v)?);
                    } else {
                        // todo handle error
                    }
                }
                Ok(Obj::Map(new_m))
            }
            Value::Tag(tag, v) => {
                // todo handle error
                assert_eq!(tag, 42);
                if let Value::Bytes(res) = *v {
                    if res.len() == 0 {
                        // TODO handle error
                    }

                    if res[0] != 0 {
                        // TODO handle error
                    }

                    // TODO handle error
                    let cid = Cid::from(&res[1..]).unwrap();
                    Ok(Obj::Cid(cid.into()))
                } else {
                    // todo handle error
                    panic!("")
                }
            }
            _ => unreachable!(),
        }
    }
}

pub fn hack_convert_int_to_float(value: Obj) -> Result<Obj> {
    let mut value = value;
    let mut func = |obj: &mut Obj| match obj {
        Obj::Integer(ref mut i) => {
            // all integer would convert into f64
            *obj = Obj::Float(*i as f64);
            Ok(())
        }
        _ => Ok(()),
    };
    traverse_obj_tree(&mut value, &mut func)?;
    Ok(value)
}

pub fn hack_convert_float_to_int(value: Obj) -> Result<Obj> {
    let mut value = value;
    let mut func = |obj: &mut Obj| match obj {
        Obj::Float(ref mut f) => {
            if f.fract() == 0.0 {
                *obj = Obj::Integer(*f as i128);
            }
            Ok(())
        }
        _ => Ok(()),
    };
    traverse_obj_tree(&mut value, &mut func)?;
    Ok(value)
}

pub fn convert_to_cborish_obj(value: Obj) -> Result<Obj> {
    let mut value = value;
    let mut func = |obj: &mut Obj| match obj {
        Obj::Map(ref mut map) => {
            if map.len() == 1 {
                if let Some(link) = map.get("/") {
                    match link {
                        Obj::Text(s) => {
                            let cid = s.to_cid()?;
                            *obj = Obj::Cid(CborCid(cid));
                        }
                        Obj::Cid(cid) => {
                            *obj = Obj::Cid(cid.clone());
                        }
                        _ => return Err(CborError::NonStringLink), // should not happen
                    }
                }
            }
            Ok(())
        }
        _ => Ok(()),
    };
    traverse_obj_tree(&mut value, &mut func)?;
    Ok(value)
}

pub fn convert_to_jsonish_obj(value: Obj) -> Result<Obj> {
    let mut value = value;
    let mut func = |obj: &mut Obj| {
        match obj {
            // change cid to map { "/", "string" }
            Obj::Cid(local_cid) => {
                let link = Obj::Text(local_cid.0.to_string());
                let mut map = BTreeMap::new();
                map.insert("/".to_string().into(), link);
                *obj = Obj::Map(map.into());
                Ok(())
            }
            Obj::Map(ref mut map) => {
                // if current map is like: { "/", cid }, change it to { "/": "string" }
                if map.len() == 1 {
                    if let Some(ref mut cid) = map.get("/") {
                        match cid {
                            Obj::Cid(local_cid) => *cid = &Obj::Text(local_cid.0.to_string()),
                            Obj::Text(_s) => {} // do nothing,
                            _ => return Err(CborError::NonStringLink), // should not happen
                        }
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    };
    traverse_obj_tree(&mut value, &mut func)?;
    Ok(value)
}

fn traverse_obj_tree<F>(obj: &mut Obj, f: &mut F) -> Result<()>
where
    F: FnMut(&mut Obj) -> Result<()>,
{
    f(obj)?;
    match obj {
        Obj::Map(ref mut m) => {
            for (_k, v) in m.iter_mut() {
                traverse_obj_tree(v, f)?;
            }
            Ok(())
        }
        Obj::Array(ref mut arr) => {
            for v in arr.iter_mut() {
                traverse_obj_tree(v, f)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
