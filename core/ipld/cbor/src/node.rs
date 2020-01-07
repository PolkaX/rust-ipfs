// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::str::FromStr;

use block_format::{BasicBlock, Block};
use bytes::Bytes;
use cid::{AsCidRef, Cid, Codec};
use either::Either;
use ipld_format::{FormatError, Link, Node, NodeStat, Resolver};
use multihash::Hash;

use crate::convert::{
    convert_to_cborish_obj, convert_to_jsonish_obj, hack_convert_float_to_int,
    hack_convert_int_to_float,
};
use crate::error::{IpldCborError, Result};
use crate::obj::Obj;

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
    fn new_with_obj(block: &dyn Block, obj: Obj) -> Result<Self> {
        let (tree, links) = compute(&obj)?;
        Ok(Self {
            obj,
            tree,
            links,
            raw: block.raw_data().clone(),
            cid: block.cid().clone(),
        })
    }

    /// Serialize the IPLD Node to json string.
    pub fn to_json(&self) -> Result<String> {
        // drop other info
        let obj = self.obj.clone();
        obj_to_json(obj)
    }

    /// Deserialize the json string to IPLD Node.
    pub fn from_json(json: &str, hash_type: Hash) -> Result<Self> {
        let obj = json_to_obj(json)?;
        // need to generate other info
        Self::from_object(obj, hash_type)
    }

    /// Serialize the object of IPLD Node into its CBOR serialized byte representation.
    pub fn to_cbor(&self) -> Result<Vec<u8>> {
        Ok(serde_cbor::to_vec(&self.obj)?)
    }

    /// Deserialize a CBOR object into an IPLD Node.
    pub fn from_cbor(bytes: &[u8], hash_type: Hash) -> Result<Self> {
        let obj = serde_cbor::from_slice::<Obj>(bytes)?;
        Self::from_object(obj, hash_type)
    }

    /// Just to match the golang version, it will be `deprecated` in the future.
    /// Please use `from_cbor` method of `IpldNode`.
    pub fn decode(bytes: &[u8], hash_type: Hash) -> Result<Self> {
        Self::from_cbor(bytes, hash_type)
    }

    /// Creates an IPLD Node with the given value and hash type.
    pub fn from_object<T: serde::Serialize>(value: T, hash_type: Hash) -> Result<Self> {
        Self::from_object_with_codec(value, hash_type, Codec::DagCBOR)
    }

    /// Creates an IPLD Node with the given value, hash type and codec.
    pub fn from_object_with_codec<T: serde::Serialize>(
        value: T,
        hash_type: Hash,
        codec: Codec,
    ) -> Result<Self> {
        let data = serde_cbor::to_vec(&value)?;
        let obj = serde_cbor::from_slice::<Obj>(&data)?;
        let hash = multihash::encode(hash_type, &data)?;
        let cid = Cid::new_cid_v1(codec, hash)?;
        let block = BasicBlock::new_with_cid(data.into(), cid)?;
        Self::new_with_obj(&block, obj)
    }

    /// Creates an IPLD Node with the given block.
    pub fn from_block(block: &dyn Block) -> Result<Self> {
        let obj = serde_cbor::from_slice::<Obj>(block.raw_data())?;
        Self::new_with_obj(block, obj)
    }

    /// Returns obj of the IPLD Node.
    pub fn obj(&self) -> &Obj {
        &self.obj
    }
}

impl Block for IpldNode {
    fn raw_data(&self) -> &Bytes {
        &self.raw
    }
}

impl AsCidRef for IpldNode {
    fn cid(&self) -> &Cid {
        &self.cid
    }
}

impl Resolver for IpldNode {
    type Output = Either<Link, Obj>;

