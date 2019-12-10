mod error;
mod localcid;
mod obj;

use std::result;
use std::str::FromStr;

use either::*;

use bytes::Bytes;

use cid::{Cid, Codec};
use multihash::Hash as MHashEnum;

use block_format::{BasicBlock, Block};
use ipld_format::{FormatError, Link, Node as NodeT, NodeStat, Resolver};

pub use crate::error::*;
pub use crate::localcid::LocalCid;
pub use crate::obj::{convert_to_cborish_obj, convert_to_jsonish_obj, Obj};

/// Node represents an IPLD node.
pub struct Node {
    obj: Obj,
    tree: Vec<String>,
    links: Vec<Link>,
    raw: Bytes,
    cid: Cid,
}

impl Node {
    fn new_node(block: &dyn Block, obj: Obj) -> Result<Node> {
        let (tree, links) = compute(&obj)?;
        Ok(Node {
            obj,
            tree,
            links,
            raw: block.raw_data().clone(),
            cid: block.cid().clone(),
        })
    }
}

impl Block for Node {
    fn raw_data(&self) -> &Bytes {
        &self.raw
    }

    fn cid(&self) -> &Cid {
        &self.cid
    }
}

impl Resolver for Node {
    type Output = Either<Link, Obj>;
    /// Resolve resolves a given path, and returns the object found at the end, as well
    /// as the possible tail of the path that was not resolved.
    fn resolve(&self, path: &[String]) -> result::Result<(Self::Output, Vec<String>), FormatError> {
        let mut cur = &self.obj;
        for (index, val) in path.iter().enumerate() {
            match cur {
                Obj::Map(m) => {
                    cur = m
                        .get(val)
                        .ok_or(FormatError::Other(Box::new(CborError::NoSuchLink(
                            val.clone(),
                        ))))?;
                }
                Obj::Array(arr) => {
                    let index =
                        usize::from_str(val).map_err(|e| FormatError::Other(Box::new(e)))?;
                    cur = arr.get(index).ok_or(FormatError::Other(Box::new(
                        CborError::NoSuchLink(format!("array index out of range[{}]", index)),
                    )))?;
                }
                Obj::Cid(cid) => {
                    let link = Link::new_default(cid.0.clone());
                    return Ok((
                        Left(link),
                        path.iter().skip(index).map(|s| s.clone()).collect(),
                    ));
                }
                _ => return Err(FormatError::Other(Box::new(CborError::NoLinks))),
            }
        }
        if let Obj::Cid(cid) = cur {
            let link = Link::new_default(cid.0.clone());
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
            let s: String = t
                .chars()
                .skip(path.len())
                .skip_while(|c| *c != '/')
                .collect();
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

impl NodeT for Node {
    fn resolve_link(&self, path: &[String]) -> result::Result<(Link, Vec<String>), FormatError> {
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
    fn stat(&self) -> result::Result<&NodeStat, FormatError> {
        // TODO: implement?
        unimplemented!()
    }

    // Size returns the size of the binary representation of the Node.
    fn size(&self) -> u64 {
        self.raw_data().len() as u64
    }
}

pub fn to_json(node: &Node) -> Result<String> {
    // drop other info
    let obj = node.obj.clone();
    let json_obj = convert_to_jsonish_obj(obj)?;
    let s = serde_json::to_string(&json_obj)?;
    Ok(s)
}

pub fn from_json(json_str: &str, hash_type: MHashEnum) -> Result<Node> {
    let obj = serde_json::from_str::<Obj>(json_str)?;
    let obj = convert_to_cborish_obj(obj)?;
    // need to generate other info
    wrap_obj(obj, hash_type)
}

fn wrap_obj(obj: Obj, hash_type: MHashEnum) -> Result<Node> {
    let data = serde_cbor::to_vec(&obj)?;

    let hash = multihash::encode(hash_type, &data)?;

    let c = Cid::new_cid_v1(Codec::DagCBOR, hash)?;

    let block = BasicBlock::new_with_cid(data.into(), c)?;
    Node::new_node(&block, obj)
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
            links.push(Link::new_default(cid.0.clone()))
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

fn decode_block(block: &dyn Block) -> Result<Node> {
    let obj: Obj = serde_cbor::from_slice(block.raw_data())?;
    Node::new_node(block, obj)
}

pub fn decode_block_for_coding(block: &dyn Block) -> Result<Box<dyn NodeT>> {
    let n = decode_block(block).map(|n| Box::new(n))?;
    Ok(n)
}
