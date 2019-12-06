mod error;

use std::collections::HashMap;
use std::result;
use std::str::FromStr;

use bytes::Bytes;
use cid::{Cid, ToCid};

use ipld_format::{FormatError, Link, Node as NodeT, Resolver};

pub use crate::error::*;
use serde_json::Value as JValue;

use serde_cbor::Value as CValue;

pub enum Object {
    Cid(Cid),
    Value(CValue),
}

pub fn from_json(json_str: &str) -> Result<()> {
    let mut value = serde_json::from_str(json_str)?;

    Ok(())
}

fn convert_to_cbor(value: JValue) -> Result<Object> {
    let mut value = value;

    //    match value {
    //        Value::Object(map) => {
    //            if map.len() == 0 {
    //                return Ok(Object::Map(HashMap::new()));
    //            }
    //            if map.len() == 1 {
    //                if let Some(link) = map.get("/") {
    //                    let s = link.as_str().ok_or(CborError::NotLink)?;
    //                    let cid = s.to_cid()?;
    //                    return Ok(Object::Cid(cid));
    //                }
    //            }
    //            let mut new_map = HashMap::new();
    //            for (key, value) in map {
    //                let v = convert_to_cbor(value)?;
    //                new_map.insert(key, v);
    //            }
    //            return Ok(Object::Map(new_map));
    //        }
    //        Value::Array(v) => {
    //            if v.len() == 0 {
    //                return Ok(Object::Vec(vec![]));
    //            }
    //            let mut new_vec = vec![];
    //            for i in v {
    //                new_vec.push(convert_to_cbor(i)?);
    //            }
    //            return Ok(Object::Vec(new_vec));
    //        }
    //        _ => Ok(Object::Value(value.to_string())),
    //    }
}

/// Node represents an IPLD node.
pub struct Node {
    obj: Object,
    tree: Vec<String>,
    links: Vec<Link>,
    raw: Bytes,
    cid: Cid,
}

//impl Resolver for Node {
//    fn resolve(&self, path: &[String]) -> result::Result<Vec<String>,  FormatError>{
//        let mut cur: &Object = &self.obj;
//        for (i, s) in path.iter().enumerate() {
//            match cur {
//                Object::Map(m) => {
//                    let next = m.get(s).ok_or(FormatError::NoSuchLink)?;
//                    cur = next;
//                }
//                Object::Vec(v) => {
//                    let n = i32::from_str(s).map_err(|e| FormatError::Other(Box::new(e)))?;
//                    let n = n as usize;
//                    let next = v.get(n).ok_or(FormatError::NoSuchLink)?;
//                    cur = next;
//                }
//                Cid => {
//
//                }
//            }
//        }
//        Ok(vec![])
//    }
//
//    fn tree(&self, path: &str, depth: i32) -> Vec<String> {
//        unimplemented!()
//    }
//}

//impl<T> NodeT for Node<T> {
//    fn resolve_link(&self, path: &str, depth: i32) -> Vec<String> {
//        unimplemented!()
//    }
//
//    fn links(&self) -> Vec<&Link> {
//        unimplemented!()
//    }
//
//    fn stat(&self) -> Result<&NodeStat, FormatError> {
//        unimplemented!()
//    }
//
//    fn size(&self) -> u64 {
//        unimplemented!()
//    }
//}
