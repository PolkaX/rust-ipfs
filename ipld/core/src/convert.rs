// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::collections::BTreeMap;
use std::convert::TryFrom;

use cid::Cid;

use crate::error::{IpldCborError, Result};
use crate::value::IpldValue;

/// Convert structure to CBOR Value.
pub fn struct_to_cbor_value<S: serde::Serialize>(v: &S) -> Result<serde_cbor::Value> {
    let s = serde_cbor::to_vec(&v)?;
    let value: serde_cbor::Value = serde_cbor::from_slice(&s)?;
    Ok(value)
}

/// Convert CBOR Value to structure.
pub fn cbor_value_to_struct<O: serde::de::DeserializeOwned>(v: serde_cbor::Value) -> Result<O> {
    Ok(serde_cbor::value::from_value(v)?)
}

/// Convert Obj Integer to Obj Float for matching golang version.
pub fn hack_convert_int_to_float(value: IpldValue) -> Result<IpldValue> {
    let mut value = value;
    let mut func = |obj: &mut IpldValue| match obj {
        IpldValue::Integer(ref mut i) => {
            // all integer would convert into f64
            *obj = IpldValue::Float(*i as f64);
            Ok(())
        }
        _ => Ok(()),
    };
    traverse_obj_tree(&mut value, &mut func)?;
    Ok(value)
}

/// Convert Obj Float to Obj Integer for matching golang version.
pub fn hack_convert_float_to_int(value: IpldValue) -> Result<IpldValue> {
    let mut value = value;
    let mut func = |obj: &mut IpldValue| match obj {
        IpldValue::Float(ref mut f) => {
            if f.fract() == 0.0 {
                *obj = IpldValue::Integer(*f as i128);
            }
            Ok(())
        }
        _ => Ok(()),
    };
    traverse_obj_tree(&mut value, &mut func)?;
    Ok(value)
}

///
pub fn convert_to_cborish_obj(mut value: IpldValue) -> Result<IpldValue> {
    let mut func = |obj: &mut IpldValue| match obj {
        IpldValue::Map(ref mut map) => {
            if map.len() == 1 {
                if let Some(link) = map.get("/") {
                    match link {
                        IpldValue::Text(s) => {
                            let cid = Cid::try_from(s.as_str())?;
                            *obj = IpldValue::Link(cid);
                        }
                        IpldValue::Link(cid) => {
                            *obj = IpldValue::Link(cid.clone());
                        }
                        _ => return Err(IpldCborError::NonStringLink), // should not happen
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

///
pub fn convert_to_jsonish_obj(mut value: IpldValue) -> Result<IpldValue> {
    let mut func = |obj: &mut IpldValue| {
        match obj {
            // change cid to map { "/", "string" }
            IpldValue::Link(cid) => {
                let link = IpldValue::Text(cid.to_string());
                let mut map = BTreeMap::new();
                map.insert("/".to_string(), link);
                *obj = IpldValue::Map(map);
                Ok(())
            }
            IpldValue::Map(ref mut map) => {
                // if current map is like: { "/", cid }, change it to { "/": "string" }
                if map.len() == 1 {
                    if let Some(ref mut obj) = map.get("/") {
                        match obj {
                            IpldValue::Link(cid) => *obj = &IpldValue::Text(cid.to_string()),
                            IpldValue::Text(_) => {} // do nothing,
                            _ => return Err(IpldCborError::NonStringLink), // should not happen
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

fn traverse_obj_tree<F>(obj: &mut IpldValue, f: &mut F) -> Result<()>
where
    F: FnMut(&mut IpldValue) -> Result<()>,
{
    f(obj)?;
    match obj {
        IpldValue::Array(ref mut arr) => {
            for v in arr.iter_mut() {
                traverse_obj_tree(v, f)?;
            }
            Ok(())
        }
        IpldValue::Map(ref mut m) => {
            for v in m.values_mut() {
                traverse_obj_tree(v, f)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Convert obj into json string.
/// Just for testing. Please use the `to_json` method of `IpldNode`.
#[inline]
pub fn obj_to_json(obj: IpldValue) -> Result<String> {
    let json_obj = convert_to_jsonish_obj(obj)?;
    // hack handle for rust, to match go
    let json_obj = hack_convert_float_to_int(json_obj)?;
    Ok(serde_json::to_string(&json_obj)?)
}

/// Convert json string into Obj.
/// Just for testing. Please use the `from_json` method of `IpldNode`.
#[inline]
pub fn json_to_obj(json: &str) -> Result<IpldValue> {
    let obj = serde_json::from_str::<IpldValue>(json)?;
    // hack handle for rust, to match go
    let obj = hack_convert_int_to_float(obj)?;
    convert_to_cborish_obj(obj)
}
