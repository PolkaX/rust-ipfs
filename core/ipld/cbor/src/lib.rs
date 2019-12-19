// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

#![cfg_attr(feature = "bench", feature(test))]
#[cfg(feature = "bench")]
extern crate test;

#[cfg(feature = "bigint")]
mod bigint;
mod error;
mod localcid;
mod obj;
#[cfg(test)]
mod tests;

use std::result::Result as StdResult;
use std::str::FromStr;

use bytes::Bytes;
use either::*;
use serde::Serialize;

use block_format::{BasicBlock, Block};
use cid::{Cid, Codec};
use ipld_format::{FormatError, Link, Node, NodeStat, Resolver};
use multihash::Hash as MHashEnum;

#[cfg(feature = "bigint")]
pub use self::bigint::CborBigUint;
pub use self::error::{CborError, Result};
pub use self::localcid::CborCid;
pub use self::obj::{
    convert_to_cborish_obj, convert_to_jsonish_obj, hack_convert_float_to_int,
    hack_convert_int_to_float, Obj,
};

/// `IpldNode` represents an IPLD node.
#[derive(Debug, Clone, PartialEq)]
pub struct IpldNode {
    obj: Obj,
    tree: Vec<String>,
    links: Vec<Link>,
    raw: Bytes,
    cid: Cid,
}

impl IpldNode {
    fn new_node(block: &dyn Block, obj: Obj) -> Result<IpldNode> {
        let (tree, links) = compute(&obj)?;
        Ok(IpldNode {
            obj,
            tree,
            links,
            raw: block.raw_data().clone(),
            cid: block.cid().clone(),
        })
    }
}

impl Block for IpldNode {
    fn raw_data(&self) -> &Bytes {
        &self.raw
    }

    fn cid(&self) -> &Cid {
        &self.cid
    }
}

impl Resolver for IpldNode {
    type Output = Either<Link, Obj>;
    /// Resolve resolves a given path, and returns the object found at the end, as well
    /// as the possible tail of the path that was not resolved.
    fn resolve(&self, path: &[&str]) -> StdResult<(Self::Output, Vec<String>), FormatError> {
        let mut cur = &self.obj;
        for (index, val) in path.iter().enumerate() {
            match cur {
                Obj::Map(m) => {
                    cur = m.get::<str>(val).ok_or(FormatError::Other(Box::new(
                        CborError::NoSuchLink(val.clone().to_string()),
                    )))?;
                }
                Obj::Array(arr) => {
                    let index =
                        usize::from_str(val).map_err(|e| FormatError::Other(Box::new(e)))?;
                    cur = arr.get(index).ok_or(FormatError::Other(Box::new(
                        CborError::NoSuchLink(format!("array index out of range[{}]", index)),
                    )))?;
                }
                Obj::Cid(cid) => {
                    let link = Link::new_with_cid(cid.0.clone());
                    return Ok((
                        Left(link),
                        path.iter().skip(index).map(|s| s.clone()).collect(),
                    ));
                }
                _ => return Err(FormatError::Other(Box::new(CborError::NoLinks))),
            }
        }
        if let Obj::Cid(cid) = cur {
            let link = Link::new_with_cid(cid.0.clone());
            return Ok((Left(link), vec![]));
        }
        let jsonish =
            convert_to_jsonish_obj(cur.clone()).map_err(|e| FormatError::Other(Box::new(e)))?;
        Ok((Right(jsonish), vec![]))
    }

    /// Tree returns a flattend array of paths at the given path for the given depth.
    fn tree(&self, path: &str, depth: Option<usize>) -> Vec<String> {
        if path == "" && depth.is_none() {
            return self.tree.clone();
        }
        let mut out = vec![];
        for t in self.tree.iter() {
            if !t.starts_with(path) {
                continue;
            }
            // start from path length.
            // e.g. tree item like "123456/123", path is "123", then s would be "/123"
            // `skip_while` would ignore first chars until meet "/"
            // `skip_while` plus `trim_start_matches` would equal to `strings.TrimLeft` in GO
            // but `s` is allocated, use char.utf8_len to peek slice for `t` could avoid allocate.
            let skip = t.chars().skip(path.len()); // equal to `[len(path):]`
            let s: String = if path.len() != 0 {
                // only filter when path is not "", notice GO not impl for this!
                skip.skip_while(|c| *c != '/').collect()
            } else {
                skip.collect()
            };
            // "/123/123" would be "123/123", "//123/123" would be "123/123"
            let sub = s.trim_start_matches('/');
            if sub == "" {
                // means current tree have no child
                continue;
            }

            match depth {
                None => {
                    // means not filter by depth
                    out.push(sub.to_string());
                    continue;
                }
                Some(dep) => {
                    // for example sub like "123/123/123", and depth is 2, would not peek
                    let parts = sub.split('/').collect::<Vec<_>>();
                    if parts.len() <= dep {
                        out.push(sub.to_string());
                    }
                }
            }
        }
        out
    }
}

