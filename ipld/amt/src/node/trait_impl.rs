// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::fmt;
use std::ops::Deref;
use std::result;

use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use serde_cbor::Value;

use super::{Amt, Item, Node};
use crate::blocks::Blocks;

impl<B> Serialize for Amt<B>
where
    B: Blocks,
{
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (self.height, self.count, &self.root).serialize(serializer)
    }
}

impl<B> Eq for Amt<B> where B: Blocks {}

impl<B> PartialEq for Amt<B>
where
    B: Blocks,
{
    fn eq(&self, other: &Self) -> bool {
        self.height.eq(&other.height) && self.count.eq(&other.count) && self.root.eq(&other.root)
    }
}

impl<B> fmt::Debug for Amt<B>
where
    B: Blocks,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Root{{ height:{:}, count:{:}, node:{:?} }}",
            self.height, self.count, self.root
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct PartAmt(pub u64, pub u64, pub Node);

impl Serialize for Item {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        match self {
            Item::Link(cid) => cid::ipld_dag_cbor::serialize(cid, serializer),
            Item::Ptr(_) => unreachable!("could not serialize `Ptr`, just allow `Link`"),
        }
    }
}

impl<'de> Deserialize<'de> for Item {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        cid::ipld_dag_cbor::deserialize(deserializer).map(Item::Link)
    }
}

impl Serialize for Node {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let r = self.bitmap.to_be_bytes();
        let bytes: [u8; 1] = [r[r.len() - 1]; 1];
        (
            serde_bytes::Bytes::new(bytes.as_ref()),
            self.branches.borrow().deref(),
            &self.leafs,
        )
            .serialize(serializer)
    }
}

#[derive(Deserialize)]
struct NodeVisitor(serde_bytes::ByteBuf, Vec<Item>, Vec<Value>);
impl<'de> Deserialize<'de> for Node {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let visitor = NodeVisitor::deserialize(deserializer)?;
        if visitor.0.len() != 1 {
            return Err(D::Error::custom(format!(
                "node bitmap must be 1 byte, current is:{:?}",
                visitor.0
            )));
        }
        Ok(Node::new_from_raw(
            visitor.0[0] as usize,
            visitor.1,
            visitor.2,
        ))
    }
}
