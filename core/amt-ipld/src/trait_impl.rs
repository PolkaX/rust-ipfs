use std::fmt;
use std::result;

use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use serde_cbor::Value;

use cid::Cid;

use crate::blocks::Blocks;
use crate::{Node, Root};

impl<B> Serialize for Root<B>
where
    B: Blocks,
{
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (self.height, self.count, &self.node).serialize(serializer)
    }
}

impl<B> Eq for Root<B> where B: Blocks {}

impl<B> PartialEq for Root<B>
where
    B: Blocks,
{
    fn eq(&self, other: &Self) -> bool {
        self.height.eq(&other.height) && self.count.eq(&other.count) && self.node.eq(&other.node)
    }
}

impl<B> fmt::Debug for Root<B>
where
    B: Blocks,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Root{{ height:{:}, count:{:}, node:{:?} }}",
            self.height, self.count, self.node
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct PartRoot(pub u64, pub u64, pub Node);

impl PartRoot {
    pub fn into_root<B>(self, bs: B) -> Root<B>
    where
        B: Blocks,
    {
        Root::<B>::from_partroot(self, bs)
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
            &self.links,
            &self.values,
        )
            .serialize(serializer)
    }
}

#[derive(Deserialize)]
struct NodeVisitor(serde_bytes::ByteBuf, Vec<Cid>, Vec<Value>);
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

        Ok(Node {
            bitmap: visitor.0[0] as usize,
            links: visitor.1,
            values: visitor.2,
            cache: Default::default(),
        })
    }
}