    /// Resolve resolves a given path, and returns the object found at the end, as well
    /// as the possible tail of the path that was not resolved.
    fn resolve(&self, path: &[&str]) -> ipld_format::Result<(Self::Output, Vec<String>)> {
        let mut cur = &self.obj;
        for (index, val) in path.iter().enumerate() {
            match cur {
                Obj::Map(m) => {
                    cur = m.get::<str>(val).ok_or_else(|| {
                        FormatError::Other(Box::new(IpldCborError::NoSuchLink((*val).to_string())))
                    })?;
                }
                Obj::Array(arr) => {
                    let index =
                        usize::from_str(val).map_err(|e| FormatError::Other(Box::new(e)))?;
                    cur = arr.get(index).ok_or_else(|| {
                        FormatError::Other(Box::new(IpldCborError::NoSuchLink(format!(
                            "array index out of range[{}]",
                            index
                        ))))
                    })?;
                }
                Obj::Cid(cid) => {
                    let link = Link::new_with_cid(cid.clone());
                    return Ok((
                        Either::Left(link),
                        path.iter().skip(index).map(|s| (*s).to_string()).collect(),
                    ));
                }
                _ => return Err(FormatError::Other(Box::new(IpldCborError::NoLinks))),
            }
        }
        if let Obj::Cid(cid) = cur {
            let link = Link::new_with_cid(cid.clone());
            return Ok((Either::Left(link), vec![]));
        }
        let jsonish =
            convert_to_jsonish_obj(cur.clone()).map_err(|e| FormatError::Other(Box::new(e)))?;
        Ok((Either::Right(jsonish), vec![]))
    }

    /// Tree returns a flatten array of paths at the given path for the given depth.
    fn tree(&self, path: &str, depth: Option<usize>) -> Vec<String> {
        if path.is_empty() && depth.is_none() {
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
            let s: String = if !path.is_empty() {
                // only filter when path is not "", notice GO not impl for this!
                skip.skip_while(|c| *c != '/').collect()
            } else {
                skip.collect()
            };
            // "/123/123" would be "123/123", "//123/123" would be "123/123"
            let sub = s.trim_start_matches('/');
            if sub.is_empty() {
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
    fn resolve_link(&self, path: &[&str]) -> ipld_format::Result<(Link, Vec<String>)> {
        let (either, rest) = self.resolve(path)?;

        match either {
            Either::Left(link) => Ok((link, rest)),
            Either::Right(_) => Err(FormatError::Other(Box::new(IpldCborError::NonLink))),
        }
    }

    fn links(&self) -> Vec<&Link> {
        self.links.iter().collect()
    }

    /// Stat returns stats about the Node.
    fn stat(&self) -> ipld_format::Result<&NodeStat> {
        // TODO: implement?
        unimplemented!()
    }

    // Size returns the size of the binary representation of the Node.
    fn size(&self) -> u64 {
        self.raw_data().len() as u64
    }
}

fn compute(obj: &Obj) -> Result<(Vec<String>, Vec<Link>)> {
    let mut tree = vec![];
    let mut links = vec![];
    let mut func = |name: &str, obj: &Obj| {
        if !name.is_empty() {
            // [1:]
            let name = name.chars().skip(1).collect::<String>();
            tree.push(name);
        }
        if let Obj::Cid(cid) = obj {
            links.push(Link::new_with_cid(cid.clone()))
        }
        Ok(())
    };
    traverse(obj, "", &mut func)?;
    Ok((tree, links))
}

fn traverse<F>(obj: &Obj, cur: &str, f: &mut F) -> Result<()>
where
    F: FnMut(&str, &Obj) -> Result<()>,
{
    f(cur, obj)?;
    match obj {
        Obj::Map(m) => {
            for (k, v) in m.iter() {
                let this = format!("{}/{}", cur, k.as_ref());
                traverse(v, &this, f)?;
            }
            Ok(())
        }
        Obj::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let this = format!("{}/{}", cur, i);
                traverse(v, &this, f)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Convert obj into json string.
/// Just for testing. Please use the `to_json` method of `IpldNode`.
#[inline]
pub fn obj_to_json(obj: Obj) -> Result<String> {
    let json_obj = convert_to_jsonish_obj(obj)?;
    // hack handle for rust, to match go
    let json_obj = hack_convert_float_to_int(json_obj)?;
    Ok(serde_json::to_string(&json_obj)?)
}

/// Convert json string into Obj.
/// Just for testing. Please use the `from_json` method of `IpldNode`.
#[inline]
pub fn json_to_obj(json: &str) -> Result<Obj> {
    let obj = serde_json::from_str::<Obj>(json)?;
    // hack handle for rust, to match go
    let obj = hack_convert_int_to_float(obj)?;
    convert_to_cborish_obj(obj)
}
