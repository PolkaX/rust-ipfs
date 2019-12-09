mod error;

use std::collections::{BTreeMap, HashMap};
use std::result;
use std::str::FromStr;

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_cbor::Value as CValue;
use serde_json::Value as JValue;

use cid::{Cid, Codec, ToCid};
use multihash::Hash as MHashEnum;

use block_format::{BasicBlock, Block};
use ipld_format::{FormatError, Link, Node as NodeT, Resolver};

pub use crate::error::*;
use std::rc::Rc;

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
    Cid(Cid),
}

pub fn from_json(json_str: &str, hash_type: MHashEnum) -> Result<()> {
    let obj = serde_json::from_str::<Obj>(json_str)?;

    let obj = convert_to_cborish_obj(obj)?;

    Ok(())
}

fn convert_to_cborish_obj(value: Obj) -> Result<Obj> {
    let mut value = value;
    match value {
        Obj::Map(map) => {
            if map.len() == 1 {
                if let Some(link) = map.get("/") {
                    if let Obj::Text(s) = link {
                        let cid = s.to_cid()?;
                        return Ok(Obj::Cid(cid));
                    } else {
                        return Err(CborError::NotLink);
                    }
                }
            }
            let mut new_map = BTreeMap::new();
            for (key, value) in map {
                let v = convert_to_cborish_obj(value)?;
                new_map.insert(key, v);
            }
            return Ok(Obj::Map(new_map));
        }
        Obj::Array(v) => {
            let mut new_vec = vec![];
            for i in v {
                new_vec.push(convert_to_cborish_obj(i)?);
            }
            return Ok(Obj::Array(new_vec));
        }
        _ => return Ok(value),
    }
}

fn wrap_obj(obj: Obj, hash_type: MHashEnum) -> Result<()> {
    let data = serde_cbor::to_vec(&obj)?;

    let hash = multihash::encode(hash_type, &data)?;

    let c = Cid::new_cid_v1(Codec::DagCBOR, hash)?;

    let block = BasicBlock::new_with_cid(data.into(), c)?;
    Ok(())
}

//fn new_node(block: &dyn Block, obj: Obj) -> Result<Node> {
//
//}

fn compute(obj: &Obj) -> Result<()> {
    let mut tree = vec![];
    let mut links = vec![];
    let mut func = |name: String, obj: &Obj| {
        if &name != "" {
            // [1:]
            let name = name.chars().skip(1).collect::<String>();
            tree.push(name);
        }
        if let Obj::Cid(cid) = obj {
            links.push(Link {
                name: "".to_string(),
                size: 0,
                cid: cid.clone(),
            })
        }
        Ok(())
    };
    traverse(obj, "".to_string(), &mut func);
    Ok(())
}

fn traverse<F>(obj: &Obj, cur: String, f: &mut F) -> Result<()>
where
    F: FnMut(String, &Obj) -> Result<()>,
{
    f(cur.clone(), obj)?;
    match obj {
        Obj::Map(m) => {
            for (k, v) in m.iter() {
                let this = cur.clone() + "/" + k.as_ref();
                traverse(v, this, f)?;
            }
            Ok(())
        }
        Obj::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let this = cur.clone() + "/" + i.to_string().as_ref();
                traverse(v, this, f)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Node represents an IPLD node.
pub struct Node {
    obj: Obj,
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
//                Obj::Map(m) => {
//                    let next = m.get(s).ok_or(FormatError::NoSuchLink)?;
//                    cur = next;
//                }
//                Obj::Vec(v) => {
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
