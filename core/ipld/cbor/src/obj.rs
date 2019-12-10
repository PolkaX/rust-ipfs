use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use cid::ToCid;

use crate::error::*;
use crate::localcid::LocalCid;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Obj {
    Null,
    Bool(bool),
    Integer(i128),
    Float(f64),
    Bytes(Vec<u8>),
    Text(String),
    Array(Vec<Obj>),
    Map(BTreeMap<String, Obj>),
    Cid(LocalCid),
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
                            *obj = Obj::Cid(LocalCid(cid));
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
                map.insert("/".to_string(), link);
                *obj = Obj::Map(map);
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
