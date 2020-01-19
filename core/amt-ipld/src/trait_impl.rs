use std::fmt;
use std::result;

use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use serde_cbor::Value;

use cid::Cid;

use crate::blocks::Blocks;
use crate::{Node, NodeRefLike, Root};
use std::fmt::Debug;

impl<B, NodeRef> Serialize for Root<B, NodeRef>
where
    B: Blocks,
    NodeRef: NodeRefLike<Target=Node<NodeRef>> + Debug,
{
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (self.height, self.count, &self.node).serialize(serializer)
    }
}

impl<B, NodeRef> Eq for Root<B, NodeRef>
where
    B: Blocks,
    NodeRef: NodeRefLike<Target=Node<NodeRef>> + Debug,
{
}

impl<B, NodeRef> PartialEq for Root<B, NodeRef>
where
    B: Blocks,
    NodeRef: NodeRefLike<Target=Node<NodeRef>> + Debug,
{
    fn eq(&self, other: &Self) -> bool {
        self.height.eq(&other.height) && self.count.eq(&other.count) && self.node.eq(&other.node)
    }
}

impl<B, NodeRef> fmt::Debug for Root<B, NodeRef>
where
    B: Blocks,
    NodeRef: NodeRefLike<Target=Node<NodeRef>> + Debug,
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
pub struct PartRoot<NodeRef>(pub u64, pub u64, pub Node<NodeRef>)
where
    NodeRef: NodeRefLike<Target=Node<NodeRef>> + Debug;

impl<NodeRef> PartRoot<NodeRef>
where
    NodeRef: NodeRefLike<Target=Node<NodeRef>> + Debug,
{
    pub fn into_root<B>(self, bs: Rc<RefCell<B>>) -> Root<B, NodeRef>
    where
        B: Blocks,
    {
        Root::<B, NodeRef>::from_partroot(self, bs)
    }
}

impl<NodeRef> Serialize for Node<NodeRef>
where
    NodeRef: NodeRefLike<Target=Node<NodeRef>> + Debug,
{
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
impl<'de, NodeRef> Deserialize<'de> for Node<NodeRef>
where
    NodeRef: NodeRefLike<Target=Node<NodeRef>> + Debug,
{
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

impl<NodeRef> Eq for Node<NodeRef> where NodeRef: NodeRefLike<Target=Node<NodeRef>> + Debug {}

impl<NodeRef> PartialEq for Node<NodeRef>
where
    NodeRef: NodeRefLike<Target=Node<NodeRef>> + Debug,
{
    fn eq(&self, other: &Self) -> bool {
        self.bitmap.eq(&other.bitmap)
            && self.links.eq(&other.links)
            && self.values.eq(&other.values)
    }
}