impl Node for IpldNode {
    fn resolve_link(&self, path: &[&str]) -> StdResult<(Link, Vec<String>), FormatError> {
        let (either, rest) = self.resolve(path)?;

        match either {
            Left(link) => Ok((link, rest)),
            Right(_) => Err(FormatError::Other(Box::new(CborError::NonLink))),
        }
    }

    fn links(&self) -> Vec<&Link> {
        self.links.iter().collect()
    }

    /// Stat returns stats about the Node.
    fn stat(&self) -> StdResult<&NodeStat, FormatError> {
        // TODO: implement?
        unimplemented!()
    }

    // Size returns the size of the binary representation of the Node.
    fn size(&self) -> u64 {
        self.raw_data().len() as u64
    }
}

// json Serialize/Deserialize
/// Serialize `Node` to json string
pub fn to_json(node: &IpldNode) -> Result<String> {
    // drop other info
    let obj = node.obj.clone();
    obj_to_json(obj)
}

// sample for test
#[inline]
fn obj_to_json(obj: Obj) -> Result<String> {
    let json_obj = convert_to_jsonish_obj(obj)?;
    // hack handle for rust, to match go
    let json_obj = hack_convert_float_to_int(json_obj)?;
    let s = serde_json::to_string(&json_obj)?;
    Ok(s)
}

/// Deserialize json string to `Node`
pub fn from_json(json_str: &str, hash_type: MHashEnum) -> Result<IpldNode> {
    let obj = json_to_obj(json_str)?;
    // need to generate other info
    wrap_obj(obj, hash_type)
}

// sample for test
#[inline]
fn json_to_obj(json_str: &str) -> Result<Obj> {
    let obj = serde_json::from_str::<Obj>(json_str)?;
    // hack handle for rust, to match go
    let obj = hack_convert_int_to_float(obj)?;
    convert_to_cborish_obj(obj)
}

// cbor Serialize/Deserialize
/// Decode decodes a CBOR object into an IPLD Node.
#[inline]
pub fn decode(bytes: &[u8], hash_type: MHashEnum) -> Result<IpldNode> {
    let obj: Obj = serde_cbor::from_slice(bytes)?;
    wrap_obj(obj, hash_type)
}

#[inline]
pub fn dump_object<T: Serialize>(obj: &T) -> Result<Vec<u8>> {
    serde_cbor::to_vec(&obj).map_err(CborError::CborErr)
}

fn wrap_obj(obj: Obj, hash_type: MHashEnum) -> Result<IpldNode> {
    let data = dump_object(&obj)?;
    let hash = multihash::encode(hash_type, &data)?;
    let c = Cid::new_cid_v1(Codec::DagCBOR, hash)?;

    let block = BasicBlock::new_with_cid(data.into(), c)?;
    IpldNode::new_node(&block, obj)
}

fn compute(obj: &Obj) -> Result<(Vec<String>, Vec<Link>)> {
    let mut tree = vec![];
    let mut links = vec![];
    let mut func = |name: String, obj: &Obj| {
        if &name != "" {
            // [1:]
            let name = name.chars().skip(1).collect::<String>();
            tree.push(name);
        }
        if let Obj::Cid(cid) = obj {
            links.push(Link::new_with_cid(cid.0.clone()))
        }
        Ok(())
    };
    traverse(obj, "".to_string(), &mut func)?;
    Ok((tree, links))
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

fn decode_block(block: &impl Block) -> Result<IpldNode> {
    let obj: Obj = serde_cbor::from_slice(block.raw_data())?;
    IpldNode::new_node(block, obj)
}

pub fn decode_block_for_coding(block: &impl Block) -> Result<Box<dyn Node>> {
    let n = decode_block(block).map(|n| Box::new(n))?;
    Ok(n)
}
