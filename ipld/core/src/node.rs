// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::str::FromStr;

use bytes::Bytes;
use cid::{Cid, Codec, IntoExt};
use either::Either;
use minicbor::{encode, Encoder};
use multihash::Code;
use serde::ser;

use block_format::{BasicBlock, Block};
use ipld_format::{FormatError, Link, Node, NodeStat, Resolver};

use crate::error::IpldCoreError;
use crate::value::IpldValue;

/// `IpldNode` represents an IPLD node.
#[derive(Debug, Clone, PartialEq)]
pub struct IpldNode {
    obj: IpldValue,
    tree: Vec<String>,
    links: Vec<Link>,
    raw: Bytes,
    cid: Cid,
}

impl IpldNode {
    fn new_with_obj<B: Block>(block: &B, obj: IpldValue) -> Result<Self, IpldCoreError> {
        let (tree, links) = compute(&obj)?;
        Ok(Self {
            obj,
            tree,
            links,
            raw: block.raw_data().clone(),
            cid: block.cid().clone(),
        })
    }

    /// Deserialize a CBOR object into an IPLD Node.
    ///
    /// Equivalent to the `Decode` in `go-ipld-cbor`
    pub fn from_cbor(cbor: &[u8], hash_type: Code) -> Result<Self, IpldCoreError> {
        let value = minicbor::decode::<IpldValue>(cbor)?;
        println!("Value: {:?}", value);
        Self::wrap_object(&value, hash_type)
    }

    /// Serialize the object of IPLD Node into its CBOR serialized byte representation.
    pub fn to_cbor(&self) -> Result<Vec<u8>, IpldCoreError> {
        Ok(minicbor::to_vec(&self.obj)?)
    }

    /// Deserialize the JSON object into IPLD Node.
    pub fn from_json(json: &str, hash_type: Code) -> Result<Self, IpldCoreError> {
        let value = serde_json::from_str::<IpldValue>(json)?;
        Self::wrap_object(&value, hash_type)
    }

    /// Serialize the object of IPLD Node into its json string representation.
    pub fn to_json(&self) -> Result<String, IpldCoreError> {
        Ok(serde_json::to_string(&self.obj)?)
    }

    /// Convert an CBOR object into IPLD Node.
    pub fn wrap_object<T: minicbor::Encode>(
        value: &T,
        hash_type: Code,
    ) -> Result<Self, IpldCoreError> {
        let data = minicbor::to_vec(value)?;
        // println!("{:?}", data);
        let value = minicbor::decode::<IpldValue>(&data)?;
        let hash = hash_type.digest(&data);
        // println!("Hash: {:?}", hash.as_bytes());
        let cid = Cid::new_v1(Codec::DagCBOR, hash.into_ext());
        let block = BasicBlock::new_with_cid(data.into(), cid)?;
        Self::new_with_obj(&block, value)
    }

    /// Decode a CBOR encoded Block into an IPLD Node.
    ///
    /// In general, you should not be calling this method directly.
    /// Instead, you should be calling the `from_cbor` or `from_json`` method.
    pub fn from_block<B: Block>(block: &B) -> Result<Self, IpldCoreError> {
        let value = minicbor::decode::<IpldValue>(block.raw_data())?;
        Self::new_with_obj(block, value)
    }

    /// Returns obj of the IPLD Node.
    pub fn obj(&self) -> &IpldValue {
        &self.obj
    }
}

// Implement JSON serialization for IpldNode.
// Equivalent to the `to_json`  of `IpldNode`.
impl ser::Serialize for IpldNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        self.obj.serialize(serializer)
    }
}

// Implement CBOR serialization for IpldNode.
// Equivalent to the `to_cbor`  of `IpldNode`.
impl encode::Encode for IpldNode {
    fn encode<W: encode::Write>(&self, e: &mut Encoder<W>) -> Result<(), encode::Error<W::Error>> {
        e.encode(&self.obj)?.ok()
    }
}

impl Block for IpldNode {
    fn raw_data(&self) -> &Bytes {
        &self.raw
    }
}

impl AsRef<Cid> for IpldNode {
    fn as_ref(&self) -> &Cid {
        &self.cid
    }
}

impl Resolver for IpldNode {
    type Output = Either<Link, IpldValue>;

    /// Resolve resolves a given path, and returns the object found at the end, as well
    /// as the possible tail of the path that was not resolved.
    fn resolve(&self, path: &[&str]) -> ipld_format::Result<(Self::Output, Vec<String>)> {
        let mut cur = &self.obj;
        for (index, val) in path.iter().enumerate() {
            match cur {
                IpldValue::Map(m) => {
                    cur = m.get::<str>(val).ok_or_else(|| {
                        FormatError::Other(Box::new(IpldCoreError::NoSuchLink((*val).to_string())))
                    })?;
                }
                IpldValue::List(arr) => {
                    let index =
                        usize::from_str(val).map_err(|e| FormatError::Other(Box::new(e)))?;
                    cur = arr.get(index).ok_or_else(|| {
                        FormatError::Other(Box::new(IpldCoreError::NoSuchLink(format!(
                            "array index out of range[{}]",
                            index
                        ))))
                    })?;
                }
                IpldValue::Link(cid) => {
                    let link = Link::new_with_cid(cid.clone());
                    return Ok((
                        Either::Left(link),
                        path.iter().skip(index).map(|s| (*s).to_string()).collect(),
                    ));
                }
                _ => return Err(FormatError::Other(Box::new(IpldCoreError::NoLinks))),
            }
        }
        if let IpldValue::Link(cid) = cur {
            let link = Link::new_with_cid(cid.clone());
            return Ok((Either::Left(link), vec![]));
        }
        Ok((Either::Right(cur.clone()), vec![]))
    }

    /// Tree returns a flatten array of paths at the given path for the given depth.
    fn tree(&self, path: &str, depth: Option<usize>) -> Vec<String> {
        if path.is_empty() && depth.is_none() {
            return self.tree.clone();
        }
        let mut out = vec![];
        for t in &self.tree {
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
            Either::Right(_) => Err(FormatError::Other(Box::new(IpldCoreError::NonLink))),
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

fn compute(obj: &IpldValue) -> Result<(Vec<String>, Vec<Link>), IpldCoreError> {
    let mut tree = vec![];
    let mut links = vec![];
    let mut func = |name: &str, obj: &IpldValue| {
        if !name.is_empty() {
            // [1:]
            let name = name.chars().skip(1).collect::<String>();
            tree.push(name);
        }
        if let IpldValue::Link(cid) = obj {
            links.push(Link::new_with_cid(cid.clone()))
        }
        Ok(())
    };
    traverse(obj, "", &mut func)?;
    Ok((tree, links))
}

fn traverse<F>(obj: &IpldValue, cur: &str, f: &mut F) -> Result<(), IpldCoreError>
where
    F: FnMut(&str, &IpldValue) -> Result<(), IpldCoreError>,
{
    f(cur, obj)?;
    match obj {
        IpldValue::Map(m) => {
            for (k, v) in m.iter() {
                let this = format!("{}/{}", cur, k);
                traverse(v, &this, f)?;
            }
            Ok(())
        }
        IpldValue::List(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let this = format!("{}/{}", cur, i);
                traverse(v, &this, f)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
