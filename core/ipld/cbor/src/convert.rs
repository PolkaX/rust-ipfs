// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::collections::BTreeMap;

use cid::ToCid;

use crate::error::{IpldCborError, Result};
use crate::localcid::CborCid;
use crate::obj::Obj;

/// Convert structure to CBOR Value.
pub fn struct_to_cbor_value<S: serde::Serialize>(v: &S) -> Result<serde_cbor::Value> {
    let s = serde_cbor::to_vec(&v)?;
    let value: serde_cbor::Value = serde_cbor::from_slice(&s)?;
    Ok(value)
}

/// Convert Obj Integer to Obj Float for matching golang version.
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

/// Convert Obj Float to Obj Integer for matching golang version.
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

///
pub fn convert_to_cborish_obj(mut value: Obj) -> Result<Obj> {
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
pub fn convert_to_jsonish_obj(mut value: Obj) -> Result<Obj> {
    let mut func = |obj: &mut Obj| {
        match obj {
            // change cid to map { "/", "string" }
            Obj::Cid(local_cid) => {
                let link = Obj::Text(local_cid.0.to_string());
                let mut map = BTreeMap::new();
                map.insert("/".to_string().into(), link);
                *obj = Obj::Map(map);
                Ok(())
            }
            Obj::Map(ref mut map) => {
                // if current map is like: { "/", cid }, change it to { "/": "string" }
                if map.len() == 1 {
                    if let Some(ref mut cid) = map.get("/") {
                        match cid {
                            Obj::Cid(local_cid) => *cid = &Obj::Text(local_cid.0.to_string()),
                            Obj::Text(_) => {} // do nothing,
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

fn traverse_obj_tree<F>(obj: &mut Obj, f: &mut F) -> Result<()>
where
    F: FnMut(&mut Obj) -> Result<()>,
{
    f(obj)?;
    match obj {
        Obj::Array(ref mut arr) => {
            for v in arr.iter_mut() {
                traverse_obj_tree(v, f)?;
            }
            Ok(())
        }
        Obj::Map(ref mut m) => {
            for v in m.values_mut() {
                traverse_obj_tree(v, f)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
