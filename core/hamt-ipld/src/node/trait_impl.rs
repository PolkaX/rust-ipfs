// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::collections::BTreeMap;
use std::fmt;

use bigint::U256;
use cid::Cid;
use serde::{de, ser, Deserialize, Serialize};

use super::{Hamt, Item, Node, KVT};
use crate::ipld::CborIpldStore;

impl<B> PartialEq for Hamt<B>
where
    B: CborIpldStore,
{
    fn eq(&self, other: &Self) -> bool {
        self.root == other.root && self.bit_width == other.bit_width
    }
}

impl<B> Eq for Hamt<B> where B: CborIpldStore {}

impl<B> fmt::Debug for Hamt<B>
where
    B: CborIpldStore,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Hamt{{ root: {:?},\n bit_width:{:}}}",
            self.root, self.bit_width
        )
    }
}

impl ser::Serialize for Node {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut bitmap_bytes = [0_u8; std::mem::size_of::<U256>()]; // u256
        self.bitfield.to_big_endian(&mut bitmap_bytes);
        // remove left 0 bytes, if all is 0, means an empty "" bytes.
        let index = bitmap_bytes
            .iter()
            .position(|i| *i != 0)
            .unwrap_or_else(|| std::mem::size_of_val(&self.bitfield));
        let b = serde_bytes::Bytes::new(&bitmap_bytes[index..]);
        let tuple = (b, &self.items);
        tuple.serialize(serializer)
    }
}

impl<'de> de::Deserialize<'de> for Node {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct TupleVisitor;
        impl<'de> de::Visitor<'de> for TupleVisitor {
            type Value = (serde_bytes::ByteBuf, Vec<Item>);

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "tuple must be 2 item, bytes and Vec<Pointer>")
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let first = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let second = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok((first, second))
            }
        }
        let (byte_buf, items) = deserializer.deserialize_tuple(2, TupleVisitor)?;

        // it's big ending bytes, we copy value from end.
        // the buf is size of `u64` u8 array, notice could not out of bounds.
        let mut buf = [0_u8; std::mem::size_of::<U256>()];
        let mut index = std::mem::size_of::<U256>();
        for i in byte_buf.iter().rev() {
            index -= 1;
            buf[index] = *i;
            if index == 0 {
                break;
            }
        }
        // U256 receipt a big ending array
        let bitfield = buf.into();
        Ok(Node::from_raw(bitfield, items))
    }
}

#[derive(Serialize)]
struct CborCid<'a>(#[serde(serialize_with = "cid::ipld_dag_cbor::serialize")] &'a Cid);

impl ser::Serialize for Item {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match self {
            Item::Link(cid) => {
                let mut m = BTreeMap::new();
                m.insert("0", CborCid(cid));
                m.serialize(serializer)
            }
            Item::Leaf(kvs) => {
                let mut m = BTreeMap::new();
                m.insert("1", kvs.iter().collect::<Vec<_>>());
                m.serialize(serializer)
            }
            Item::Ptr(_) => unreachable!("should not happen, could not serialize a node ptr"),
        }
    }
}

#[derive(Deserialize)]
enum ItemRef {
    #[serde(rename = "0")]
    #[serde(deserialize_with = "cid::ipld_dag_cbor::deserialize")]
    Link(Cid),
    #[serde(rename = "1")]
    KVs(Vec<KVT>),
}

impl<'de> de::Deserialize<'de> for Item {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let item_ref = ItemRef::deserialize(deserializer)?;
        let i = match item_ref {
            ItemRef::Link(cid) => Item::from_link(cid),
            ItemRef::KVs(kvs) => Item::from_kvs(kvs),
        };
        Ok(i)
    }
}
